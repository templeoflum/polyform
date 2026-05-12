use std::f64::consts::TAU;

use crate::types::{Param, ShapeFunction};

// =============================================================================
// Types
// =============================================================================

/// Identifies which pipeline a layer belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineKind {
    Operations,
    Lift,
}

/// Structured path to a specific animatable param.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamPath {
    LayerParam {
        kind: PipelineKind,
        layer_id: String,
        key: String,
    },
    LayerBlend {
        kind: PipelineKind,
        layer_id: String,
    },
    View(String),
    Tempo,
}

// =============================================================================
// Helpers
// =============================================================================

/// Returns 1.0 when ratio is 0, NaN, or infinity. Otherwise returns ratio.
pub fn safe_ratio(r: f64) -> f64 {
    if r == 0.0 || r.is_nan() || r.is_infinite() {
        1.0
    } else {
        r
    }
}

// =============================================================================
// Shape function
// =============================================================================

/// Map a phase in [0, 1] to a value in [0, 1] according to the shape.
pub fn apply_shape(shape: ShapeFunction, phase: f64) -> f64 {
    match shape {
        ShapeFunction::Sawtooth => phase,
        ShapeFunction::Cosine => 0.5 - 0.5 * (phase * TAU).cos(),
    }
}

// =============================================================================
// Core resolve function
// =============================================================================

