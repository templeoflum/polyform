// Polyform GPU compute shader — operations + lift pipeline
// All 25 operations and 15 lifts dispatched via switch statements.

struct GridVertex {
    x0: f32,
    y0: f32,
    line_idx: u32,
    _pad: u32,
}

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

struct OpDescriptor {
    type_index: u32,
    blend: f32,
    needs_origin: u32,
    num_params: u32,
    params: vec4<f32>,  // max 4 params per op, packed in registry order
    center: vec4<f32>,  // xy = center offset, zw = padding
}

struct LiftDescriptor {
    type_index: u32,
    blend: f32,
    needs_jacobian: u32,
    num_params: u32,
    params: vec4<f32>,  // max 4 params per lift, padded
}

struct PipelineUniforms {
    num_ops: u32,
    num_lifts: u32,
    any_needs_jacobian: u32,
    compute_differential: u32,
}

@group(0) @binding(0) var<storage, read> grid_input: array<GridVertex>;
@group(0) @binding(1) var<storage, read_write> grid_output: array<TransformedVertex>;
@group(0) @binding(2) var<uniform> uniforms: PipelineUniforms;
@group(0) @binding(3) var<storage, read> op_chain: array<OpDescriptor>;
@group(0) @binding(4) var<storage, read> lift_chain: array<LiftDescriptor>;

const PI: f32 = 3.14159265358979323846;
const TAU: f32 = 6.28318530717958647692;
const EPS: f32 = 0.001;

// =============================================================================
// Operations (R2 -> R2) — 25 types
// =============================================================================

fn op_translate(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x + p.x, y + p.y);
}

fn op_scale(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x * p.x, y * p.y);
}

fn op_rotate(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let c = cos(p.x);
    let s = sin(p.x);
    return vec2<f32>(x * c - y * s, x * s + y * c);
}

fn op_shear(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x + p.x * y, y + p.y * x);
}

fn op_matrix(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(p.x * x + p.y * y, p.z * x + p.w * y);
}

fn op_z_pow(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let n = p.x;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let theta = atan2(y, x);
    let rn = pow(r, n);
    return vec2<f32>(rn * cos(n * theta), rn * sin(n * theta));
}

fn op_z_inv(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let r2 = x * x + y * y;
    if r2 < 1e-10 { return vec2<f32>(x, y); }
    return vec2<f32>(x / r2, -y / r2);
}

fn op_z_exp(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let ex = exp(x);
    return vec2<f32>(ex * cos(y), ex * sin(y));
}

fn op_z_log(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    return vec2<f32>(log(r), atan2(y, x));
}

fn op_z_sin(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(sin(x) * cosh(y), cos(x) * sinh(y));
}

fn op_z_cos(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(cos(x) * cosh(y), -sin(x) * sinh(y));
}

fn op_mobius(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    // (az+b)/(cz+d) where a=p.x, b=p.y, c=p.z, d=p.w
    let a = p.x; let b = p.y; let c = p.z; let d = p.w;
    // Numerator: (a+bi)(x+yi) = (ax - by) + (ay + bx)i  ... wait, a,b,c,d are real here
    // Actually: num = a*z + b, den = c*z + d (all real coefficients)
    let num_re = a * x + b;
    let num_im = a * y;
    let den_re = c * x + d;
    let den_im = c * y;
    let den_r2 = den_re * den_re + den_im * den_im;
    if den_r2 < 1e-10 { return vec2<f32>(x, y); }
    return vec2<f32>(
        (num_re * den_re + num_im * den_im) / den_r2,
        (num_im * den_re - num_re * den_im) / den_r2
    );
}

fn op_z_plus_c(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x + p.x, y + p.y);
}

fn op_spiral(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let k = p.x;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let theta = atan2(y, x) + k * log(max(r, 1e-10));
    return vec2<f32>(r * cos(theta), r * sin(theta));
}

fn op_pinch(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let pw = p.x;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let rp = pow(r, pw);
    let theta = atan2(y, x);
    return vec2<f32>(rp * cos(theta), rp * sin(theta));
}

fn op_swirl(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let k = p.x;
    let sigma = p.y;
    let r = sqrt(x * x + y * y);
    let angle = k * exp(-(r * r) / (2.0 * sigma * sigma));
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(x * c - y * s, x * s + y * c);
}

fn op_radial_sine(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let amp = p.x;
    let freq = p.y;
    let phase = p.z;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let new_r = r + amp * sin(freq * r + phase);
    let scale_f = new_r / r;
    return vec2<f32>(x * scale_f, y * scale_f);
}

fn op_angle_warp(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let amp = p.x;
    let k = p.y;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let theta = atan2(y, x);
    let new_theta = theta + amp * sin(k * theta);
    return vec2<f32>(r * cos(new_theta), r * sin(new_theta));
}

