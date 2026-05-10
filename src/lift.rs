use std::collections::HashMap;

/// Uniform lift signature: (x, y, params, x0, y0, j_signed) -> (dx, dy, dz)
/// All lifts receive j_signed even if unused (only zLogJac and zSignedJac use it).
pub type LiftFn = fn(f64, f64, &HashMap<String, f64>, f64, f64, f64) -> (f64, f64, f64);

// =============================================================================
// Height-style (z only) — 7
// =============================================================================

pub fn z_magnitude(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(1.0);
    (0.0, 0.0, amp * x.hypot(y))
}

pub fn z_displacement(x: f64, y: f64, p: &HashMap<String, f64>, x0: f64, y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(1.0);
    (0.0, 0.0, amp * (x.hypot(y) - x0.hypot(y0)))
}

pub fn z_log_jac(_x: f64, _y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, j_signed: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.5);
    (0.0, 0.0, amp * (j_signed.abs() + 1e-6).ln())
}

pub fn z_signed_jac(_x: f64, _y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, j_signed: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.3);
    (0.0, 0.0, amp * j_signed)
}

pub fn z_real(x: f64, _y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(1.0);
    (0.0, 0.0, amp * x)
}

pub fn z_imag(_x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(1.0);
    (0.0, 0.0, amp * y)
}

pub fn z_arg(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.3);
    (0.0, 0.0, amp * y.atan2(x))
}

// =============================================================================
// Wave / bumps (z) — 3
// =============================================================================

pub fn z_sine_wave(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.5);
    let freq = p.get("freq").copied().unwrap_or(4.0);
    let phase = p.get("phase").copied().unwrap_or(0.0);
    (0.0, 0.0, amp * (freq * x.hypot(y) + phase).sin())
}

pub fn z_sine_xy(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.5);
    let fx = p.get("fx").copied().unwrap_or(3.0);
    let fy = p.get("fy").copied().unwrap_or(3.0);
    (0.0, 0.0, amp * (fx * x).sin() * (fy * y).sin())
}

pub fn z_gaussian(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(1.0);
    let sigma = p.get("sigma").copied().unwrap_or(0.5);
    (0.0, 0.0, amp * (-(x * x + y * y) / (sigma * sigma)).exp())
}

// =============================================================================
// 3D displacement (full vector) — 5
// =============================================================================

pub fn spherical(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let r_param = p.get("r").copied().unwrap_or(1.0);
    let r = x.hypot(y);
    let t = y.atan2(x);
    let phi = (r / r_param).atan();
    let sx = r_param * phi.sin() * t.cos();
    let sy = r_param * phi.sin() * t.sin();
    let sz = r_param * phi.cos();
    (sx - x, sy - y, sz - r_param)
}

pub fn conical(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.8);
    (0.0, 0.0, amp * x.hypot(y))
}

pub fn helical(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let lift = p.get("lift").copied().unwrap_or(0.5);
    let twist = p.get("twist").copied().unwrap_or(2.0);
    let z = lift * x.hypot(y);
    let c = (twist * z).cos();
    let s = (twist * z).sin();
    (x * c - y * s - x, x * s + y * c - y, z)
}

pub fn noise_3d(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.3);
    let freq = p.get("freq").copied().unwrap_or(3.0);
    let seed = p.get("seed").copied().unwrap_or(0.0);
    let nx = (freq * x + 1.3 * seed).sin() * (freq * y * 1.7 + 2.1).cos();
    let ny = (freq * y + 2.7 * seed).sin() * (freq * x * 1.3 + 0.7).cos();
    let nz = (freq * (x + y) + 0.5 * seed).sin() * (freq * (x - y) * 1.1).cos();
    (amp * nx, amp * ny, amp * nz)
}

pub fn ripple_3d(x: f64, y: f64, p: &HashMap<String, f64>, _x0: f64, _y0: f64, _j: f64) -> (f64, f64, f64) {
    let amp = p.get("amp").copied().unwrap_or(0.2);
    let freq = p.get("freq").copied().unwrap_or(6.0);
    let phase = p.get("phase").copied().unwrap_or(0.0);
    let r = x.hypot(y) + 1e-6;
    let t = y.atan2(x);
    let w = (freq * r + phase).sin();
    (amp * w * t.cos() * 0.3, amp * w * t.sin() * 0.3, amp * w)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn params(entries: &[(&str, f64)]) -> HashMap<String, f64> {
        entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    #[test]
    fn test_z_magnitude() {
        let (dx, dy, dz) = z_magnitude(3.0, 4.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dx, 0.0, 1e-10);
        assert_near(dy, 0.0, 1e-10);
        assert_near(dz, 5.0, 1e-10);
    }

    #[test]
    fn test_z_gaussian_origin() {
        let (dx, dy, dz) = z_gaussian(0.0, 0.0, &params(&[("amp", 1.0), ("sigma", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dx, 0.0, 1e-10);
        assert_near(dy, 0.0, 1e-10);
        assert_near(dz, 1.0, 1e-10);
    }

    #[test]
    fn test_z_displacement_identity() {
        let (_, _, dz) = z_displacement(1.0, 0.0, &params(&[("amp", 1.0)]), 1.0, 0.0, 0.0);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_z_real() {
        let (_, _, dz) = z_real(2.0, 3.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 2.0, 1e-10);
    }

    #[test]
    fn test_z_imag() {
        let (_, _, dz) = z_imag(2.0, 3.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 3.0, 1e-10);
    }

    #[test]
    fn test_z_arg_at_one_zero() {
        let (_, _, dz) = z_arg(1.0, 0.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_z_log_jac_det_one() {
        let (_, _, dz) = z_log_jac(0.0, 0.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 1.0);
        // log(1 + 1e-6) ~ 1e-6
        assert_near(dz, 0.0, 1e-4);
    }

    #[test]
    fn test_z_signed_jac() {
        let (_, _, dz) = z_signed_jac(0.0, 0.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 4.0);
        assert_near(dz, 4.0, 1e-10);
    }

    #[test]
    fn test_z_sine_wave_origin() {
        let (_, _, dz) = z_sine_wave(0.0, 0.0, &params(&[("amp", 1.0), ("freq", 4.0), ("phase", 0.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_z_sine_xy_origin() {
        let (_, _, dz) = z_sine_xy(0.0, 0.0, &params(&[("amp", 1.0), ("fx", 3.0), ("fy", 3.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_conical_origin() {
        let (_, _, dz) = conical(0.0, 0.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_conical_3_4() {
        let (_, _, dz) = conical(3.0, 4.0, &params(&[("amp", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dz, 5.0, 1e-10);
    }

    #[test]
    fn test_helical_origin() {
        let (dx, dy, dz) = helical(0.0, 0.0, &params(&[("lift", 0.5), ("twist", 2.0)]), 0.0, 0.0, 0.0);
        assert_near(dx, 0.0, 1e-10);
        assert_near(dy, 0.0, 1e-10);
        assert_near(dz, 0.0, 1e-10);
    }

    #[test]
    fn test_spherical_origin() {
        let (dx, dy, dz) = spherical(0.0, 0.0, &params(&[("r", 1.0)]), 0.0, 0.0, 0.0);
        assert_near(dx, 0.0, 1e-10);
        assert_near(dy, 0.0, 1e-10);
        assert_near(dz, 0.0, 1e-10);
    }
}