/// Pure function: resolve a param value from master phase and anim weight.
///
/// - If `anim_weight <= 0` or `!param.anim`, returns `param.value`.
/// - Computes carrier phase from master_phase, ratio, offset.
/// - Applies optional FM modulator.
/// - Maps through shape function and scales to [min, max].
/// - Blends between base value and animated value using weight.
pub fn resolve_param(param: &Param, master_phase: f64, anim_weight: f64) -> f64 {
    if anim_weight <= 0.0 || !param.anim {
        return param.value;
    }

    // Carrier phase
    let r = safe_ratio(param.ratio);
    let mut phase = ((master_phase * r + param.offset) % 1.0 + 1.0) % 1.0;

    // FM modulation
    if let Some(ref m) = param.modulator {
        if m.depth.abs() > 1e-12 {
            let mod_phase = ((master_phase * safe_ratio(m.ratio)) % 1.0 + 1.0) % 1.0;
            phase = ((phase + m.depth * apply_shape(m.shape, mod_phase)) % 1.0 + 1.0) % 1.0;
        }
    }

    let t = apply_shape(param.shape, phase);
    let anim_value = param.min + t * (param.max - param.min);

    // Weighted blend
    if anim_weight >= 1.0 {
        anim_value
    } else {
        (1.0 - anim_weight) * param.value + anim_weight * anim_value
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Param, PhaseModulator, ShapeFunction};

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    fn make_cosine_param(min: f64, max: f64, ratio: f64) -> Param {
        Param {
            value: min,
            min,
            max,
            anim: true,
            ratio,
            offset: 0.0,
            shape: ShapeFunction::Cosine,
            modulator: None,
        }
    }

    fn make_sawtooth_param(value: f64, min: f64, max: f64, ratio: f64) -> Param {
        Param {
            value,
            min,
            max,
            anim: true,
            ratio,
            offset: 0.0,
            shape: ShapeFunction::Sawtooth,
            modulator: None,
        }
    }

    // ---- apply_shape tests ----

    #[test]
    fn shape_cosine_at_zero() {
        // cos(0) = 1, t = 0.5 - 0.5*1 = 0
        assert_near(apply_shape(ShapeFunction::Cosine, 0.0), 0.0, 1e-10);
    }

    #[test]
    fn shape_cosine_at_half() {
        // cos(PI) = -1, t = 0.5 - 0.5*(-1) = 1.0
        assert_near(apply_shape(ShapeFunction::Cosine, 0.5), 1.0, 1e-10);
    }

    #[test]
    fn shape_cosine_at_quarter() {
        // cos(PI/2) = 0, t = 0.5
        assert_near(apply_shape(ShapeFunction::Cosine, 0.25), 0.5, 1e-10);
    }

    #[test]
    fn shape_sawtooth_identity() {
        assert_near(apply_shape(ShapeFunction::Sawtooth, 0.0), 0.0, 1e-10);
        assert_near(apply_shape(ShapeFunction::Sawtooth, 0.5), 0.5, 1e-10);
        assert_near(apply_shape(ShapeFunction::Sawtooth, 0.75), 0.75, 1e-10);
    }

    // ---- resolve_param tests (cosine shape) ----

    #[test]
    fn resolve_at_phase_zero_returns_min() {
        let p = make_cosine_param(0.0, 1.0, 1.0);
        // master_phase=0, phase=0, cosine shape -> t=0 -> value=min=0
        let v = resolve_param(&p, 0.0, 1.0);
        assert_near(v, 0.0, 1e-10);
    }

    #[test]
    fn resolve_at_phase_half_returns_max() {
        let p = make_cosine_param(0.0, 1.0, 1.0);
        // master_phase=0.5, ratio=1, phase=0.5, cosine -> t=1 -> value=max=1
        let v = resolve_param(&p, 0.5, 1.0);
        assert_near(v, 1.0, 1e-10);
    }

    #[test]
    fn resolve_at_phase_quarter_returns_midpoint() {
        let p = make_cosine_param(0.0, 1.0, 1.0);
        // master_phase=0.25, ratio=1, phase=0.25, cosine -> t=0.5
        let v = resolve_param(&p, 0.25, 1.0);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- resolve_param tests (sawtooth shape) ----

    #[test]
    fn sawtooth_linear_ramp() {
        let p = make_sawtooth_param(0.0, 0.0, 1.0, 1.0);
        // master_phase=0.3 -> phase=0.3 -> t=0.3 -> value=0.3
        let v = resolve_param(&p, 0.3, 1.0);
        assert_near(v, 0.3, 1e-10);
    }

    #[test]
    fn sawtooth_wraps_at_one() {
        let p = make_sawtooth_param(0.0, 0.0, 1.0, 1.0);
        // master_phase=1.5 -> phase = (1.5 % 1 + 1) % 1 = 0.5
        let v = resolve_param(&p, 1.5, 1.0);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- anim=false ----

    #[test]
    fn anim_false_returns_value() {
        let p = Param {
            value: 0.42,
            min: 0.0,
            max: 1.0,
            anim: false,
            ratio: 1.0,
            offset: 0.0,
            shape: ShapeFunction::Cosine,
            modulator: None,
        };
        let v = resolve_param(&p, 10.0, 1.0);
        assert_near(v, 0.42, 1e-15);
    }

    // ---- weight=0 returns base value ----

    #[test]
    fn weight_zero_returns_value() {
        let p = make_cosine_param(0.0, 1.0, 1.0);
        let v = resolve_param(&p, 0.5, 0.0);
        assert_near(v, 0.0, 1e-15); // p.value = 0 (same as min)
    }

    // ---- partial weight blends ----

    #[test]
    fn partial_weight_blends() {
        let mut p = make_cosine_param(0.0, 1.0, 1.0);
        p.value = 0.2; // base value
        // At master_phase=0.5, full anim value = 1.0
        // weight=0.5 -> 0.5*0.2 + 0.5*1.0 = 0.6
        let v = resolve_param(&p, 0.5, 0.5);
        assert_near(v, 0.6, 1e-10);
    }

    // ---- ratio ----

    #[test]
    fn ratio_zero_treated_as_one() {
        assert_near(safe_ratio(0.0), 1.0, 1e-15);
        assert_near(safe_ratio(f64::NAN), 1.0, 1e-15);
        assert_near(safe_ratio(2.5), 2.5, 1e-15);
    }

    #[test]
    fn ratio_scales_phase() {
        let p = make_cosine_param(0.0, 1.0, 2.0);
        // master_phase=0.25, ratio=2 -> phase=0.5, cosine -> t=1 -> value=max=1
        let v = resolve_param(&p, 0.25, 1.0);
        assert_near(v, 1.0, 1e-10);
    }

    // ---- offset ----

    #[test]
    fn offset_shifts_phase() {
        let mut p = make_cosine_param(0.0, 1.0, 1.0);
        p.offset = 0.25;
        // master_phase=0, phase = (0*1 + 0.25) % 1 = 0.25, cosine -> t=0.5
        let v = resolve_param(&p, 0.0, 1.0);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- FM modulator ----

    #[test]
    fn modulator_shifts_phase() {
        let mut p = make_cosine_param(0.0, 1.0, 1.0);
        p.modulator = Some(PhaseModulator {
            ratio: 1.0,
            depth: 0.25,
            shape: ShapeFunction::Cosine,
        });
        // At master_phase=0:
        //   carrier phase = 0
        //   mod_phase = 0, apply_shape(cosine, 0) = 0
        //   phase = (0 + 0.25 * 0) % 1 = 0
        //   t = cosine(0) = 0 -> value = 0
        let v = resolve_param(&p, 0.0, 1.0);
        assert_near(v, 0.0, 1e-10);
    }

    // ---- safe_ratio edge cases ----

    #[test]
    fn safe_ratio_infinity() {
        assert_near(safe_ratio(f64::INFINITY), 1.0, 1e-15);
        assert_near(safe_ratio(f64::NEG_INFINITY), 1.0, 1e-15);
    }

    // ---- determinism ----

    #[test]
    fn deterministic_same_inputs_same_output() {
        let p = make_cosine_param(0.0, 1.0, 1.0);
        let v1 = resolve_param(&p, 0.37, 1.0);
        let v2 = resolve_param(&p, 0.37, 1.0);
        assert_near(v1, v2, 1e-15);
    }
}