fn op_sine_warp(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let amp = p.x;
    let freq = p.y;
    let phase = p.z;
    return vec2<f32>(x + amp * sin(freq * y + phase), y + amp * sin(freq * x + phase));
}

fn op_ripple(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let amp = p.x;
    let freq = p.y;
    let phase = p.z;
    let r = sqrt(x * x + y * y);
    if r < 1e-10 { return vec2<f32>(0.0, 0.0); }
    let displacement = amp * sin(freq * r + phase);
    let nx = x / r;
    let ny = y / r;
    return vec2<f32>(x + displacement * nx, y + displacement * ny);
}

fn op_conjugate(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x, -y);
}

fn op_reflect(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    let theta = p.x;
    let c2 = cos(2.0 * theta);
    let s2 = sin(2.0 * theta);
    return vec2<f32>(x * c2 + y * s2, x * s2 - y * c2);
}

fn op_abs(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(abs(x), abs(y));
}

fn op_mixed(x: f32, y: f32, p: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(x * x - y * y, 2.0 * x * y);
}

fn op_blend_op(x: f32, y: f32, p: vec4<f32>, x0: f32, y0: f32) -> vec2<f32> {
    let amount = p.x;
    return vec2<f32>(
        x + (x0 - x) * amount,
        y + (y0 - y) * amount
    );
}

// Dispatch a single operation by type_index
fn dispatch_op(type_idx: u32, x: f32, y: f32, p: vec4<f32>, x0: f32, y0: f32) -> vec2<f32> {
    switch type_idx {
        case 0u: { return op_translate(x, y, p); }
        case 1u: { return op_scale(x, y, p); }
        case 2u: { return op_rotate(x, y, p); }
        case 3u: { return op_shear(x, y, p); }
        case 4u: { return op_matrix(x, y, p); }
        case 5u: { return op_z_pow(x, y, p); }
        case 6u: { return op_z_inv(x, y, p); }
        case 7u: { return op_z_exp(x, y, p); }
        case 8u: { return op_z_log(x, y, p); }
        case 9u: { return op_z_sin(x, y, p); }
        case 10u: { return op_z_cos(x, y, p); }
        case 11u: { return op_mobius(x, y, p); }
        case 12u: { return op_z_plus_c(x, y, p); }
        case 13u: { return op_spiral(x, y, p); }
        case 14u: { return op_pinch(x, y, p); }
        case 15u: { return op_swirl(x, y, p); }
        case 16u: { return op_radial_sine(x, y, p); }
        case 17u: { return op_angle_warp(x, y, p); }
        case 18u: { return op_sine_warp(x, y, p); }
        case 19u: { return op_ripple(x, y, p); }
        case 20u: { return op_conjugate(x, y, p); }
        case 21u: { return op_reflect(x, y, p); }
        case 22u: { return op_abs(x, y, p); }
        case 23u: { return op_mixed(x, y, p); }
        case 24u: { return op_blend_op(x, y, p, x0, y0); }
        default: { return vec2<f32>(x, y); }
    }
}

// Apply the full operations pipeline with blend per layer
fn apply_ops_pipeline(x0: f32, y0: f32) -> vec2<f32> {
    var x = x0;
    var y = y0;
    for (var i: u32 = 0u; i < uniforms.num_ops; i = i + 1u) {
        let op = op_chain[i];
        if op.blend <= 0.0 { continue; }
        // Center offset: translate input by -center, apply op, translate output by +center
        let cx = op.center.x;
        let cy = op.center.y;
        let local_x = x - cx;
        let local_y = y - cy;
        let result = dispatch_op(op.type_index, local_x, local_y, op.params, x0, y0);
        let global_x = result.x + cx;
        let global_y = result.y + cy;
        if op.blend >= 1.0 {
            x = global_x;
            y = global_y;
        } else {
            x = x + (global_x - x) * op.blend;
            y = y + (global_y - y) * op.blend;
        }
    }
    return vec2<f32>(x, y);
}

// =============================================================================
// Extended Jacobian — returns (det_j, curl, conformal_defect)
// =============================================================================

fn extended_jacobian(x0: f32, y0: f32) -> vec3<f32> {
    let center = apply_ops_pipeline(x0, y0);
    let dx = apply_ops_pipeline(x0 + EPS, y0);
    let dy = apply_ops_pipeline(x0, y0 + EPS);

    let dudx = (dx.x - center.x) / EPS;  // du/dx
    let dudy = (dy.x - center.x) / EPS;  // du/dy
    let dvdx = (dx.y - center.y) / EPS;  // dv/dx
    let dvdy = (dy.y - center.y) / EPS;  // dv/dy

    let det_j = dudx * dvdy - dvdx * dudy;
    let curl_val = dvdx - dudy;
    let conformal = abs(dudx - dvdy) + abs(dudy + dvdx);  // Cauchy-Riemann violation

    return vec3<f32>(det_j, curl_val, conformal);
}

