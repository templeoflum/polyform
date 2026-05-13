// Polyform render shader — instanced beam quads with Gaussian cross-section
//
// Each instance = one line segment. Each segment draws 6 vertices (2 triangles)
// forming a screen-aligned quad that extends ±3σ from the line center.
// The fragment shader evaluates the Gaussian beam profile perpendicular to the line.

struct TransformedVertex {
    vx: f32,
    vy: f32,
    vz: f32,
    det_j: f32,
    curl: f32,
    conformal_defect: f32,
    nx: f32,
    ny: f32,
    nz: f32,
    tx: f32,
    ty: f32,
    _pad: f32,
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
    beam_width: f32,
    beam_intensity: f32,
    gain: f32,
    blend_mix: f32,
    verts_per_line: u32,
    viewport_width: f32,
    viewport_height: f32,
    color_mode: u32,
    normal_shading_on: u32,
}

@group(0) @binding(0) var<storage, read> vertices: array<TransformedVertex>;
@group(0) @binding(1) var<uniform> ru: RenderUniforms;

// Quad vertices: 2 triangles forming a quad from (-1,-1) to (1,1)
const QUAD_VERTS = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0),
    vec2(-1.0, 1.0),  vec2(1.0, -1.0), vec2(1.0, 1.0),
);

const PI: f32 = 3.14159265358979323846;

// Brown-Conrady lens distortion applied in clip space
fn apply_distortion(clip: vec4<f32>) -> vec4<f32> {
    let k1 = ru.k1;
    let k2 = ru.k2;
    let k3 = ru.k3;
    let k4 = ru.k4;
    let rp1 = ru.p1;
    let rp2 = ru.p2;
    let dcx = ru.distortion_cx;
    let dcy = ru.distortion_cy;
    let sq = ru.squeeze;

    let any_distortion = k1 != 0.0 || k2 != 0.0 || k3 != 0.0 || k4 != 0.0
        || rp1 != 0.0 || rp2 != 0.0 || dcx != 0.0 || dcy != 0.0 || sq != 1.0;

    if !any_distortion {
        return clip;
    }

    let ndc = clip.xy / clip.w;
    let dx = ndc.x - dcx;
    let dy = ndc.y - dcy;
    let sx = dx * sq;
    let r2 = sx * sx + dy * dy;
    let r4 = r2 * r2;
    let radial = 1.0 + k1 * r2 + k2 * r4 + k3 * r2 * r4 + k4 * r4 * r4;
    let tang_x = 2.0 * rp1 * sx * dy + rp2 * (r2 + 2.0 * sx * sx);
    let tang_y = rp1 * (r2 + 2.0 * dy * dy) + 2.0 * rp2 * sx * dy;
    let out_x = dx * radial + tang_x + dcx;
    let out_y = dy * radial + tang_y + dcy;
    return vec4<f32>(out_x * clip.w, out_y * clip.w, clip.z, clip.w);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) perp_dist: f32,       // perpendicular distance in pixels (for Gaussian)
    @location(1) det_j: f32,           // Jacobian determinant (for caustic coloring)
    @location(2) seg_length_px: f32,   // segment length in pixels (for dwell time)
    @location(3) post_ops_x: f32,      // post-ops x (for domain coloring)
    @location(4) post_ops_y: f32,      // post-ops y (for domain coloring)
    @location(5) curl: f32,            // curl value
    @location(6) conformal_defect: f32, // conformal defect value
    @location(7) @interpolate(linear) normal_x: f32,   // surface normal x
    @location(8) @interpolate(linear) normal_y: f32,   // surface normal y
    @location(9) @interpolate(linear) normal_z: f32,   // surface normal z
}

