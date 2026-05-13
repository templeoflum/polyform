use std::collections::HashMap;

/// Uniform operation signature: (x, y, params, x0, y0) -> (out_x, out_y)
/// All operations receive x0/y0 even if unused (only `blend` uses them).
pub type OperationFn = fn(f64, f64, &HashMap<String, f64>, f64, f64) -> (f64, f64);

// =============================================================================
// Linear / Affine (5)
// =============================================================================

pub fn translate(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let tx = p.get("tx").copied().unwrap_or(0.0);
    let ty = p.get("ty").copied().unwrap_or(0.0);
    (x + tx, y + ty)
}

pub fn scale(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let sx = p.get("sx").copied().unwrap_or(1.0);
    let sy = p.get("sy").copied().unwrap_or(1.0);
    (x * sx, y * sy)
}

pub fn rotate(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let theta = p.get("theta").copied().unwrap_or(0.0);
    let c = theta.cos();
    let s = theta.sin();
    (x * c - y * s, x * s + y * c)
}

pub fn shear(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let kx = p.get("kx").copied().unwrap_or(0.0);
    let ky = p.get("ky").copied().unwrap_or(0.0);
    (x + kx * y, y + ky * x)
}

pub fn matrix(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let a = p.get("a").copied().unwrap_or(1.0);
    let b = p.get("b").copied().unwrap_or(0.0);
    let c = p.get("c").copied().unwrap_or(0.0);
    let d = p.get("d").copied().unwrap_or(1.0);
    (a * x + b * y, c * x + d * y)
}

// =============================================================================
// Complex / Conformal (8)
// =============================================================================

pub fn z_pow(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let n = p.get("n").copied().unwrap_or(2.0);
    let r = x.hypot(y);
    let t = y.atan2(x);
    let rn = r.powf(n);
    (rn * (n * t).cos(), rn * (n * t).sin())
}

pub fn z_inv(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let d = x * x + y * y + 1e-6;
    (x / d, -y / d)
}

pub fn z_exp(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let ex = x.exp();
    (ex * y.cos(), ex * y.sin())
}

pub fn z_log(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let r = x.hypot(y) + 1e-6;
    (r.ln(), y.atan2(x))
}

pub fn z_sin(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    (x.sin() * y.cosh(), x.cos() * y.sinh())
}

pub fn z_cos(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    (x.cos() * y.cosh(), -x.sin() * y.sinh())
}

pub fn mobius(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let a = p.get("a").copied().unwrap_or(1.0);
    let b = p.get("b").copied().unwrap_or(0.0);
    let c = p.get("c").copied().unwrap_or(0.0);
    let d = p.get("d").copied().unwrap_or(1.0);
    // (a*z + b) / (c*z + d) where z = x + iy, b and d are real
    let nx = a * x + b;
    let ny = a * y;
    let dx = c * x + d;
    let dy = c * y;
    let den = dx * dx + dy * dy + 1e-6;
    ((nx * dx + ny * dy) / den, (ny * dx - nx * dy) / den)
}

pub fn z_plus_c(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let cx = p.get("cx").copied().unwrap_or(0.0);
    let cy = p.get("cy").copied().unwrap_or(0.0);
    (x + cx, y + cy)
}

pub fn joukowski(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let a = p.get("a").copied().unwrap_or(0.5);
    let r2 = x * x + y * y;
    // Smooth fade: joukowski effect fades to identity below r=0.3
    let fade = (r2 / (0.3 * 0.3)).min(1.0);
    let inv = if r2 > 1e-12 { fade * a / r2 } else { 0.0 };
    (x + inv * x, y - inv * y)
}

pub fn z_tan(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    // tan(z) = sin(z)/cos(z) where z = x + iy
    let sin_re = x.sin() * y.cosh();
    let sin_im = x.cos() * y.sinh();
    let cos_re = x.cos() * y.cosh();
    let cos_im = -x.sin() * y.sinh();
    // Complex division: sin(z) / cos(z)
    let den = cos_re * cos_re + cos_im * cos_im;
    if den < 1e-6 {
        // Near singularity — clamp
        let r = (sin_re * sin_re + sin_im * sin_im).sqrt();
        let max_r = 10.0;
        if r > max_r {
            return (sin_re / r * max_r, sin_im / r * max_r);
        }
        return (sin_re, sin_im);
    }
    let re = (sin_re * cos_re + sin_im * cos_im) / den;
    let im = (sin_im * cos_re - sin_re * cos_im) / den;
    // Magnitude clamp
    let mag = re.hypot(im);
    let max_r = 10.0;
    if mag > max_r {
        (re / mag * max_r, im / mag * max_r)
    } else {
        (re, im)
    }
}

