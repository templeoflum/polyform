use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};

use polyform_core::frame_state::{FlatLayerState, FrameState};
use polyform_core::palette;
use polyform_core::preset::LatticeParams;
use polyform_core::types::LatticeType;

use crate::camera::build_camera;

// =============================================================================
// Constants
// =============================================================================

/// Samples per grid line (number of segments = SAMPLES, vertices = SAMPLES+1)
const SAMPLES: u32 = 80;
/// Maximum lattice density (Cartesian: 2*(N+1) lines, max N=51)
const MAX_DENSITY: u32 = 51;
/// Maximum lines across all lattice types (104 lines * 81 verts each = 8424)
const MAX_LINES: u32 = 104;
/// Maximum vertices: MAX_LINES * (SAMPLES+1)
const MAX_VERTICES: u32 = MAX_LINES * (SAMPLES + 1);
/// Maximum operations in chain
const MAX_OPS: u32 = 25;
/// Maximum lifts in chain
const MAX_LIFTS: u32 = 15;

// =============================================================================
// GPU data structures (must match WGSL)
// =============================================================================

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GridVertex {
    pub x0: f32,
    pub y0: f32,
    pub line_idx: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TransformedVertex {
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub det_j: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuOpDescriptor {
    pub type_index: u32,
    pub blend: f32,
    pub needs_origin: u32,
    pub num_params: u32,
    pub params: [f32; 4],
    pub center_x: f32,
    pub center_y: f32,
    pub _pad_center: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuLiftDescriptor {
    pub type_index: u32,
    pub blend: f32,
    pub needs_jacobian: u32,
    pub num_params: u32,
    pub params: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct PipelineUniforms {
    pub num_ops: u32,
    pub num_lifts: u32,
    pub any_needs_jacobian: u32,
    pub compute_caustic: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderUniforms {
    pub view_projection: [[f32; 4]; 4],  // 64 bytes
    pub line_color: [f32; 4],            // 16 bytes
    pub k1: f32,                          // 4
    pub k2: f32,                          // 4
    pub k3: f32,                          // 4
    pub k4: f32,                          // 4
    pub p1: f32,                          // 4
    pub p2: f32,                          // 4
    pub distortion_cx: f32,               // 4
    pub distortion_cy: f32,               // 4
    pub squeeze: f32,                     // 4
    pub caustic_on: u32,                  // 4
    pub caustic_threshold: f32,           // 4
    pub beam_width: f32,                  // 4  (replaces _pad)
    // --- new 16-byte block ---
    pub beam_intensity: f32,              // 4
    pub gain: f32,                        // 4
    pub blend_mix: f32,                   // 4
    pub verts_per_line: u32,              // 4  (SAMPLES + 1 = 81)
    // --- new 16-byte block ---
    pub viewport_width: f32,              // 4
    pub viewport_height: f32,             // 4
    pub _pad2: [f32; 2],                  // 8 (align to 16)
}                                         // Total: 160 bytes

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DecayUniforms {
    pub decay_factor: f32,
    pub _pad: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CompositeUniforms {
    pub bg_color: [f32; 4],        // 16
    pub warp_color: [f32; 4],      // 16
    pub bloom_intensity: f32,      // 4
    pub saturation: f32,           // 4
    pub gamma: f32,                // 4
    pub _pad: f32,                 // 4
}                                   // Total: 48 bytes

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BloomUniforms {
    pub texel_size: [f32; 2],      // 8 — 1/width, 1/height of bloom texture
    pub threshold: f32,            // 4
    pub radius: f32,               // 4
}                                   // Total: 16 bytes

// =============================================================================
// Type-index mappings (static)
// =============================================================================

pub fn op_type_index(key: &str) -> u32 {
    match key {
        "translate" => 0,
        "scale" => 1,
        "rotate" => 2,
        "shear" => 3,
        "matrix" => 4,
        "zPow" => 5,
        "zInv" => 6,
        "zExp" => 7,
        "zLog" => 8,
        "zSin" => 9,
        "zCos" => 10,
        "mobius" => 11,
        "zPlusC" => 12,
        "spiral" => 13,
        "pinch" => 14,
        "swirl" => 15,
        "radialSine" => 16,
        "angleWarp" => 17,
        "sineWarp" => 18,
        "ripple" => 19,
        "conjugate" => 20,
        "reflect" => 21,
        "abs" => 22,
        "mixed" => 23,
        "blend" => 24,
        _ => 255, // unknown — passthrough
    }
}

pub fn lift_type_index(key: &str) -> u32 {
    match key {
        "zMagnitude" => 0,
        "zDisplacement" => 1,
        "zLogJac" => 2,
        "zSignedJac" => 3,
        "zReal" => 4,
        "zImag" => 5,
        "zArg" => 6,
        "zSineWave" => 7,
        "zSineXY" => 8,
        "zGaussian" => 9,
        "spherical" => 10,
        "conical" => 11,
        "helical" => 12,
        "noise3D" => 13,
        "ripple3D" => 14,
        _ => 255,
    }
}

/// Param packing order for each operation type (matches registry ParamDef order)
fn op_param_keys(type_key: &str) -> &'static [&'static str] {
    match type_key {
        "translate" => &["tx", "ty"],
        "scale" => &["sx", "sy"],
        "rotate" => &["theta"],
        "shear" => &["kx", "ky"],
        "matrix" => &["a", "b", "c", "d"],
        "zPow" => &["n"],
        "zInv" | "zExp" | "zLog" | "zSin" | "zCos" | "conjugate" | "abs" | "mixed" => &[],
        "mobius" => &["a", "b", "c", "d"],
        "zPlusC" => &["cx", "cy"],
        "spiral" => &["k"],
        "pinch" => &["p"],
        "swirl" => &["k", "sigma"],
        "radialSine" => &["amp", "freq", "phase"],
        "angleWarp" => &["amp", "k"],
        "sineWarp" => &["amp", "freq", "phase"],
        "ripple" => &["amp", "freq", "phase"],
        "reflect" => &["theta"],
        "blend" => &["amount"],
        _ => &[],
    }
}

fn lift_param_keys(type_key: &str) -> &'static [&'static str] {
    match type_key {
        "zMagnitude" | "zDisplacement" | "zLogJac" | "zSignedJac" | "zReal" | "zImag" | "zArg" => &["amp"],
        "zSineWave" => &["amp", "freq", "phase"],
        "zSineXY" => &["amp", "fx", "fy"],
        "zGaussian" => &["amp", "sigma"],
        "spherical" => &["r"],
        "conical" => &["amp"],
        "helical" => &["lift", "twist"],
        "noise3D" => &["amp", "freq", "seed"],
        "ripple3D" => &["amp", "freq", "phase"],
        _ => &[],
    }
}

// =============================================================================
// Grid transition (smooth lattice type morphing)
// =============================================================================

struct GridTransition {
    from_vertices: Vec<GridVertex>,
    to_vertices: Vec<GridVertex>,
    progress: f32,
    duration: f32,
}

/// Pad two vertex arrays to the same length by adding degenerate lines at origin.
fn pad_grid_vertices(from: &mut Vec<GridVertex>, to: &mut Vec<GridVertex>) {
    let verts_per_line = (SAMPLES + 1) as usize;
    let from_lines = from.len() / verts_per_line;
    let to_lines = to.len() / verts_per_line;
    let max_lines = from_lines.max(to_lines);

    while from.len() < max_lines * verts_per_line {
        let line_idx = (from.len() / verts_per_line) as u32;
        for _ in 0..verts_per_line {
            from.push(GridVertex { x0: 0.0, y0: 0.0, line_idx, _pad: 0 });
        }
    }

    while to.len() < max_lines * verts_per_line {
        let line_idx = (to.len() / verts_per_line) as u32;
        for _ in 0..verts_per_line {
            to.push(GridVertex { x0: 0.0, y0: 0.0, line_idx, _pad: 0 });
        }
    }
}

// =============================================================================
// GpuRenderer
// =============================================================================

pub struct GpuRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    // Compute pipeline
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout,

    // Render pipeline (beam quads)
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_layout: wgpu::BindGroupLayout,

    // Decay pipeline (accum_src × k → accum_dst)
    decay_pipeline: wgpu::RenderPipeline,
    decay_bind_group_layout: wgpu::BindGroupLayout,
    decay_uniforms_buf: wgpu::Buffer,

    // Bloom pipelines (extract, blur_h, blur_v)
    bloom_extract_pipeline: wgpu::RenderPipeline,
    bloom_blur_h_pipeline: wgpu::RenderPipeline,
    bloom_blur_v_pipeline: wgpu::RenderPipeline,
    bloom_bind_group_layout: wgpu::BindGroupLayout,
    bloom_uniforms_buf: wgpu::Buffer,

    // Bloom textures (half resolution, Rgba16Float)
    bloom_a_texture: Option<wgpu::Texture>,
    bloom_a_view: Option<wgpu::TextureView>,
    bloom_b_texture: Option<wgpu::Texture>,
    bloom_b_view: Option<wgpu::TextureView>,
    bloom_width: u32,
    bloom_height: u32,

    // Composite pipeline (accum + bloom → offscreen LDR)
    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group_layout: wgpu::BindGroupLayout,
    composite_uniforms_buf: wgpu::Buffer,

    // Blit pipeline (offscreen -> surface)
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    blit_sampler: wgpu::Sampler,

    // Offscreen render target (LDR output for Spout + blit)
    offscreen_texture: Option<wgpu::Texture>,
    offscreen_view: Option<wgpu::TextureView>,
    offscreen_width: u32,
    offscreen_height: u32,
    surface_format: wgpu::TextureFormat,

    // Ping-pong HDR accumulation buffers (Rgba16Float)
    accum_a_texture: Option<wgpu::Texture>,
    accum_a_view: Option<wgpu::TextureView>,
    accum_b_texture: Option<wgpu::Texture>,
    accum_b_view: Option<wgpu::TextureView>,
    accum_width: u32,
    accum_height: u32,
    accum_ping: bool,

    // Buffers
    grid_input_buf: wgpu::Buffer,
    grid_output_buf: wgpu::Buffer,
    pipeline_uniforms_buf: wgpu::Buffer,
    op_chain_buf: wgpu::Buffer,
    lift_chain_buf: wgpu::Buffer,
    render_uniforms_buf: wgpu::Buffer,

    // Current grid state
    current_density: u32,
    current_scale: f32,
    current_lattice_type: LatticeType,
    current_lattice_params: LatticeParams,
    current_vertices: Vec<GridVertex>,
    grid_transition: Option<GridTransition>,
    last_render_instant: Instant,
    vertex_count: u32,
    lines_count: u32,

    // GPU readback for Spout
    readback_buffer: Option<wgpu::Buffer>,
    readback_width: u32,
    readback_height: u32,
}

impl GpuRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // --- Compute shader ---
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/compute.wgsl").into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[
                    // @binding(0): grid_input (storage, read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(1): grid_output (storage, read-write)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(2): pipeline_uniforms (uniform)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(3): op_chain (storage, read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(4): lift_chain (storage, read)
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("compute_pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            });

        // --- Render shader ---
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
        });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_bind_group_layout"),
                entries: &[
                    // @binding(0): vertices (storage, read) — same as grid_output
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(1): render_uniforms (uniform)
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render_pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &render_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[], // No vertex buffers — reading from storage
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &render_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        // Beam quads render into HDR accumulation buffer (Rgba16Float)
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState {
                            // Premultiplied alpha: src_color already has alpha baked in.
                            // blend_mix=1 → alpha output is 0 → pure additive
                            // blend_mix=0 → alpha output is intensity → standard alpha blend
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        // --- Blit shader (offscreen -> surface) ---
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blit.wgsl").into()),
        });

        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blit_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let blit_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blit_pipeline_layout"),
                bind_group_layouts: &[&blit_bind_group_layout],
                push_constant_ranges: &[],
            });

        let blit_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("blit_pipeline"),
                layout: Some(&blit_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &blit_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &blit_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("blit_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // --- Decay shader (accum_src × k → accum_dst) ---
        let decay_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("decay.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/decay.wgsl").into()),
        });

        let decay_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("decay_bind_group_layout"),
                entries: &[
                    // @binding(0): src texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // @binding(1): sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // @binding(2): decay uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let decay_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("decay_pipeline_layout"),
                bind_group_layouts: &[&decay_bind_group_layout],
                push_constant_ranges: &[],
            });

        let hdr_format = wgpu::TextureFormat::Rgba16Float;

        let decay_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("decay_pipeline"),
                layout: Some(&decay_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &decay_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &decay_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: hdr_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        // --- Composite shader (accum + bloom → offscreen LDR) ---
        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
        });

        let composite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("composite_bind_group_layout"),
                entries: &[
                    // @binding(0): accum texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // @binding(1): bloom texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    // @binding(2): sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // @binding(3): composite uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let composite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("composite_pipeline_layout"),
                bind_group_layouts: &[&composite_bind_group_layout],
                push_constant_ranges: &[],
            });

        let composite_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("composite_pipeline"),
                layout: Some(&composite_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &composite_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &composite_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        // --- Bloom pipelines (extract, blur_h, blur_v) ---
        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom.wgsl").into()),
        });

        let bloom_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("bloom_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let bloom_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("bloom_pipeline_layout"),
                bind_group_layouts: &[&bloom_bind_group_layout],
                push_constant_ranges: &[],
            });

        let make_bloom_pipeline = |label: &str, entry_point: &str| -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&bloom_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &bloom_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &bloom_shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: hdr_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            })
        };

        let bloom_extract_pipeline = make_bloom_pipeline("bloom_extract_pipeline", "fs_extract");
        let bloom_blur_h_pipeline = make_bloom_pipeline("bloom_blur_h_pipeline", "fs_blur_h");
        let bloom_blur_v_pipeline = make_bloom_pipeline("bloom_blur_v_pipeline", "fs_blur_v");

        // --- Buffers (pre-allocated for max density) ---
        let grid_input_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid_input"),
            size: (MAX_VERTICES as u64) * std::mem::size_of::<GridVertex>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_output_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grid_output"),
            size: (MAX_VERTICES as u64) * std::mem::size_of::<TransformedVertex>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let pipeline_uniforms_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pipeline_uniforms"),
            size: std::mem::size_of::<PipelineUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let op_chain_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("op_chain"),
            size: (MAX_OPS as u64) * std::mem::size_of::<GpuOpDescriptor>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let lift_chain_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lift_chain"),
            size: (MAX_LIFTS as u64) * std::mem::size_of::<GpuLiftDescriptor>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let render_uniforms_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render_uniforms"),
            size: std::mem::size_of::<RenderUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let decay_uniforms_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("decay_uniforms"),
            size: std::mem::size_of::<DecayUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let composite_uniforms_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite_uniforms"),
            size: std::mem::size_of::<CompositeUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bloom_uniforms_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom_uniforms"),
            size: std::mem::size_of::<BloomUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut renderer = Self {
            device,
            queue,
            compute_pipeline,
            compute_bind_group_layout,
            render_pipeline,
            render_bind_group_layout,
            decay_pipeline,
            decay_bind_group_layout,
            decay_uniforms_buf,
            bloom_extract_pipeline,
            bloom_blur_h_pipeline,
            bloom_blur_v_pipeline,
            bloom_bind_group_layout,
            bloom_uniforms_buf,
            bloom_a_texture: None,
            bloom_a_view: None,
            bloom_b_texture: None,
            bloom_b_view: None,
            bloom_width: 0,
            bloom_height: 0,
            composite_pipeline,
            composite_bind_group_layout,
            composite_uniforms_buf,
            blit_pipeline,
            blit_bind_group_layout,
            blit_sampler,
            offscreen_texture: None,
            offscreen_view: None,
            offscreen_width: 0,
            offscreen_height: 0,
            surface_format,
            accum_a_texture: None,
            accum_a_view: None,
            accum_b_texture: None,
            accum_b_view: None,
            accum_width: 0,
            accum_height: 0,
            accum_ping: true,
            grid_input_buf,
            grid_output_buf,
            pipeline_uniforms_buf,
            op_chain_buf,
            lift_chain_buf,
            render_uniforms_buf,
            current_density: 0,
            current_scale: 0.0,
            current_lattice_type: LatticeType::Cartesian,
            current_lattice_params: LatticeParams::default(),
            current_vertices: Vec::new(),
            grid_transition: None,
            last_render_instant: Instant::now(),
            vertex_count: 0,
            lines_count: 0,
            readback_buffer: None,
            readback_width: 0,
            readback_height: 0,
        };

        // Generate initial grid
        let initial = renderer.generate_grid_vertices(27, 0.6, LatticeType::Cartesian, &LatticeParams::default());
        renderer.upload_grid(&initial);
        renderer.current_vertices = initial;
        renderer.current_density = 27;
        renderer.current_scale = 0.6;
        renderer
    }

    /// Generate grid vertices for a given lattice configuration (pure, no side effects).
    fn generate_grid_vertices(&self, density: u32, scale: f32, lattice_type: LatticeType, params: &LatticeParams) -> Vec<GridVertex> {
        match lattice_type {
            LatticeType::Cartesian => self.generate_cartesian(density, scale),
            LatticeType::Polar => self.generate_polar(density, scale, params),
            LatticeType::Hex => self.generate_hex(density, scale),
            LatticeType::LogPolar => self.generate_log_polar(density, scale, params),
            LatticeType::Bipolar => self.generate_bipolar(density, scale, params),
        }
    }

    /// Upload vertices to GPU and update draw counts.
    fn upload_grid(&mut self, vertices: &[GridVertex]) {
        self.vertex_count = vertices.len() as u32;
        self.lines_count = self.vertex_count / (SAMPLES + 1);
        self.queue.write_buffer(
            &self.grid_input_buf,
            0,
            bytemuck::cast_slice(vertices),
        );
    }

    /// Update grid with smooth transition support for lattice type changes.
    fn update_grid(&mut self, density: u32, scale: f32, lattice_type: LatticeType, params: &LatticeParams) {
        let density = density.min(MAX_DENSITY);

        // Compute dt from wall clock
        let now = Instant::now();
        let dt = now.duration_since(self.last_render_instant).as_secs_f32();
        self.last_render_instant = now;

        let type_changed = lattice_type != self.current_lattice_type;
        let needs_regen = type_changed
            || density != self.current_density
            || (scale - self.current_scale).abs() > 1e-6
            || *params != self.current_lattice_params;

        if needs_regen {
            // Generate new target vertices
            let target = self.generate_grid_vertices(density, scale, lattice_type, params);

            if type_changed && !self.current_vertices.is_empty() {
                // Start a morph transition from current visual state to target.
                // If mid-transition, interpolate current visual to use as "from".
                let from = if let Some(ref tr) = self.grid_transition {
                    let t = 0.5 - 0.5 * (tr.progress * std::f32::consts::PI).cos();
                    tr.from_vertices.iter().zip(tr.to_vertices.iter())
                        .map(|(f, tv)| GridVertex {
                            x0: f.x0 + (tv.x0 - f.x0) * t,
                            y0: f.y0 + (tv.y0 - f.y0) * t,
                            line_idx: f.line_idx,
                            _pad: 0,
                        })
                        .collect()
                } else {
                    self.current_vertices.clone()
                };

                let mut padded_from = from;
                let mut padded_to = target;
                pad_grid_vertices(&mut padded_from, &mut padded_to);

                // Upload 'from' as current frame
                self.upload_grid(&padded_from);

                self.grid_transition = Some(GridTransition {
                    from_vertices: padded_from,
                    to_vertices: padded_to,
                    progress: 0.0,
                    duration: 0.4,
                });
            } else {
                // Instant update (density/scale/param change within same type)
                self.grid_transition = None;
                self.upload_grid(&target);
                self.current_vertices = target;
            }

            self.current_density = density;
            self.current_scale = scale;
            self.current_lattice_type = lattice_type;
            self.current_lattice_params = params.clone();
            return;
        }

        // No regen needed — just advance any active transition
        if let Some(ref mut tr) = self.grid_transition {
            tr.progress += dt / tr.duration;
            if tr.progress >= 1.0 {
                // Transition complete — regenerate clean (unpadded) vertices
                let actual = self.generate_grid_vertices(
                    self.current_density, self.current_scale,
                    self.current_lattice_type, &self.current_lattice_params,
                );
                self.upload_grid(&actual);
                self.current_vertices = actual;
                self.grid_transition = None;
            } else {
                // Cosine ease-in-out interpolation
                let t = 0.5 - 0.5 * (tr.progress * std::f32::consts::PI).cos();
                let interpolated: Vec<GridVertex> = tr.from_vertices.iter()
                    .zip(tr.to_vertices.iter())
                    .map(|(f, tv)| GridVertex {
                        x0: f.x0 + (tv.x0 - f.x0) * t,
                        y0: f.y0 + (tv.y0 - f.y0) * t,
                        line_idx: tv.line_idx,
                        _pad: 0,
                    })
                    .collect();
                self.upload_grid(&interpolated);
            }
        }
    }

    /// Cartesian grid: 2*(N+1) lines in [-scale, scale].
    fn generate_cartesian(&self, density: u32, scale: f32) -> Vec<GridVertex> {
        let n = density as usize;
        let samples = SAMPLES as usize;
        let verts_per_line = samples + 1;
        let total_lines = (n + 1) * 2;
        let mut vertices = Vec::with_capacity(total_lines * verts_per_line);

        // Horizontal lines
        for i in 0..=n {
            let a = -scale + 2.0 * scale * (i as f32) / (n as f32);
            let line_idx = i as u32;
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let y_pos = -scale + 2.0 * scale * t;
                vertices.push(GridVertex { x0: a, y0: y_pos, line_idx, _pad: 0 });
            }
        }

        // Vertical lines
        for i in 0..=n {
            let a = -scale + 2.0 * scale * (i as f32) / (n as f32);
            let line_idx = (n + 1 + i) as u32;
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let x_pos = -scale + 2.0 * scale * t;
                vertices.push(GridVertex { x0: x_pos, y0: a, line_idx, _pad: 0 });
            }
        }

        vertices
    }

    /// Polar grid: concentric rings + radial spokes.
    /// Density drives ring_count; spoke_count from params.
    fn generate_polar(&self, density: u32, scale: f32, params: &LatticeParams) -> Vec<GridVertex> {
        let samples = SAMPLES as usize;
        let spoke_count = params.spoke_count.min(80).max(4);
        let max_rings = (MAX_LINES - spoke_count).min(density);
        let ring_count = max_rings.max(1);

        let mut vertices = Vec::new();
        let mut line_idx: u32 = 0;

        // Rings: concentric circles
        for i in 0..ring_count {
            let r = scale * ((i + 1) as f32) / (ring_count as f32);
            for s in 0..=samples {
                let theta = std::f32::consts::TAU * (s as f32) / (samples as f32);
                vertices.push(GridVertex {
                    x0: r * theta.cos(),
                    y0: r * theta.sin(),
                    line_idx,
                    _pad: 0,
                });
            }
            line_idx += 1;
        }

        // Spokes: radial lines from center to edge
        for j in 0..spoke_count {
            let theta = std::f32::consts::TAU * (j as f32) / (spoke_count as f32);
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let r = scale * t;
                vertices.push(GridVertex {
                    x0: r * cos_t,
                    y0: r * sin_t,
                    line_idx,
                    _pad: 0,
                });
            }
            line_idx += 1;
        }

        vertices
    }

    /// Hex grid: 3 families of parallel lines at 0°, 60°, 120° (triangular lattice),
    /// clipped to a regular hexagonal boundary with circumradius = scale.
    fn generate_hex(&self, density: u32, scale: f32) -> Vec<GridVertex> {
        let samples = SAMPLES as usize;

        // Hexagonal boundary: flat-top regular hexagon, circumradius = scale
        let hex_r = scale;
        let inradius = hex_r * 3.0_f32.sqrt() / 2.0; // R * √3/2

        // Lines per family: must be ODD (2M+1) so grid intersection points
        // fall exactly on hex boundary edges. M = density/3 controls resolution.
        // Reserve 6 lines for the hexagonal boundary outline.
        let m = ((density as usize) / 3).max(1);
        let max_m = (((MAX_LINES as usize) - 6) / 3 - 1) / 2;
        let m = m.min(max_m);
        let n_per_family = 2 * m + 1;

        // Three families at 0°, 60°, 120°
        let family_angles = [
            0.0_f32,
            std::f32::consts::FRAC_PI_3,
            2.0 * std::f32::consts::FRAC_PI_3,
        ];

        // Precompute hexagon edge outward normals at (2k+1) * 30° for k=0..5
        let mut hex_normals = [(0.0_f32, 0.0_f32); 6];
        for k in 0..6 {
            let theta = (2 * k + 1) as f32 * std::f32::consts::FRAC_PI_6;
            hex_normals[k] = (theta.cos(), theta.sin());
        }

        let mut vertices = Vec::new();
        let mut line_idx: u32 = 0;

        for &alpha in &family_angles {
            let dir = (alpha.cos(), alpha.sin());
            let norm = (-alpha.sin(), alpha.cos());

            for i in 0..n_per_family {
                // Offset along normal: span [-inradius, inradius]
                let d = if n_per_family <= 1 {
                    0.0
                } else {
                    -inradius + 2.0 * inradius * (i as f32) / ((n_per_family - 1) as f32)
                };

                let base = (norm.0 * d, norm.1 * d);

                // Clip line to hexagonal boundary using half-plane intersection
                let mut t_lo = f32::NEG_INFINITY;
                let mut t_hi = f32::INFINITY;

                for &(nx, ny) in &hex_normals {
                    let a = dir.0 * nx + dir.1 * ny;
                    let b = base.0 * nx + base.1 * ny;

                    if a.abs() < 1e-9 {
                        // Line parallel to this edge
                        if b > inradius + 1e-6 {
                            t_lo = 1.0;
                            t_hi = 0.0;
                            break;
                        }
                    } else {
                        let t_edge = (inradius - b) / a;
                        if a > 0.0 {
                            t_hi = t_hi.min(t_edge);
                        } else {
                            t_lo = t_lo.max(t_edge);
                        }
                    }
                }

                if t_lo >= t_hi {
                    // Line outside hexagon — emit degenerate point
                    for _ in 0..=samples {
                        vertices.push(GridVertex {
                            x0: base.0,
                            y0: base.1,
                            line_idx,
                            _pad: 0,
                        });
                    }
                } else {
                    // Generate vertices along the clipped segment
                    for s in 0..=samples {
                        let frac = (s as f32) / (samples as f32);
                        let t = t_lo + (t_hi - t_lo) * frac;
                        vertices.push(GridVertex {
                            x0: base.0 + dir.0 * t,
                            y0: base.1 + dir.1 * t,
                            line_idx,
                            _pad: 0,
                        });
                    }
                }
                line_idx += 1;
            }
        }

        // Add 6 hexagonal boundary edges as explicit lines
        for k in 0..6u32 {
            let angle_a = k as f32 * std::f32::consts::FRAC_PI_3;
            let angle_b = (k + 1) as f32 * std::f32::consts::FRAC_PI_3;
            let ax = hex_r * angle_a.cos();
            let ay = hex_r * angle_a.sin();
            let bx = hex_r * angle_b.cos();
            let by = hex_r * angle_b.sin();
            for s in 0..=samples {
                let frac = (s as f32) / (samples as f32);
                vertices.push(GridVertex {
                    x0: ax + (bx - ax) * frac,
                    y0: ay + (by - ay) * frac,
                    line_idx,
                    _pad: 0,
                });
            }
            line_idx += 1;
        }

        vertices
    }

    /// Log-polar grid: logarithmically spaced rings + radial spokes.
    fn generate_log_polar(&self, density: u32, scale: f32, params: &LatticeParams) -> Vec<GridVertex> {
        let samples = SAMPLES as usize;
        let spoke_count = params.spoke_count.min(80).max(4);
        let inner_radius = (params.inner_radius as f32).max(0.001);
        let max_rings = (MAX_LINES - spoke_count).min(density);
        let ring_count = max_rings.max(2);

        let mut vertices = Vec::new();
        let mut line_idx: u32 = 0;

        let ln_ratio = (scale / inner_radius).ln();

        // Rings: logarithmically spaced
        for i in 0..ring_count {
            let r = inner_radius * (i as f32 * ln_ratio / ((ring_count - 1).max(1) as f32)).exp();
            for s in 0..=samples {
                let theta = std::f32::consts::TAU * (s as f32) / (samples as f32);
                vertices.push(GridVertex {
                    x0: r * theta.cos(),
                    y0: r * theta.sin(),
                    line_idx,
                    _pad: 0,
                });
            }
            line_idx += 1;
        }

        // Spokes: radial lines with logarithmically spaced vertices (matching ring spacing)
        let rho_min = inner_radius.ln();
        let rho_max = scale.ln();
        for j in 0..spoke_count {
            let theta = std::f32::consts::TAU * (j as f32) / (spoke_count as f32);
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let rho = rho_min + (rho_max - rho_min) * t;
                let r = rho.exp();
                vertices.push(GridVertex {
                    x0: r * cos_t,
                    y0: r * sin_t,
                    line_idx,
                    _pad: 0,
                });
            }
            line_idx += 1;
        }

        vertices
    }

    /// Bipolar grid: two curve families with foci at (±a, 0).
    fn generate_bipolar(&self, density: u32, scale: f32, params: &LatticeParams) -> Vec<GridVertex> {
        let samples = SAMPLES as usize;
        let tau_count = params.tau_count.min(80).max(4);
        let max_sigma = (MAX_LINES - tau_count).min(density);
        let sigma_count = max_sigma.max(2);
        let a = (params.focus_distance as f32) / 2.0;

        let mut vertices = Vec::new();
        let mut line_idx: u32 = 0;

        let pi = std::f32::consts::PI;

        // Sigma curves (constant sigma, trace tau)
        for i in 0..sigma_count {
            // Distribute sigma in (-pi, pi) excluding 0
            let sigma = -pi + (2.0 * pi) * ((i as f32) + 0.5) / (sigma_count as f32);
            let cos_sigma = sigma.cos();
            let sin_sigma = sigma.sin();

            for s in 0..=samples {
                let tau = -3.0 + 6.0 * (s as f32) / (samples as f32);
                let cosh_tau = tau.cosh();
                let sinh_tau = tau.sinh();
                let denom = cosh_tau - cos_sigma;
                if denom.abs() < 1e-6 {
                    // Degenerate point — place at origin
                    vertices.push(GridVertex { x0: 0.0, y0: 0.0, line_idx, _pad: 0 });
                } else {
                    let x = a * sinh_tau / denom;
                    let y = a * sin_sigma / denom;
                    // Clamp to prevent extreme values
                    let x = x.clamp(-scale * 4.0, scale * 4.0);
                    let y = y.clamp(-scale * 4.0, scale * 4.0);
                    vertices.push(GridVertex { x0: x, y0: y, line_idx, _pad: 0 });
                }
            }
            line_idx += 1;
        }

        // Tau curves (constant tau, trace sigma)
        for j in 0..tau_count {
            let tau = -3.0 + 6.0 * (j as f32) / ((tau_count - 1).max(1) as f32);
            let cosh_tau = tau.cosh();
            let sinh_tau = tau.sinh();

            for s in 0..=samples {
                // Trace sigma in (-pi, pi)
                let sigma = -pi + 2.0 * pi * (s as f32) / (samples as f32);
                let cos_sigma = sigma.cos();
                let sin_sigma = sigma.sin();
                let denom = cosh_tau - cos_sigma;
                if denom.abs() < 1e-6 {
                    vertices.push(GridVertex { x0: 0.0, y0: 0.0, line_idx, _pad: 0 });
                } else {
                    let x = a * sinh_tau / denom;
                    let y = a * sin_sigma / denom;
                    let x = x.clamp(-scale * 4.0, scale * 4.0);
                    let y = y.clamp(-scale * 4.0, scale * 4.0);
                    vertices.push(GridVertex { x0: x, y0: y, line_idx, _pad: 0 });
                }
            }
            line_idx += 1;
        }

        vertices
    }

    /// Ensure the offscreen texture matches the requested viewport dimensions.
    fn ensure_accum_textures(&mut self, width: u32, height: u32) {
        if self.accum_width == width && self.accum_height == height
            && self.accum_a_texture.is_some()
        {
            return;
        }

        let hdr_format = wgpu::TextureFormat::Rgba16Float;
        for (label, tex_slot, view_slot) in [
            ("accum_a", &mut self.accum_a_texture as *mut _, &mut self.accum_a_view as *mut _),
            ("accum_b", &mut self.accum_b_texture as *mut _, &mut self.accum_b_view as *mut _),
        ] {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: hdr_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            // SAFETY: we're assigning to distinct fields through raw pointers
            // to work around the borrow checker with the loop pattern
            unsafe {
                *tex_slot = Some(texture);
                *view_slot = Some(view);
            }
        }
        self.accum_width = width;
        self.accum_height = height;
        self.accum_ping = true;
    }

    fn ensure_bloom_textures(&mut self, width: u32, height: u32) {
        if self.bloom_width == width && self.bloom_height == height
            && self.bloom_a_texture.is_some()
        {
            return;
        }

        let hdr_format = wgpu::TextureFormat::Rgba16Float;
        for (label, tex_slot, view_slot) in [
            ("bloom_a", &mut self.bloom_a_texture as *mut _, &mut self.bloom_a_view as *mut _),
            ("bloom_b", &mut self.bloom_b_texture as *mut _, &mut self.bloom_b_view as *mut _),
        ] {
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: hdr_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            unsafe {
                *tex_slot = Some(texture);
                *view_slot = Some(view);
            }
        }
        self.bloom_width = width;
        self.bloom_height = height;
    }

    /// Get (src, dst) accumulation texture views based on ping-pong state.
    fn accum_views(&self) -> (&wgpu::TextureView, &wgpu::TextureView) {
        if self.accum_ping {
            (self.accum_a_view.as_ref().unwrap(), self.accum_b_view.as_ref().unwrap())
        } else {
            (self.accum_b_view.as_ref().unwrap(), self.accum_a_view.as_ref().unwrap())
        }
    }

    fn ensure_offscreen(&mut self, width: u32, height: u32) {
        if self.offscreen_width == width && self.offscreen_height == height && self.offscreen_texture.is_some() {
            return;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.offscreen_texture = Some(texture);
        self.offscreen_view = Some(view);
        self.offscreen_width = width;
        self.offscreen_height = height;
    }

    /// Render a frame using the given FrameState.
    ///
    /// Renders to an offscreen texture at `render_width x render_height`,
    /// then blits that texture to the surface with letterboxing.
    /// Returns the SurfaceTexture so the caller can render overlays before presenting.
    pub fn render(
        &mut self,
        frame_state: &FrameState,
        render_width: u32,
        render_height: u32,
        dt: f64,
        surface: &wgpu::Surface,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        // 0. Ensure textures are the right size
        self.ensure_accum_textures(render_width, render_height);
        let bloom_w = (render_width / 2).max(1);
        let bloom_h = (render_height / 2).max(1);
        self.ensure_bloom_textures(bloom_w, bloom_h);
        self.ensure_offscreen(render_width, render_height);

        // 1. Update grid (with smooth transitions for lattice type changes)
        let density = frame_state.density.round() as u32;
        let scale = frame_state.scale as f32;
        self.update_grid(density, scale, frame_state.lattice_type, &frame_state.lattice_params);

        // 2. Pack operations into GPU descriptors
        let ops = self.pack_operations(&frame_state.operations);
        let lifts = self.pack_lifts(&frame_state.lifts);

        // 3. Upload pipeline uniforms
        let pipeline_uniforms = PipelineUniforms {
            num_ops: ops.len() as u32,
            num_lifts: lifts.len() as u32,
            any_needs_jacobian: if frame_state.any_lift_needs_jacobian { 1 } else { 0 },
            compute_caustic: if frame_state.show_caustic { 1 } else { 0 },
        };
        self.queue.write_buffer(
            &self.pipeline_uniforms_buf,
            0,
            bytemuck::bytes_of(&pipeline_uniforms),
        );

        // Upload op chain (pad to at least 1 element for valid buffer binding)
        let ops_data = if ops.is_empty() {
            vec![GpuOpDescriptor { type_index: 0, blend: 0.0, needs_origin: 0, num_params: 0, params: [0.0; 4], center_x: 0.0, center_y: 0.0, _pad_center: [0.0; 2] }]
        } else {
            ops
        };
        self.queue.write_buffer(
            &self.op_chain_buf,
            0,
            bytemuck::cast_slice(&ops_data),
        );

        // Upload lift chain
        let lifts_data = if lifts.is_empty() {
            vec![GpuLiftDescriptor { type_index: 0, blend: 0.0, needs_jacobian: 0, num_params: 0, params: [0.0; 4] }]
        } else {
            lifts
        };
        self.queue.write_buffer(
            &self.lift_chain_buf,
            0,
            bytemuck::cast_slice(&lifts_data),
        );

        // 4. Build camera matrices — aspect from viewport, not window
        let aspect = render_width as f32 / render_height as f32;
        let camera = build_camera(frame_state, aspect);

        // Resolve palette colors
        let pal = palette::get_palette(&frame_state.palette_key)
            .unwrap_or_else(|| palette::get_palette("phosphor").unwrap());
        let line_color = parse_hex_color(&pal.grid);
        let bg_color_arr = parse_hex_color(&pal.bg);
        let warp_color_arr = parse_hex_color(&pal.warp);

        let render_uniforms = RenderUniforms {
            view_projection: camera.view_projection.to_cols_array_2d(),
            line_color,
            k1: frame_state.k1 as f32,
            k2: frame_state.k2 as f32,
            k3: frame_state.k3 as f32,
            k4: frame_state.k4 as f32,
            p1: frame_state.p1 as f32,
            p2: frame_state.p2 as f32,
            distortion_cx: frame_state.distortion_cx as f32,
            distortion_cy: frame_state.distortion_cy as f32,
            squeeze: frame_state.squeeze as f32,
            caustic_on: if frame_state.show_caustic { 1 } else { 0 },
            caustic_threshold: 0.1,
            beam_width: frame_state.beam_width as f32,
            beam_intensity: frame_state.beam_intensity as f32,
            gain: frame_state.gain as f32,
            blend_mix: frame_state.blend_mix as f32,
            verts_per_line: SAMPLES + 1,
            viewport_width: render_width as f32,
            viewport_height: render_height as f32,
            _pad2: [0.0; 2],
        };
        self.queue.write_buffer(
            &self.render_uniforms_buf,
            0,
            bytemuck::bytes_of(&render_uniforms),
        );

        // Upload decay uniforms
        let persistence = frame_state.persistence as f32;
        let decay_factor = if persistence <= 0.0 {
            0.0_f32
        } else {
            persistence.powf(dt as f32 * 60.0)
        };
        let decay_uniforms = DecayUniforms {
            decay_factor,
            _pad: [0.0; 3],
        };
        self.queue.write_buffer(
            &self.decay_uniforms_buf,
            0,
            bytemuck::bytes_of(&decay_uniforms),
        );

        // Upload composite uniforms
        let composite_uniforms = CompositeUniforms {
            bg_color: bg_color_arr,
            warp_color: warp_color_arr,
            bloom_intensity: frame_state.bloom_intensity as f32,
            saturation: frame_state.saturation as f32,
            gamma: frame_state.gamma as f32,
            _pad: 0.0,
        };
        self.queue.write_buffer(
            &self.composite_uniforms_buf,
            0,
            bytemuck::bytes_of(&composite_uniforms),
        );

        // 5. Get surface texture
        let output = surface.get_current_texture()?;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        // 6. Compute pass (grid transform — unchanged)
        {
            let compute_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("compute_bind_group"),
                layout: &self.compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.grid_input_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.grid_output_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: self.pipeline_uniforms_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: self.op_chain_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 4, resource: self.lift_chain_buf.as_entire_binding() },
                ],
            });

            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &compute_bind_group, &[]);
            let workgroups = (self.vertex_count + 63) / 64;
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Get accum views (src, dst) for this frame
        let (accum_src_view, accum_dst_view) = self.accum_views();
        // Need raw pointers to avoid borrow issues with &self
        let accum_src_ptr = accum_src_view as *const wgpu::TextureView;
        let accum_dst_ptr = accum_dst_view as *const wgpu::TextureView;

        // 7. Decay pass: accum_src × k → accum_dst
        {
            let decay_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("decay_bind_group"),
                layout: &self.decay_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(unsafe { &*accum_src_ptr }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.decay_uniforms_buf.as_entire_binding(),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("decay_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: unsafe { &*accum_dst_ptr },
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.decay_pipeline);
            pass.set_bind_group(0, &decay_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // 8. Beam render pass: instanced quads → accum_dst (additive, LoadOp::Load)
        {
            let render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group"),
                layout: &self.render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.grid_output_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.render_uniforms_buf.as_entire_binding() },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("beam_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: unsafe { &*accum_dst_ptr },
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &render_bind_group, &[]);
            let num_segments = self.lines_count * SAMPLES;
            pass.draw(0..6, 0..num_segments);
        }

        // 9. Bloom passes (skip when bloom_intensity = 0)
        let bloom_active = frame_state.bloom_intensity > 0.0;
        let bloom_a_view_ptr = self.bloom_a_view.as_ref().unwrap() as *const wgpu::TextureView;
        let bloom_b_view_ptr = self.bloom_b_view.as_ref().unwrap() as *const wgpu::TextureView;

        if bloom_active {
            // Upload bloom uniforms
            let bloom_uniforms = BloomUniforms {
                texel_size: [1.0 / bloom_w as f32, 1.0 / bloom_h as f32],
                threshold: frame_state.bloom_threshold as f32,
                radius: frame_state.bloom_radius as f32,
            };
            self.queue.write_buffer(
                &self.bloom_uniforms_buf,
                0,
                bytemuck::bytes_of(&bloom_uniforms),
            );

            // 9a. Extract: accum_dst → bloom_a
            {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bloom_extract_bind_group"),
                    layout: &self.bloom_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(unsafe { &*accum_dst_ptr }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.bloom_uniforms_buf.as_entire_binding(),
                        },
                    ],
                });

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bloom_extract_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: unsafe { &*bloom_a_view_ptr },
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bloom_extract_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }

            // 9b. Blur H: bloom_a → bloom_b
            {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bloom_blur_h_bind_group"),
                    layout: &self.bloom_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(unsafe { &*bloom_a_view_ptr }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.bloom_uniforms_buf.as_entire_binding(),
                        },
                    ],
                });

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bloom_blur_h_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: unsafe { &*bloom_b_view_ptr },
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bloom_blur_h_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }

            // 9c. Blur V: bloom_b → bloom_a (final bloom result)
            {
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bloom_blur_v_bind_group"),
                    layout: &self.bloom_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(unsafe { &*bloom_b_view_ptr }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.bloom_uniforms_buf.as_entire_binding(),
                        },
                    ],
                });

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("bloom_blur_v_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: unsafe { &*bloom_a_view_ptr },
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bloom_blur_v_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
        }

        // 10. Composite pass: accum_dst + bloom_a → offscreen (LDR)
        let offscreen_view = self.offscreen_view.as_ref().unwrap();
        {
            // bloom_a has the final bloom result (or is black if bloom was skipped)
            let composite_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite_bind_group"),
                layout: &self.composite_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(unsafe { &*accum_dst_ptr }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(unsafe { &*bloom_a_view_ptr }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.composite_uniforms_buf.as_entire_binding(),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: offscreen_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.composite_pipeline);
            pass.set_bind_group(0, &composite_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // 10. Blit pass: offscreen → surface with letterboxing
        {
            let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let blit_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("blit_bind_group"),
                layout: &self.blit_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(offscreen_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                ],
            });

            let (vp_x, vp_y, vp_w, vp_h) = letterbox_viewport(
                render_width, render_height,
                surface_config.width, surface_config.height,
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_viewport(vp_x, vp_y, vp_w, vp_h, 0.0, 1.0);
            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, &blit_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // 11. Swap ping-pong for next frame
        self.accum_ping = !self.accum_ping;

        Ok(output)
    }

    /// Get the offscreen texture for external readback (e.g., Spout).
    pub fn offscreen_texture(&self) -> Option<&wgpu::Texture> {
        self.offscreen_texture.as_ref()
    }

    /// Get the current offscreen dimensions.
    pub fn offscreen_dims(&self) -> (u32, u32) {
        (self.offscreen_width, self.offscreen_height)
    }

    /// Read back the offscreen texture's pixel data.
    /// Wrapper that avoids the split-borrow issue of calling
    /// `readback_texture(self.offscreen_texture(), ...)`.
    pub fn readback_offscreen(&mut self, width: u32, height: u32) -> Option<Vec<u8>> {
        let texture = self.offscreen_texture.as_ref()?;
        // SAFETY: readback_texture only mutates readback_buffer/width/height fields,
        // not offscreen_texture. The borrow checker can't prove this, so we use a
        // raw pointer to split the borrow.
        let tex_ptr: *const wgpu::Texture = texture;
        self.readback_texture(unsafe { &*tex_ptr }, width, height)
    }

    // =========================================================================
    // Packing helpers
    // =========================================================================

    fn pack_operations(&self, layers: &[FlatLayerState]) -> Vec<GpuOpDescriptor> {
        layers
            .iter()
            .filter(|l| l.blend > 0.0)
            .map(|layer| {
                let type_index = op_type_index(&layer.type_key);
                let param_keys = op_param_keys(&layer.type_key);
                let mut params = [0.0_f32; 4];
                for (i, key) in param_keys.iter().enumerate().take(4) {
                    params[i] = layer.params.get(*key).copied().unwrap_or(0.0) as f32;
                }
                let needs_origin = if layer.type_key == "blend" { 1 } else { 0 };
                let center_x = layer.params.get("cx").copied().unwrap_or(0.0) as f32;
                let center_y = layer.params.get("cy").copied().unwrap_or(0.0) as f32;
                GpuOpDescriptor {
                    type_index,
                    blend: layer.blend as f32,
                    needs_origin,
                    num_params: param_keys.len().min(4) as u32,
                    params,
                    center_x,
                    center_y,
                    _pad_center: [0.0; 2],
                }
            })
            .collect()
    }

    fn pack_lifts(&self, layers: &[FlatLayerState]) -> Vec<GpuLiftDescriptor> {
        let jacobian_lifts = ["zLogJac", "zSignedJac"];
        layers
            .iter()
            .filter(|l| l.blend > 0.0)
            .map(|layer| {
                let type_index = lift_type_index(&layer.type_key);
                let param_keys = lift_param_keys(&layer.type_key);
                let mut params = [0.0_f32; 4];
                for (i, key) in param_keys.iter().enumerate().take(4) {
                    params[i] = layer.params.get(*key).copied().unwrap_or(0.0) as f32;
                }
                let needs_jacobian = if jacobian_lifts.contains(&layer.type_key.as_str()) { 1 } else { 0 };
                GpuLiftDescriptor {
                    type_index,
                    blend: layer.blend as f32,
                    needs_jacobian,
                    num_params: param_keys.len().min(4) as u32,
                    params,
                }
            })
            .collect()
    }

    /// Read back a texture's pixel data to the CPU.
    ///
    /// Returns BGRA pixel data (width * height * 4 bytes) with row padding
    /// stripped. The readback buffer is lazily created and reused across frames.
    pub fn readback_texture(
        &mut self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Option<Vec<u8>> {
        let bytes_per_pixel: u32 = 4;
        let unpadded_row = width * bytes_per_pixel;
        // wgpu requires rows aligned to 256 bytes for buffer copies
        let align: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row = (unpadded_row + align - 1) / align * align;
        let buffer_size = (padded_row * height) as u64;

        // Recreate buffer if dimensions changed
        if self.readback_width != width || self.readback_height != height {
            self.readback_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("readback_buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.readback_width = width;
            self.readback_height = height;
        }

        let buffer = self.readback_buffer.as_ref()?;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("readback_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Synchronous map — blocks until GPU is done
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device.poll(wgpu::Maintain::Wait);

        if rx.recv().ok()?.is_err() {
            return None;
        }

        let mapped = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((unpadded_row * height) as usize);

        // Strip row padding
        for row in 0..height {
            let start = (row * padded_row) as usize;
            let end = start + unpadded_row as usize;
            pixels.extend_from_slice(&mapped[start..end]);
        }

        drop(mapped);
        buffer.unmap();

        Some(pixels)
    }
}

// =============================================================================
// Letterbox viewport calculation
// =============================================================================

/// Compute the viewport rect to letterbox/pillarbox the render resolution
/// into the surface dimensions.
fn letterbox_viewport(
    render_w: u32, render_h: u32,
    surface_w: u32, surface_h: u32,
) -> (f32, f32, f32, f32) {
    let render_aspect = render_w as f32 / render_h as f32;
    let surface_aspect = surface_w as f32 / surface_h as f32;
    if render_aspect > surface_aspect {
        // Render is wider — bars top/bottom
        let scaled_h = surface_w as f32 / render_aspect;
        let y_offset = (surface_h as f32 - scaled_h) / 2.0;
        (0.0, y_offset, surface_w as f32, scaled_h)
    } else {
        // Render is taller — bars left/right
        let scaled_w = surface_h as f32 * render_aspect;
        let x_offset = (surface_w as f32 - scaled_w) / 2.0;
        (x_offset, 0.0, scaled_w, surface_h as f32)
    }
}

// =============================================================================
// Color parsing
// =============================================================================

/// Parse a CSS hex color ("#rgb" or "#rrggbb") into [r, g, b, a] as f32 in [0,1].
fn parse_hex_color(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).unwrap_or(0);
            [
                (r * 17) as f32 / 255.0,
                (g * 17) as f32 / 255.0,
                (b * 17) as f32 / 255.0,
                1.0,
            ]
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                1.0,
            ]
        }
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}