// =============================================================================
// Lift (R3 displacement) — 15 types
// =============================================================================

fn lift_z_magnitude(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let mag = sqrt(tx * tx + ty * ty);
    return vec3<f32>(0.0, amp * mag, 0.0);
}

fn lift_z_displacement(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let mag_t = sqrt(tx * tx + ty * ty);
    let mag_o = sqrt(x0 * x0 + y0 * y0);
    return vec3<f32>(0.0, amp * (mag_t - mag_o), 0.0);
}

fn lift_z_log_jac(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let abs_det = abs(det_j);
    let log_det = log(max(abs_det, 1e-10));
    return vec3<f32>(0.0, amp * log_det, 0.0);
}

fn lift_z_signed_jac(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    return vec3<f32>(0.0, amp * det_j, 0.0);
}

fn lift_z_real(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    return vec3<f32>(0.0, amp * tx, 0.0);
}

fn lift_z_imag(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    return vec3<f32>(0.0, amp * ty, 0.0);
}

fn lift_z_arg(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    return vec3<f32>(0.0, amp * atan2(ty, tx), 0.0);
}

fn lift_z_sine_wave(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let freq = p.y;
    let phase = p.z;
    let r = sqrt(tx * tx + ty * ty);
    return vec3<f32>(0.0, amp * sin(freq * r + phase), 0.0);
}

fn lift_z_sine_xy(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let fx = p.y;
    let fy = p.z;
    return vec3<f32>(0.0, amp * sin(fx * tx) * sin(fy * ty), 0.0);
}

fn lift_z_gaussian(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let sigma = p.y;
    let r2 = tx * tx + ty * ty;
    return vec3<f32>(0.0, amp * exp(-r2 / (2.0 * sigma * sigma)), 0.0);
}

fn lift_spherical(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let r = p.x;
    let mag = sqrt(tx * tx + ty * ty);
    let theta = mag / r;
    let phi = atan2(ty, tx);
    let st = sin(theta);
    let dx = r * st * cos(phi) - tx;
    let dz = r * st * sin(phi) - ty;
    let dy = r * cos(theta) - r;
    return vec3<f32>(dx, dy, dz);
}

fn lift_conical(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let r = sqrt(tx * tx + ty * ty);
    return vec3<f32>(0.0, amp * r, 0.0);
}

fn lift_helical(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let lift_amt = p.x;
    let twist = p.y;
    let theta = atan2(ty, tx);
    let dy = lift_amt * theta / TAU;
    let angle = twist * theta;
    let r = sqrt(tx * tx + ty * ty);
    let dx = r * cos(angle) - tx;
    let dz = r * sin(angle) - ty;
    return vec3<f32>(dx, dy, dz);
}

fn lift_noise_3d(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let freq = p.y;
    let seed = p.z;
    // Simple pseudo-noise using sin combinations (matches JS implementation)
    let nx = sin(freq * tx + seed) * cos(freq * ty * 1.3 + seed * 0.7);
    let ny = sin(freq * tx * 0.7 + seed * 1.3) * cos(freq * ty + seed * 0.5);
    let nz = cos(freq * tx * 1.1 + seed * 0.3) * sin(freq * ty * 0.9 + seed * 1.1);
    return vec3<f32>(amp * nx, amp * ny, amp * nz);
}

fn lift_ripple_3d(tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    let amp = p.x;
    let freq = p.y;
    let phase = p.z;
    let r = sqrt(tx * tx + ty * ty);
    let wave = amp * sin(freq * r + phase);
    // Radial displacement in xy + height in z
    let nx = select(tx / r, 0.0, r < 1e-10);
    let ny_val = select(ty / r, 0.0, r < 1e-10);
    return vec3<f32>(wave * nx * 0.3, wave, wave * ny_val * 0.3);
}