@vertex
fn vs_main(
    @builtin(vertex_index) vid: u32,
    @builtin(instance_index) segment_id: u32,
) -> VertexOutput {
    let samples = ru.verts_per_line - 1u;  // 80
    let line_idx = segment_id / samples;
    let seg_in_line = segment_id % samples;
    let v0_idx = line_idx * ru.verts_per_line + seg_in_line;
    let v1_idx = v0_idx + 1u;

    let v0 = vertices[v0_idx];
    let v1 = vertices[v1_idx];

    // Project both endpoints through VP + distortion
    let clip0 = apply_distortion(ru.view_projection * vec4<f32>(v0.vx, v0.vy, v0.vz, 1.0));
    let clip1 = apply_distortion(ru.view_projection * vec4<f32>(v1.vx, v1.vy, v1.vz, 1.0));

    let ndc0 = clip0.xy / clip0.w;
    let ndc1 = clip1.xy / clip1.w;

    // Screen-space direction and perpendicular
    let seg_dir = ndc1 - ndc0;
    let seg_len_ndc = length(seg_dir);
    let dir = select(normalize(seg_dir), vec2(1.0, 0.0), seg_len_ndc < 1e-8);
    let perp = vec2(-dir.y, dir.x);

    // Beam width: convert σ from pixels to NDC
    let sigma = ru.beam_width;
    let px_to_ndc_x = 2.0 / ru.viewport_width;
    let px_to_ndc_y = 2.0 / ru.viewport_height;
    // Use average for perpendicular expansion (isotropic on non-square viewports)
    let px_to_ndc_avg = (px_to_ndc_x + px_to_ndc_y) * 0.5;
    let half_width_ndc = sigma * 3.0 * px_to_ndc_avg;

    // Segment length in pixels (average of x and y)
    let seg_px_x = seg_dir.x * 0.5 * ru.viewport_width;
    let seg_px_y = seg_dir.y * 0.5 * ru.viewport_height;
    let seg_length_px = length(vec2(seg_px_x, seg_px_y));

    // Expand quad
    let corner = QUAD_VERTS[vid];
    let along = (corner.x + 1.0) * 0.5;  // 0 = start, 1 = end
    let across = corner.y;                // -1 to +1

    // Interpolate clip position along segment
    let base_ndc = mix(ndc0, ndc1, along);
    let base_w = mix(clip0.w, clip1.w, along);
    let base_z = mix(clip0.z, clip1.z, along);

    // Add perpendicular offset for beam width
    let offset = perp * across * half_width_ndc;
    let final_ndc = base_ndc + offset;

    // Perpendicular distance in pixels (for Gaussian evaluation)
    let perp_dist_px = across * sigma * 3.0;

    var out: VertexOutput;
    out.position = vec4<f32>(final_ndc * base_w, base_z, base_w);
    out.perp_dist = perp_dist_px;
    out.det_j = mix(v0.det_j, v1.det_j, along);
    out.seg_length_px = seg_length_px;
    out.post_ops_x = mix(v0.tx, v1.tx, along);
    out.post_ops_y = mix(v0.ty, v1.ty, along);
    out.curl = mix(v0.curl, v1.curl, along);
    out.conformal_defect = mix(v0.conformal_defect, v1.conformal_defect, along);
    out.normal_x = mix(v0.nx, v1.nx, along);
    out.normal_y = mix(v0.ny, v1.ny, along);
    out.normal_z = mix(v0.nz, v1.nz, along);
    return out;
}

// =============================================================================
// Color functions
// =============================================================================

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let hp = h * 6.0;
    let x = c * (1.0 - abs(hp % 2.0 - 1.0));
    let m = v - c;
    var rgb: vec3<f32>;
    if hp < 1.0 { rgb = vec3(c, x, 0.0); }
    else if hp < 2.0 { rgb = vec3(x, c, 0.0); }
    else if hp < 3.0 { rgb = vec3(0.0, c, x); }
    else if hp < 4.0 { rgb = vec3(0.0, x, c); }
    else if hp < 5.0 { rgb = vec3(x, 0.0, c); }
    else { rgb = vec3(c, 0.0, x); }
    return rgb + vec3(m);
}

