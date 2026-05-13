// Polyform decay shader — multiply previous frame's accumulation by decay factor
//
// Fullscreen triangle reads accum_src, writes accum_dst *= decay_factor.
// decay_factor = pow(persistence, dt * 60.0):
//   persistence=0 → k=0 → clears each frame
//   persistence=1 → k=1 → infinite persistence (DVBST storage tube)

struct DecayUniforms {
    decay_factor: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> decay: DecayUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // Oversized triangle covers entire clip space
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(pos[idx], 0.0, 1.0);
    out.uv = (pos[idx] + 1.0) * 0.5;
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(src_tex, samp, in.uv) * decay.decay_factor;
}