// Dispatch a single lift by type_index
fn dispatch_lift(type_idx: u32, tx: f32, ty: f32, p: vec4<f32>, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    switch type_idx {
        case 0u: { return lift_z_magnitude(tx, ty, p, x0, y0, det_j); }
        case 1u: { return lift_z_displacement(tx, ty, p, x0, y0, det_j); }
        case 2u: { return lift_z_log_jac(tx, ty, p, x0, y0, det_j); }
        case 3u: { return lift_z_signed_jac(tx, ty, p, x0, y0, det_j); }
        case 4u: { return lift_z_real(tx, ty, p, x0, y0, det_j); }
        case 5u: { return lift_z_imag(tx, ty, p, x0, y0, det_j); }
        case 6u: { return lift_z_arg(tx, ty, p, x0, y0, det_j); }
        case 7u: { return lift_z_sine_wave(tx, ty, p, x0, y0, det_j); }
        case 8u: { return lift_z_sine_xy(tx, ty, p, x0, y0, det_j); }
        case 9u: { return lift_z_gaussian(tx, ty, p, x0, y0, det_j); }
        case 10u: { return lift_spherical(tx, ty, p, x0, y0, det_j); }
        case 11u: { return lift_conical(tx, ty, p, x0, y0, det_j); }
        case 12u: { return lift_helical(tx, ty, p, x0, y0, det_j); }
        case 13u: { return lift_noise_3d(tx, ty, p, x0, y0, det_j); }
        case 14u: { return lift_ripple_3d(tx, ty, p, x0, y0, det_j); }
        default: { return vec3<f32>(0.0, 0.0, 0.0); }
    }
}

// Apply the full lift pipeline to post-ops coordinates
fn apply_lift_pipeline(ops_x: f32, ops_y: f32, x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    var world_x = ops_x;
    var world_y: f32 = 0.0;
    var world_z = ops_y;

    for (var i: u32 = 0u; i < uniforms.num_lifts; i = i + 1u) {
        let lift = lift_chain[i];
        if lift.blend <= 0.0 { continue; }
        let disp = dispatch_lift(lift.type_index, ops_x, ops_y, lift.params, x0, y0, det_j);
        if lift.blend >= 1.0 {
            world_x = world_x + disp.x;
            world_y = world_y + disp.y;
            world_z = world_z + disp.z;
        } else {
            world_x = world_x + disp.x * lift.blend;
            world_y = world_y + disp.y * lift.blend;
            world_z = world_z + disp.z * lift.blend;
        }
    }

    return vec3<f32>(world_x, world_y, world_z);
}

// =============================================================================
// Normal computation via tangent cross product
// =============================================================================

fn compute_normal(x0: f32, y0: f32, det_j: f32) -> vec3<f32> {
    // Compute full pipeline (ops + lift) at center and two offset points
    let ops_center = apply_ops_pipeline(x0, y0);
    let pos_center = apply_lift_pipeline(ops_center.x, ops_center.y, x0, y0, det_j);

    let ops_dx = apply_ops_pipeline(x0 + EPS, y0);
    let pos_dx = apply_lift_pipeline(ops_dx.x, ops_dx.y, x0 + EPS, y0, det_j);

    let ops_dy = apply_ops_pipeline(x0, y0 + EPS);
    let pos_dy = apply_lift_pipeline(ops_dy.x, ops_dy.y, x0, y0 + EPS, det_j);

    let tangent_x = pos_dx - pos_center;
    let tangent_y = pos_dy - pos_center;
    let normal = cross(tangent_x, tangent_y);
    let len = length(normal);
    if len < 1e-8 {
        return vec3<f32>(0.0, 1.0, 0.0);  // degenerate -> up
    }
    return normal / len;
}

// =============================================================================
// Main compute entry point
// =============================================================================

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= arrayLength(&grid_input) { return; }

    let vertex = grid_input[idx];
    let x0 = vertex.x0;
    let y0 = vertex.y0;

    // Stage 1: Apply operations pipeline
    let transformed = apply_ops_pipeline(x0, y0);
    let ops_x = transformed.x;
    let ops_y = transformed.y;

    // Initialize output fields
    var det_j: f32 = 1.0;
    var curl_val: f32 = 0.0;
    var conformal_val: f32 = 0.0;
    var norm_x: f32 = 0.0;
    var norm_y: f32 = 1.0;
    var norm_z: f32 = 0.0;

    // Stage 2: Compute differentials (tiered gating)
    // Tier 1+: det_j needed for lift Jacobian or caustic or extended differentials
    if uniforms.compute_differential >= 1u || uniforms.any_needs_jacobian != 0u {
        let jac = extended_jacobian(x0, y0);
        det_j = jac.x;
        if uniforms.compute_differential >= 2u {
            curl_val = jac.y;
            conformal_val = jac.z;
        }
    }

    // Tier 3: surface normals (full pipeline re-evaluation at offset points)
    if uniforms.compute_differential >= 3u {
        let n = compute_normal(x0, y0, det_j);
        norm_x = n.x;
        norm_y = n.y;
        norm_z = n.z;
    }

    // Stage 3: Apply lift pipeline
    let world_pos = apply_lift_pipeline(ops_x, ops_y, x0, y0, det_j);

    grid_output[idx] = TransformedVertex(
        world_pos.x, world_pos.y, world_pos.z,
        det_j,
        curl_val, conformal_val,
        norm_x, norm_y, norm_z,
        ops_x, ops_y,
        0.0  // _pad
    );
}
