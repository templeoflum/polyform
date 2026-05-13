// Polyform render shader — vertex transform with Brown-Conrady lens distortion + caustic overlay

struct TransformedVertex {
    vx: f32,
    vy: f32,
    vz: f32,
    det_j: f32,
}

struct RenderUniforms {
    view_projection: mat4x4<f32>,
    line_color: vec4<f32>,
    k1: f32,
    k2: f32,
    k3: f32,
    k4: f32,
    p1: f32,
    p2: f32,
    distortion_cx: f32,
    distortion_cy: f32,
    squeeze: f32,
    caustic_on: u32,
    caustic_threshold: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read> vertices: array<TransformedVertex>;
@group(0) @binding(1) var<uniform> render_uniforms: RenderUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) det_j: f32,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let v = vertices[idx];
    let world_pos = vec4<f32>(v.vx, v.vy, v.vz, 1.0);
    var clip = render_uniforms.view_projection * world_pos;

    // Full Brown-Conrady lens distortion in NDC space
    // Radial: r' = r * (1 + k1*r^2 + k2*r^4 + k3*r^6 + k4*r^8)
    // Tangential: dx = 2*p1*x*y + p2*(r^2 + 2*x^2), dy = p1*(r^2 + 2*y^2) + 2*p2*x*y
    // Center offset + anisotropic squeeze
    let k1 = render_uniforms.k1;
    let k2 = render_uniforms.k2;
    let k3 = render_uniforms.k3;
    let k4 = render_uniforms.k4;
    let rp1 = render_uniforms.p1;
    let rp2 = render_uniforms.p2;
    let dcx = render_uniforms.distortion_cx;
    let dcy = render_uniforms.distortion_cy;
    let sq = render_uniforms.squeeze;

    let any_distortion = k1 != 0.0 || k2 != 0.0 || k3 != 0.0 || k4 != 0.0
        || rp1 != 0.0 || rp2 != 0.0 || dcx != 0.0 || dcy != 0.0 || sq != 1.0;

    if any_distortion {
        let ndc = clip.xy / clip.w;
        let dx = ndc.x - dcx;
        let dy = ndc.y - dcy;
        let sx = dx * sq;  // elliptical field
        let r2 = sx * sx + dy * dy;
        let r4 = r2 * r2;
        let radial = 1.0 + k1 * r2 + k2 * r4 + k3 * r2 * r4 + k4 * r4 * r4;
        let tang_x = 2.0 * rp1 * sx * dy + rp2 * (r2 + 2.0 * sx * sx);
        let tang_y = rp1 * (r2 + 2.0 * dy * dy) + 2.0 * rp2 * sx * dy;
        let out_x = dx * radial + tang_x + dcx;
        let out_y = dy * radial + tang_y + dcy;
        clip = vec4<f32>(out_x * clip.w, out_y * clip.w, clip.z, clip.w);
    }

    var out: VertexOutput;
    out.position = clip;
    out.det_j = v.det_j;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Caustic coloring: blend toward highlight as |det_j| approaches 0
    if render_uniforms.caustic_on != 0u {
        let threshold = render_uniforms.caustic_threshold;
        let t = 1.0 - clamp(abs(in.det_j) / threshold, 0.0, 1.0);
        let caustic_color = vec4<f32>(1.0, 0.4, 0.1, 1.0);  // warm orange glow
        return mix(render_uniforms.line_color, caustic_color, t * t);
    }
    return render_uniforms.line_color;
}
