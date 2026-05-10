use std::f64::consts::{PI, TAU};

use crate::types::{AnimMode, Param};

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
    /// Phase offset sub-param for a LayerParam.
    LayerParamPhaseOffset {
        kind: PipelineKind,
        layer_id: String,
        key: String,
    },
    View(String),
    Tempo,
}

/// Persistent state for a loop-mode param.
#[derive(Debug, Clone, Default)]
pub struct LoopState {
    pub base: f64,
    /// Separate accumulator for phase-offset in loop mode.
    pub phase_offset_base: Option<f64>,
}

/// State for resync blending when animation resumes after interruption.
#[derive(Debug, Clone)]
pub struct ResyncState {
    pub value: f64,
    pub clock: f64,
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
// Core sweep function
// =============================================================================

/// Pure function that computes the next animated value for a param.
///
/// Ports the JS `sweepParam` exactly:
/// 1. If `!param.anim` -> return `param.value`
/// 2. Compute phase offset (if present and animated)
/// 3. Compute main value (loop or sweep mode)
/// 4. Apply resync blend if present
///
/// # Arguments
/// - `param`: The param to sweep
/// - `anim_clock`: Monotonic animation time (seconds)
/// - `anim_dt`: Time delta for this frame (seconds)
/// - `base_rate`: Tempo-scaled base rate
/// - `loop_state`: Mutable loop accumulator (updated in place for loop mode)
/// - `resync`: Optional resync state for smooth blend-back
///
/// # Returns
/// The swept value for this frame.
pub fn sweep_param(
    param: &Param,
    anim_clock: f64,
    anim_dt: f64,
    base_rate: f64,
    loop_state: &mut LoopState,
    resync: Option<&ResyncState>,
) -> f64 {
    if !param.anim {
        return param.value;
    }

    // --- Phase offset ---
    let po_val = compute_phase_offset(param, anim_clock, anim_dt, base_rate, loop_state);

    // --- Main value ---
    let speed = base_rate * safe_ratio(param.ratio);
    let natural_value = match param.anim_mode {
        AnimMode::Loop => {
            let span = param.max - param.min;
            if span <= 0.0 {
                return param.value;
            }
            let base = &mut loop_state.base;
            *base += anim_dt * speed * span;
            *base = ((*base - param.min) % span + span) % span + param.min;
            let shifted = (*base - param.min + po_val * span) % span;
            (shifted + span) % span + param.min
        }
        AnimMode::Sweep => {
            let phase = anim_clock * speed + po_val;
            let t = 0.5 - 0.5 * (phase * TAU).cos();
            param.min + t * (param.max - param.min)
        }
    };

    // --- Resync blend ---
    if let Some(rs) = resync {
        let elapsed = anim_clock - rs.clock;
        if elapsed >= 2.0 {
            // Resync expired — return natural value
            natural_value
        } else {
            let rt = 0.5 - 0.5 * ((elapsed / 2.0) * PI).cos();
            rs.value + (natural_value - rs.value) * rt
        }
    } else {
        natural_value
    }
}

/// Compute the phase offset contribution for a param.
fn compute_phase_offset(
    param: &Param,
    anim_clock: f64,
    anim_dt: f64,
    base_rate: f64,
    loop_state: &mut LoopState,
) -> f64 {
    let po = match &param.phase_offset {
        Some(po) => po.as_ref(),
        None => return 0.0,
    };

    if !po.anim {
        return po.value;
    }

    let po_speed = base_rate * safe_ratio(po.ratio);

    match po.anim_mode {
        AnimMode::Loop => {
            let po_span = po.max - po.min;
            if po_span <= 0.0 {
                return po.value;
            }
            let po_base = loop_state
                .phase_offset_base
                .get_or_insert(po.value);
            *po_base += anim_dt * po_speed * po_span;
            *po_base = ((*po_base - po.min) % po_span + po_span) % po_span + po.min;
            *po_base
        }
        AnimMode::Sweep => {
            let po_phase = anim_clock * po_speed;
            let pot = 0.5 - 0.5 * (po_phase * TAU).cos();
            po.min + pot * (po.max - po.min)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Param;

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    fn make_sweep_param(min: f64, max: f64, ratio: f64) -> Param {
        Param {
            value: min,
            min,
            max,
            anim: true,
            anim_mode: AnimMode::Sweep,
            ratio,
            phase_offset: None,
        }
    }

    fn make_loop_param(value: f64, min: f64, max: f64, ratio: f64) -> Param {
        Param {
            value,
            min,
            max,
            anim: true,
            anim_mode: AnimMode::Loop,
            ratio,
            phase_offset: None,
        }
    }

    // ---- Sweep mode tests ----

    #[test]
    fn sweep_at_t0_returns_min() {
        let p = make_sweep_param(0.0, 1.0, 1.0);
        let mut ls = LoopState::default();
        // At clock=0, phase=0, cos(0)=1, t = 0.5-0.5*1 = 0 -> value = min
        let v = sweep_param(&p, 0.0, 0.0, 1.0, &mut ls, None);
        assert_near(v, 0.0, 1e-10);
    }

    #[test]
    fn sweep_at_half_period_returns_max() {
        let p = make_sweep_param(0.0, 1.0, 1.0);
        let mut ls = LoopState::default();
        // At clock=0.5, speed=1, phase=0.5, cos(0.5*TAU)=cos(PI)=-1, t=0.5-0.5*(-1)=1 -> value=max
        let v = sweep_param(&p, 0.5, 0.0, 1.0, &mut ls, None);
        assert_near(v, 1.0, 1e-10);
    }

    #[test]
    fn sweep_at_quarter_returns_midpoint() {
        let p = make_sweep_param(0.0, 1.0, 1.0);
        let mut ls = LoopState::default();
        // At clock=0.25, phase=0.25, cos(0.25*TAU)=cos(PI/2)=0, t=0.5 -> midpoint
        let v = sweep_param(&p, 0.25, 0.0, 1.0, &mut ls, None);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- Loop mode tests ----

    #[test]
    fn loop_wraps_correctly() {
        // span=1, speed=1*1*1=1, dt=1.0 means base += 1.0*1.0*1.0 = 1.0
        // If start value is 0.5, after dt=0.6 base goes 0.5+0.6=1.1, wraps to 0.1
        let p = make_loop_param(0.5, 0.0, 1.0, 1.0);
        let mut ls = LoopState { base: 0.5, phase_offset_base: None };
        let v = sweep_param(&p, 1.0, 0.6, 1.0, &mut ls, None);
        assert_near(v, 0.1, 1e-10);
    }

    #[test]
    fn loop_accumulates_across_frames() {
        let p = make_loop_param(0.0, 0.0, 1.0, 1.0);
        let mut ls = LoopState { base: 0.0, phase_offset_base: None };

        // Frame 1: dt=0.3, base 0.0+0.3=0.3
        let v1 = sweep_param(&p, 0.3, 0.3, 1.0, &mut ls, None);
        assert_near(v1, 0.3, 1e-10);

        // Frame 2: dt=0.4, base 0.3+0.4=0.7
        let v2 = sweep_param(&p, 0.7, 0.4, 1.0, &mut ls, None);
        assert_near(v2, 0.7, 1e-10);

        // Frame 3: dt=0.5, base 0.7+0.5=1.2, wraps to 0.2
        let v3 = sweep_param(&p, 1.2, 0.5, 1.0, &mut ls, None);
        assert_near(v3, 0.2, 1e-10);
    }

    // ---- anim=false ----

    #[test]
    fn anim_false_returns_value() {
        let p = Param {
            value: 0.42,
            min: 0.0,
            max: 1.0,
            anim: false,
            anim_mode: AnimMode::Sweep,
            ratio: 1.0,
            phase_offset: None,
        };
        let mut ls = LoopState::default();
        let v = sweep_param(&p, 10.0, 0.1, 1.0, &mut ls, None);
        assert_near(v, 0.42, 1e-15);
    }

    // ---- ratio=0 treated as 1 ----

    #[test]
    fn ratio_zero_treated_as_one() {
        assert_near(safe_ratio(0.0), 1.0, 1e-15);
        assert_near(safe_ratio(f64::NAN), 1.0, 1e-15);
        assert_near(safe_ratio(2.5), 2.5, 1e-15);
    }

    #[test]
    fn sweep_with_ratio_zero_uses_one() {
        let p = make_sweep_param(0.0, 1.0, 0.0); // ratio=0
        let mut ls = LoopState::default();
        // Should behave identically to ratio=1
        let v = sweep_param(&p, 0.25, 0.0, 1.0, &mut ls, None);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- Phase offset ----

    #[test]
    fn phase_offset_shifts_sweep() {
        // Main param: sweep [0,1], ratio=1
        // Phase offset: static value=0.25 (shifts phase by 0.25)
        let p = Param {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            anim: true,
            anim_mode: AnimMode::Sweep,
            ratio: 1.0,
            phase_offset: Some(Box::new(Param {
                value: 0.25,
                min: 0.0,
                max: 1.0,
                anim: false,
                anim_mode: AnimMode::Sweep,
                ratio: 1.0,
                phase_offset: None,
            })),
        };
        let mut ls = LoopState::default();
        // At clock=0, phase = 0*1 + 0.25 = 0.25
        // t = 0.5 - 0.5*cos(0.25*TAU) = 0.5 - 0.5*cos(PI/2) = 0.5
        let v = sweep_param(&p, 0.0, 0.0, 1.0, &mut ls, None);
        assert_near(v, 0.5, 1e-10);
    }

    // ---- Resync blend ----

    #[test]
    fn resync_blends_at_midpoint() {
        let p = make_sweep_param(0.0, 1.0, 1.0);
        let mut ls = LoopState::default();

        // Resync from value=0.8 at clock=0.0
        let rs = ResyncState { value: 0.8, clock: 0.0 };

        // At clock=1.0 (halfway through 2s resync), rt = 0.5 - 0.5*cos(0.5*PI) = 0.5
        // natural_value at clock=1.0: phase=1.0, cos(TAU)=1, t=0 -> natural=0.0
        // blended = 0.8 + (0.0 - 0.8) * 0.5 = 0.4
        let v = sweep_param(&p, 1.0, 0.0, 1.0, &mut ls, Some(&rs));
        assert_near(v, 0.4, 1e-10);
    }

    #[test]
    fn resync_expires_after_two_seconds() {
        let p = make_sweep_param(0.0, 1.0, 1.0);
        let mut ls = LoopState::default();

        let rs = ResyncState { value: 0.8, clock: 0.0 };

        // At clock=2.5, elapsed=2.5 >= 2.0, resync expired -> natural value
        let v_with_resync = sweep_param(&p, 2.5, 0.0, 1.0, &mut ls, Some(&rs));
        let mut ls2 = LoopState::default();
        let v_without_resync = sweep_param(&p, 2.5, 0.0, 1.0, &mut ls2, None);
        assert_near(v_with_resync, v_without_resync, 1e-10);
    }

    // ---- safe_ratio edge cases ----

    #[test]
    fn safe_ratio_infinity() {
        assert_near(safe_ratio(f64::INFINITY), 1.0, 1e-15);
        assert_near(safe_ratio(f64::NEG_INFINITY), 1.0, 1e-15);
    }
}