// =============================================================================
// Polar / Radial (6)
// =============================================================================

pub fn spiral(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let k = p.get("k").copied().unwrap_or(1.0);
    let r = x.hypot(y) + 1e-6;
    let t = y.atan2(x) + k * r.ln();
    (r * t.cos(), r * t.sin())
}

pub fn pinch(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let pp = p.get("p").copied().unwrap_or(1.0);
    let r = x.hypot(y);
    if r < 1e-6 {
        return (x, y);
    }
    let t = y.atan2(x);
    let nr = r.powf(pp);
    (nr * t.cos(), nr * t.sin())
}

pub fn swirl(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let k = p.get("k").copied().unwrap_or(2.0);
    let sigma = p.get("sigma").copied().unwrap_or(0.4);
    let r = x.hypot(y);
    let t0 = y.atan2(x);
    let t = t0 + k * (-(r * r) / (sigma * sigma)).exp();
    (r * t.cos(), r * t.sin())
}

pub fn radial_sine(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.1);
    let freq = p.get("freq").copied().unwrap_or(5.0);
    let phase = p.get("phase").copied().unwrap_or(0.0);
    let r = x.hypot(y);
    if r < 1e-6 {
        return (x, y);
    }
    let t = y.atan2(x);
    let nr = r + amp * (freq * r + phase).sin();
    (nr * t.cos(), nr * t.sin())
}

pub fn kaleidoscope(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let n = p.get("n").copied().unwrap_or(6.0);
    let rotation = p.get("rotation").copied().unwrap_or(0.0);
    let r = x.hypot(y);
    if r < 1e-10 {
        return (x, y);
    }
    let sector = std::f64::consts::TAU / n;
    let mut theta = y.atan2(x) - rotation;
    // Normalize to [0, TAU)
    theta = ((theta % std::f64::consts::TAU) + std::f64::consts::TAU) % std::f64::consts::TAU;
    // Fold into first sector
    theta = theta % sector;
    // Mirror within sector
    if theta > sector / 2.0 {
        theta = sector - theta;
    }
    theta += rotation;
    (r * theta.cos(), r * theta.sin())
}

pub fn angle_warp(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.3);
    let k = p.get("k").copied().unwrap_or(3.0);
    let r = x.hypot(y);
    let t0 = y.atan2(x);
    let t = t0 + amp * (k * t0).sin();
    (r * t.cos(), r * t.sin())
}

// =============================================================================
// Wave / Periodic (2)
// =============================================================================

pub fn sine_warp(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.15);
    let freq = p.get("freq").copied().unwrap_or(4.0);
    let phase = p.get("phase").copied().unwrap_or(0.0);
    (
        x + amp * (freq * y + phase).sin(),
        y + amp * (freq * x + phase).sin(),
    )
}

pub fn ripple(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.1);
    let freq = p.get("freq").copied().unwrap_or(6.0);
    let phase = p.get("phase").copied().unwrap_or(0.0);
    let r = x.hypot(y);
    let t = y.atan2(x);
    let d = amp * (freq * r + phase).sin();
    ((r + d) * t.cos(), (r + d) * t.sin())
}

pub fn noise(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.1);
    let freq = p.get("freq").copied().unwrap_or(3.0);
    let seed = p.get("seed").copied().unwrap_or(0.0);
    let nx = (freq * x + 1.3 * seed).sin() * (freq * y * 1.7 + 2.1).cos();
    let ny = (freq * y + 2.7 * seed).sin() * (freq * x * 1.3 + 0.7).cos();
    (x + amp * nx, y + amp * ny)
}

// =============================================================================
// Non-Holomorphic (4)
// =============================================================================

pub fn conjugate(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    (x, -y)
}

pub fn reflect(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let theta = p.get("theta").copied().unwrap_or(0.0);
    let c = (2.0 * theta).cos();
    let s = (2.0 * theta).sin();
    (x * c + y * s, x * s - y * c)
}

pub fn abs_op(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    (x.abs(), y.abs())
}

pub fn mixed(x: f64, y: f64, _p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    (x * x * y * y * y, -x * x * x * y * y)
}

