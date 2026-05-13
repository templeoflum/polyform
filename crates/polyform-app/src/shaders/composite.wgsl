// Polyform composite shader — HDR accumulation → LDR output
//
// Reads the HDR accumulation buffer and optional bloom texture,
// applies palette energy color, saturation, gamma, and background.
// Outputs to the existing offscreen Bgra8Unorm texture for Spout and blit.

struct CompositeUniforms {
    bg_color: vec4<f32>,
    warp_color: vec4<f32>,
    bloom_intensity: f32,
    saturation: f32,
    gamma: f32,
    _pad: f32,
}

@group(0) @binding(0) var accum_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> composite: CompositeUniforms;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let accum = textureSample(accum_tex, samp, in.uv).rgb;
    let bloom = textureSample(bloom_tex, samp, in.uv).rgb;
    let hdr = accum + bloom * composite.bloom_intensity;

    // Palette energy color: shift from grid → warp as energy accumulates
    // Gives the phosphor overload effect (dim green → bright yellow-green)
    let energy = max(hdr.r, max(hdr.g, hdr.b));
    let energy_t = smoothstep(0.5, 2.0, energy);
    let colored = mix(hdr, composite.warp_color.rgb * energy, energy_t);

    // Saturation (BT.709 luminance-preserving)
    let lum = dot(colored, vec3(0.2126, 0.7152, 0.0722));
    let saturated = mix(vec3(lum), colored, composite.saturation);

    // Clamp + gamma
    let ldr = clamp(saturated, vec3(0.0), vec3(1.0));
    let corrected = pow(ldr, vec3(1.0 / composite.gamma));

    // Over background
    return vec4(clamp(composite.bg_color.rgb + corrected, vec3(0.0), vec3(1.0)), 1.0);
}
