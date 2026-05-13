// Polyform bloom shader — far-field halation (CRT glass scatter)
//
// Three passes at half resolution:
//   1. Extract bright pixels (soft threshold) from HDR accumulation
//   2. Horizontal 9-tap separable Gaussian blur
//   3. Vertical 9-tap separable Gaussian blur
//
// Uses a shared fullscreen triangle vertex shader and BloomUniforms.

struct BloomUniforms {
    texel_size: vec2<f32>,  // 1/width, 1/height of bloom texture
    threshold: f32,
    radius: f32,
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> bloom: BloomUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
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

// --- Extract pass: soft-threshold bright pixels ---
@fragment
fn fs_extract(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(src_tex, samp, in.uv).rgb;
    let brightness = max(color.r, max(color.g, color.b));
    let t = bloom.threshold;
    let soft = smoothstep(t - 0.1, t + 0.1, brightness);
    return vec4<f32>(color * soft, 1.0);
}

// 9-tap Gaussian kernel weights (σ ≈ 2.0)
const WEIGHTS = array<f32, 5>(
    0.2270270270,  // center
    0.1945945946,  // ±1
    0.1216216216,  // ±2
    0.0540540541,  // ±3
    0.0162162162,  // ±4
);

// --- Horizontal blur ---
@fragment
fn fs_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    let step = vec2<f32>(bloom.texel_size.x * bloom.radius, 0.0);
    var result = textureSample(src_tex, samp, in.uv).rgb * WEIGHTS[0];
    for (var i = 1; i < 5; i++) {
        let offset = step * f32(i);
        result += textureSample(src_tex, samp, in.uv + offset).rgb * WEIGHTS[i];
        result += textureSample(src_tex, samp, in.uv - offset).rgb * WEIGHTS[i];
    }
    return vec4<f32>(result, 1.0);
}

// --- Vertical blur ---
@fragment
fn fs_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    let step = vec2<f32>(0.0, bloom.texel_size.y * bloom.radius);
    var result = textureSample(src_tex, samp, in.uv).rgb * WEIGHTS[0];
    for (var i = 1; i < 5; i++) {
        let offset = step * f32(i);
        result += textureSample(src_tex, samp, in.uv + offset).rgb * WEIGHTS[i];
        result += textureSample(src_tex, samp, in.uv - offset).rgb * WEIGHTS[i];
    }
    return vec4<f32>(result, 1.0);
}