// Domain coloring: phase(z) → hue, |z| → brightness
fn domain_color(tx_val: f32, ty_val: f32) -> vec3<f32> {
    let phase = atan2(ty_val, tx_val);           // [-pi, pi]
    let hue = (phase / (2.0 * PI) + 1.0) % 1.0;  // [0, 1]
    let modulus = length(vec2(tx_val, ty_val));
    let brightness = 1.0 - exp(-modulus * 0.5);   // soft compress
    return hsv_to_rgb(hue, 0.85, brightness);
}

// Curl colormap: diverging blue → white → red
fn curl_colormap(curl_val: f32) -> vec3<f32> {
    let t = clamp(curl_val * 2.0, -1.0, 1.0);  // normalize range
    if t < 0.0 {
        return mix(vec3(1.0), vec3(0.2, 0.3, 1.0), -t);  // white → blue
    } else {
        return mix(vec3(1.0), vec3(1.0, 0.2, 0.2), t);    // white → red
    }
}

// Conformal defect colormap: sequential hot (black → red → yellow → white)
fn conformal_colormap(defect: f32) -> vec3<f32> {
    let t = clamp(defect * 1.5, 0.0, 1.0);
    if t < 0.333 {
        let s = t / 0.333;
        return mix(vec3(0.0), vec3(1.0, 0.0, 0.0), s);    // black → red
    } else if t < 0.666 {
        let s = (t - 0.333) / 0.333;
        return mix(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), s); // red → yellow
    } else {
        let s = (t - 0.666) / 0.334;
        return mix(vec3(1.0, 1.0, 0.0), vec3(1.0), s);    // yellow → white
    }
}

// Lambertian shading from fixed light direction
fn lambertian(normal: vec3<f32>) -> f32 {
    let light_dir = normalize(vec3(0.5, 1.0, 0.3));
    return max(dot(normal, light_dir), 0.05);  // 0.05 ambient floor
}

// =============================================================================
// Fragment shader
// =============================================================================

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let sigma = ru.beam_width;

    // Gaussian beam cross-section
    let normalized = in.perp_dist / max(sigma, 0.01);
    let gaussian = exp(-0.5 * normalized * normalized);

    // Dwell-time brightness: compressed regions → bright, stretched → dim
    let beam_mod = ru.beam_intensity;
    let dwell = mix(1.0, 1.0 / max(in.seg_length_px, 0.1), beam_mod);

    // Combined intensity
    let intensity = gaussian * dwell * ru.gain;

    // Base color selection by mode
    var base_color: vec3<f32>;
    switch ru.color_mode {
        case 1u: { base_color = domain_color(in.post_ops_x, in.post_ops_y); }
        case 2u: { base_color = curl_colormap(in.curl); }
        case 3u: { base_color = conformal_colormap(in.conformal_defect); }
        default: { base_color = ru.line_color.rgb; }
    }

    // Caustic overlay (independent, applies on top of any color mode)
    if ru.caustic_on != 0u {
        let threshold = ru.caustic_threshold;
        let t = 1.0 - clamp(abs(in.det_j) / threshold, 0.0, 1.0);
        let caustic_color = vec3<f32>(1.0, 0.4, 0.1);
        base_color = mix(base_color, caustic_color, t * t);
    }

    // Normal shading (intensity modulator)
    var shade = 1.0;
    if ru.normal_shading_on != 0u {
        let normal = vec3<f32>(in.normal_x, in.normal_y, in.normal_z);
        shade = lambertian(normal);
    }

    let rgb = base_color * intensity * shade;

    // Premultiplied alpha trick for blend_mix interpolation:
    // blend_mix=1 → alpha=0 → dst *= 1 → pure additive (phosphor emission)
    // blend_mix=0 → alpha=intensity → standard alpha blend (opaque lines)
    let alpha = intensity * (1.0 - ru.blend_mix);
    return vec4<f32>(rgb, alpha);
}
