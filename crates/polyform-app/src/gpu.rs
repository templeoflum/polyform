use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use polyform_core::frame_state::{FlatLayerState, FrameState};
use polyform_core::palette;

use crate::camera::build_camera;

// =============================================================================
// Constants
// =============================================================================

/// Samples per grid line (number of segments = SAMPLES, vertices = SAMPLES+1)
const SAMPLES: u32 = 80;
/// Maximum lattice density
const MAX_DENSITY: u32 = 51;
/// Maximum vertices: (MAX_DENSITY+1) * 2 * (SAMPLES+1)
const MAX_VERTICES: u32 = (MAX_DENSITY + 1) * 2 * (SAMPLES + 1);
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
    pub _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuOpDescriptor {
    pub type_index: u32,
    pub blend: f32,
    pub needs_origin: u32,
    pub num_params: u32,
    pub params: [f32; 4],
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
    pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RenderUniforms {
    pub view_projection: [[f32; 4]; 4],
    pub line_color: [f32; 4],
    pub k1: f32,
    pub k2: f32,
    pub _pad2: [f32; 2],
}

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
// GpuRenderer
// =============================================================================

pub struct GpuRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    // Compute pipeline
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group_layout: wgpu::BindGroupLayout,

    // Render pipeline
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_layout: wgpu::BindGroupLayout,

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
    vertex_count: u32,
    lines_count: u32,
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
                    topology: wgpu::PrimitiveTopology::LineStrip,
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
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

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

        let mut renderer = Self {
            device,
            queue,
            compute_pipeline,
            compute_bind_group_layout,
            render_pipeline,
            render_bind_group_layout,
            grid_input_buf,
            grid_output_buf,
            pipeline_uniforms_buf,
            op_chain_buf,
            lift_chain_buf,
            render_uniforms_buf,
            current_density: 0,
            current_scale: 0.0,
            vertex_count: 0,
            lines_count: 0,
        };

        // Generate initial grid
        renderer.regenerate_grid(27, 0.6);
        renderer
    }

    /// Regenerate grid input buffer when density or scale changes.
    fn regenerate_grid(&mut self, density: u32, scale: f32) {
        let density = density.min(MAX_DENSITY);
        if density == self.current_density && (scale - self.current_scale).abs() < 1e-6 {
            return;
        }

        self.current_density = density;
        self.current_scale = scale;

        let n = density as usize;
        let samples = SAMPLES as usize;
        let verts_per_line = samples + 1;
        let total_lines = (n + 1) * 2;
        let total_verts = total_lines * verts_per_line;

        let mut vertices = Vec::with_capacity(total_verts);

        // Horizontal lines: for each i in 0..=n, line along y-axis at fixed x
        for i in 0..=n {
            let a = -scale + 2.0 * scale * (i as f32) / (n as f32);
            let line_idx = i as u32;
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let y_pos = -scale + 2.0 * scale * t;
                vertices.push(GridVertex {
                    x0: a,
                    y0: y_pos,
                    line_idx,
                    _pad: 0,
                });
            }
        }

        // Vertical lines: for each i in 0..=n, line along x-axis at fixed y
        for i in 0..=n {
            let a = -scale + 2.0 * scale * (i as f32) / (n as f32);
            let line_idx = (n + 1 + i) as u32;
            for s in 0..=samples {
                let t = (s as f32) / (samples as f32);
                let x_pos = -scale + 2.0 * scale * t;
                vertices.push(GridVertex {
                    x0: x_pos,
                    y0: a,
                    line_idx,
                    _pad: 0,
                });
            }
        }

        self.vertex_count = vertices.len() as u32;
        self.lines_count = total_lines as u32;

        self.queue.write_buffer(
            &self.grid_input_buf,
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    /// Render a frame using the given FrameState.
    /// Returns the SurfaceTexture so the caller can render overlays before presenting.
    pub fn render(
        &mut self,
        frame_state: &FrameState,
        surface: &wgpu::Surface,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        // 1. Regenerate grid if density/scale changed
        let density = frame_state.density.round() as u32;
        let scale = frame_state.scale as f32;
        self.regenerate_grid(density, scale);

        // 2. Pack operations into GPU descriptors
        let ops = self.pack_operations(&frame_state.operations);
        let lifts = self.pack_lifts(&frame_state.lifts);

        // 3. Upload pipeline uniforms
        let pipeline_uniforms = PipelineUniforms {
            num_ops: ops.len() as u32,
            num_lifts: lifts.len() as u32,
            any_needs_jacobian: if frame_state.any_lift_needs_jacobian { 1 } else { 0 },
            _pad: 0,
        };
        self.queue.write_buffer(
            &self.pipeline_uniforms_buf,
            0,
            bytemuck::bytes_of(&pipeline_uniforms),
        );

        // Upload op chain (pad to at least 1 element for valid buffer binding)
        let ops_data = if ops.is_empty() {
            vec![GpuOpDescriptor { type_index: 0, blend: 0.0, needs_origin: 0, num_params: 0, params: [0.0; 4] }]
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

        // 4. Build camera matrices
        let aspect = surface_config.width as f32 / surface_config.height as f32;
        let camera = build_camera(frame_state, aspect);

        // Resolve palette colors
        let pal = palette::get_palette(&frame_state.palette_key)
            .unwrap_or_else(|| palette::get_palette("phosphor").unwrap());
        // Always use grid color for the lattice lines — the color represents
        // the grid substrate itself, not whether transforms are active.
        let line_color = parse_hex_color(&pal.grid);

        let render_uniforms = RenderUniforms {
            view_projection: camera.view_projection.to_cols_array_2d(),
            line_color,
            k1: frame_state.k1 as f32,
            k2: frame_state.k2 as f32,
            _pad2: [0.0; 2],
        };
        self.queue.write_buffer(
            &self.render_uniforms_buf,
            0,
            bytemuck::bytes_of(&render_uniforms),
        );

        // 5. Get surface texture
        let output = surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame_encoder"),
        });

        // 6. Compute pass
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

        // 7. Render pass
        {
            let render_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("render_bind_group"),
                layout: &self.render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: self.grid_output_buf.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.render_uniforms_buf.as_entire_binding() },
                ],
            });

            let bg = parse_hex_color(&pal.bg);
            let bg_color = wgpu::Color {
                r: bg[0] as f64,
                g: bg[1] as f64,
                b: bg[2] as f64,
                a: 1.0,
            };

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(bg_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &render_bind_group, &[]);

            // Draw each line as a separate line strip
            let verts_per_line = (SAMPLES + 1) as u32;
            for line_i in 0..self.lines_count {
                let start = line_i * verts_per_line;
                pass.draw(start..start + verts_per_line, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(output)
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
                GpuOpDescriptor {
                    type_index,
                    blend: layer.blend as f32,
                    needs_origin,
                    num_params: param_keys.len().min(4) as u32,
                    params,
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