// =============================================================================
// Utility (4)
// =============================================================================

pub fn wrap(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let w = p.get("width").copied().unwrap_or(2.0);
    let h = p.get("height").copied().unwrap_or(2.0);
    let xo = ((x % w) + 1.5 * w) % w - w / 2.0;
    let yo = ((y % h) + 1.5 * h) % h - h / 2.0;
    (xo, yo)
}

pub fn quantize(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let sx = p.get("step_x").copied().unwrap_or(0.25);
    let sy = p.get("step_y").copied().unwrap_or(0.25);
    ((x / sx).round() * sx, (y / sy).round() * sy)
}

pub fn clamp_op(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64) -> (f64, f64) {
    let radius = p.get("radius").copied().unwrap_or(2.0);
    let r = x.hypot(y);
    if r > radius && r > 1e-10 {
        (x / r * radius, y / r * radius)
    } else {
        (x, y)
    }
}

pub fn blend_op(x: f64, y: f64, p: &HashMap<String, f64>, x0: f64, y0: f64) -> (f64, f64) {
    let amount = p.get("amount").copied().unwrap_or(0.5);
    (
        x0 * (1.0 - amount) + x * amount,
        y0 * (1.0 - amount) + y * amount,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn params(entries: &[(&str, f64)]) -> HashMap<String, f64> {
        entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn empty() -> HashMap<String, f64> {
        HashMap::new()
    }

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    #[test]
    fn test_rotate_pi_half() {
        let (x, y) = rotate(1.0, 0.0, &params(&[("theta", PI / 2.0)]), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 1.0, 1e-10);
    }

    #[test]
    fn test_scale() {
        let (x, y) = scale(1.0, 1.0, &params(&[("sx", 2.0), ("sy", 3.0)]), 0.0, 0.0);
        assert_near(x, 2.0, 1e-10);
        assert_near(y, 3.0, 1e-10);
    }

    #[test]
    fn test_z_pow_i_squared() {
        // i^2 = -1
        let (x, y) = z_pow(0.0, 1.0, &params(&[("n", 2.0)]), 0.0, 0.0);
        assert_near(x, -1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_z_exp_origin() {
        let (x, y) = z_exp(0.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_conjugate() {
        let (x, y) = conjugate(1.0, 2.0, &empty(), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, -2.0, 1e-10);
    }

    #[test]
    fn test_translate() {
        let (x, y) = translate(1.0, 2.0, &params(&[("tx", 0.5), ("ty", -0.3)]), 0.0, 0.0);
        assert_near(x, 1.5, 1e-10);
        assert_near(y, 1.7, 1e-10);
    }

    #[test]
    fn test_shear_x_only() {
        let (x, y) = shear(1.0, 0.0, &params(&[("kx", 1.0), ("ky", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_shear_y_input() {
        let (x, y) = shear(0.0, 1.0, &params(&[("kx", 1.0), ("ky", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 1.0, 1e-10);
    }

    #[test]
    fn test_matrix_identity() {
        let (x, y) = matrix(
            3.0,
            7.0,
            &params(&[("a", 1.0), ("b", 0.0), ("c", 0.0), ("d", 1.0)]),
            0.0,
            0.0,
        );
        assert_near(x, 3.0, 1e-10);
        assert_near(y, 7.0, 1e-10);
    }

    #[test]
    fn test_z_inv_at_one() {
        let (x, y) = z_inv(1.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 1.0, 1e-4);
        assert_near(y, 0.0, 1e-4);
    }

    #[test]
    fn test_z_log_at_one() {
        let (x, y) = z_log(1.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 0.0, 1e-4);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_z_sin_origin() {
        let (x, y) = z_sin(0.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_z_cos_origin() {
        let (x, y) = z_cos(0.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_mobius_identity() {
        let (x, y) = mobius(
            2.0,
            3.0,
            &params(&[("a", 1.0), ("b", 0.0), ("c", 0.0), ("d", 1.0)]),
            0.0,
            0.0,
        );
        assert_near(x, 2.0, 1e-4);
        assert_near(y, 3.0, 1e-4);
    }

    #[test]
    fn test_z_plus_c() {
        let (x, y) = z_plus_c(1.0, 2.0, &params(&[("cx", 0.5), ("cy", -1.0)]), 0.0, 0.0);
        assert_near(x, 1.5, 1e-10);
        assert_near(y, 1.0, 1e-10);
    }

    #[test]
    fn test_spiral_k_zero() {
        let (x, y) = spiral(1.0, 0.0, &params(&[("k", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-4);
        assert_near(y, 0.0, 1e-4);
    }

    #[test]
    fn test_pinch_identity() {
        let (x, y) = pinch(1.0, 0.0, &params(&[("p", 1.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_swirl_origin() {
        let (x, y) = swirl(0.0, 0.0, &params(&[("k", 2.0), ("sigma", 0.4)]), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_radial_sine_origin() {
        let (x, y) = radial_sine(
            0.0,
            0.0,
            &params(&[("amp", 0.1), ("freq", 5.0), ("phase", 0.0)]),
            0.0,
            0.0,
        );
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_angle_warp_zero_amp() {
        let (x, y) = angle_warp(1.0, 0.0, &params(&[("amp", 0.0), ("k", 3.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_sine_warp_origin() {
        let (x, y) = sine_warp(
            0.0,
            0.0,
            &params(&[("amp", 0.15), ("freq", 4.0), ("phase", 0.0)]),
            0.0,
            0.0,
        );
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_ripple_origin() {
        let (x, y) = ripple(
            0.0,
            0.0,
            &params(&[("amp", 0.1), ("freq", 6.0), ("phase", 0.0)]),
            0.0,
            0.0,
        );
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_reflect_x_axis() {
        let (x, y) = reflect(1.0, 0.0, &params(&[("theta", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_reflect_y_axis() {
        let (x, y) = reflect(0.0, 1.0, &params(&[("theta", 0.0)]), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, -1.0, 1e-10);
    }

    #[test]
    fn test_abs_op() {
        let (x, y) = abs_op(-3.0, -4.0, &empty(), 0.0, 0.0);
        assert_near(x, 3.0, 1e-10);
        assert_near(y, 4.0, 1e-10);
    }

    #[test]
    fn test_mixed() {
        let (x, y) = mixed(1.0, 1.0, &empty(), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, -1.0, 1e-10);
    }

    #[test]
    fn test_blend_op() {
        let (x, y) = blend_op(2.0, 3.0, &params(&[("amount", 0.5)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 1.5, 1e-10);
    }

    #[test]
    fn test_wrap_center() {
        // Point at origin should stay at origin
        let (x, y) = wrap(0.0, 0.0, &params(&[("width", 2.0), ("height", 2.0)]), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_wrap_periodic() {
        // Point at (3.0, 0.0) with width=2 should wrap to (1.0, 0.0)
        let (x, y) = wrap(3.0, 0.0, &params(&[("width", 2.0), ("height", 2.0)]), 0.0, 0.0);
        assert_near(x, -1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_quantize() {
        let (x, y) = quantize(0.3, 0.7, &params(&[("step_x", 0.25), ("step_y", 0.5)]), 0.0, 0.0);
        assert_near(x, 0.25, 1e-10);
        assert_near(y, 0.5, 1e-10);
    }

    #[test]
    fn test_clamp_inside() {
        let (x, y) = clamp_op(1.0, 0.0, &params(&[("radius", 2.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_clamp_outside() {
        let (x, y) = clamp_op(3.0, 4.0, &params(&[("radius", 2.0)]), 0.0, 0.0);
        let r = x.hypot(y);
        assert_near(r, 2.0, 1e-10);
    }

    #[test]
    fn test_joukowski_real_axis() {
        // z + a*x/|z|^2 for z=2+0i, a=0.5 => 2 + 0.5*2/4 = 2.25
        let (x, y) = joukowski(2.0, 0.0, &params(&[("a", 0.5)]), 0.0, 0.0);
        assert_near(x, 2.25, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_z_tan_origin() {
        // tan(0) = 0
        let (x, y) = z_tan(0.0, 0.0, &empty(), 0.0, 0.0);
        assert_near(x, 0.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_kaleidoscope_symmetry() {
        // Point on x-axis should be unchanged (it's in the first sector already)
        let (x, y) = kaleidoscope(1.0, 0.0, &params(&[("n", 6.0), ("rotation", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 0.0, 1e-10);
    }

    #[test]
    fn test_noise_zero_amp() {
        let (x, y) = noise(1.0, 2.0, &params(&[("amp", 0.0), ("freq", 3.0), ("seed", 0.0)]), 0.0, 0.0);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 2.0, 1e-10);
    }
}
