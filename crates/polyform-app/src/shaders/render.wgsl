// Polyform render shader — vertex transform with Brown-Conrady lens distortion

struct TransformedVertex {
    vx: f32,
    vy: f32,
    vz: f32,
    _pad: f32,
}

struct RenderUniforms {
    view_projection: mat4x4<f32>,
    line_color: vec4<f32>,
    k1: f32,
    k2: f32,
    _pad2: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> vertices: array<TransformedVertex>;
@group(0) @binding(1) var<uniform> render_uniforms: RenderUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let v = vertices[idx];
    let world_pos = vec4<f32>(v.vx, v.vy, v.vz, 1.0);
    var clip = render_uniforms.view_projection * world_pos;

    // Brown-Conrady radial lens distortion in NDC space
    // r' = r * (1 + k1*r^2 + k2*r^4)
    // Negative k = barrel (fisheye), positive k = pincushion
    let k1 = render_uniforms.k1;
    let k2 = render_uniforms.k2;
    if (k1 != 0.0 || k2 != 0.0) {
        let ndc = clip.xy / clip.w;
        let r2 = dot(ndc, ndc);
        let r4 = r2 * r2;
        let factor = 1.0 + k1 * r2 + k2 * r4;
        clip = vec4<f32>(ndc * factor * clip.w, clip.z, clip.w);
    }

    var out: VertexOutput;
    out.position = clip;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return render_uniforms.line_color;
}
