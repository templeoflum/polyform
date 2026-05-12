use std::collections::HashMap;
use std::f64::consts::PI;

use crate::ops::{self, OperationFn};
use crate::lift::{self, LiftFn};

// =============================================================================
// ParamDef — metadata for a single parameter in a layer type
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ParamDef {
    pub key: &'static str,
    pub default: f64,
    pub min: f64,
    pub max: f64,
    pub periodic: bool,
}

// =============================================================================
// OperationDef — metadata + function for an R2->R2 operation
// =============================================================================

pub struct OperationDef {
    pub key: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub params: Vec<ParamDef>,
    pub needs_origin: bool,
    pub apply: OperationFn,
}

impl std::fmt::Debug for OperationDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationDef")
            .field("key", &self.key)
            .field("label", &self.label)
            .field("category", &self.category)
            .field("params", &self.params)
            .field("needs_origin", &self.needs_origin)
            .finish()
    }
}

impl Clone for OperationDef {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            label: self.label,
            category: self.category,
            params: self.params.clone(),
            needs_origin: self.needs_origin,
            apply: self.apply,
        }
    }
}

// =============================================================================
// LiftDef — metadata + function for an R3 displacement
// =============================================================================

pub struct LiftDef {
    pub key: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub params: Vec<ParamDef>,
    pub needs_jacobian: bool,
    pub apply: LiftFn,
}

impl std::fmt::Debug for LiftDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiftDef")
            .field("key", &self.key)
            .field("label", &self.label)
            .field("category", &self.category)
            .field("params", &self.params)
            .field("needs_jacobian", &self.needs_jacobian)
            .finish()
    }
}

impl Clone for LiftDef {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            label: self.label,
            category: self.category,
            params: self.params.clone(),
            needs_jacobian: self.needs_jacobian,
            apply: self.apply,
        }
    }
}

// =============================================================================
// Registry — all operation and lift type metadata + functions
// =============================================================================

#[derive(Debug, Clone)]
pub struct Registry {
    pub operations: HashMap<&'static str, OperationDef>,
    pub lift: HashMap<&'static str, LiftDef>,
}

impl Registry {
    /// Build the full registry with all operation and lift type metadata.
    pub fn new() -> Self {
        Self {
            operations: build_operations(),
            lift: build_lift(),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Operations (25 types)
// =============================================================================

fn build_operations() -> HashMap<&'static str, OperationDef> {
    let mut m = HashMap::new();

    // ----- linear / affine -----
    m.insert("translate", OperationDef {
        key: "translate",
        label: "translate",
        category: "linear",
        params: vec![
            ParamDef { key: "tx", default: 0.1, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "ty", default: 0.0, min: -1.0, max: 1.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::translate,
    });
    m.insert("scale", OperationDef {
        key: "scale",
        label: "scale",
        category: "linear",
        params: vec![
            ParamDef { key: "sx", default: 1.2, min: 0.1, max: 3.0, periodic: false },
            ParamDef { key: "sy", default: 1.0, min: 0.1, max: 3.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::scale,
    });
    m.insert("rotate", OperationDef {
        key: "rotate",
        label: "rotate",
        category: "linear",
        params: vec![
            ParamDef { key: "theta", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_origin: false,
        apply: ops::rotate,
    });
    m.insert("shear", OperationDef {
        key: "shear",
        label: "shear",
        category: "linear",
        params: vec![
            ParamDef { key: "kx", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "ky", default: 0.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::shear,
    });
    m.insert("matrix", OperationDef {
        key: "matrix",
        label: "2x2 matrix",
        category: "linear",
        params: vec![
            ParamDef { key: "a", default: 1.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "b", default: 0.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "c", default: 0.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "d", default: 1.0, min: -3.0, max: 3.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::matrix,
    });

    // ----- complex / conformal -----
    m.insert("zPow", OperationDef {
        key: "zPow",
        label: "z^n",
        category: "complex",
        params: vec![
            ParamDef { key: "n", default: 2.0, min: 0.1, max: 5.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::z_pow,
    });
    m.insert("zInv", OperationDef {
        key: "zInv",
        label: "1/z",
        category: "complex",
        params: vec![],
        needs_origin: false,
        apply: ops::z_inv,
    });
    m.insert("zExp", OperationDef {
        key: "zExp",
        label: "exp(z)",
        category: "complex",
        params: vec![],
        needs_origin: false,
        apply: ops::z_exp,
    });
    m.insert("zLog", OperationDef {
        key: "zLog",
        label: "log(z)",
        category: "complex",
        params: vec![],
        needs_origin: false,
        apply: ops::z_log,
    });
    m.insert("zSin", OperationDef {
        key: "zSin",
        label: "sin(z)",
        category: "complex",
        params: vec![],
        needs_origin: false,
        apply: ops::z_sin,
    });
    m.insert("zCos", OperationDef {
        key: "zCos",
        label: "cos(z)",
        category: "complex",
        params: vec![],
        needs_origin: false,
        apply: ops::z_cos,
    });
    m.insert("mobius", OperationDef {
        key: "mobius",
        label: "mobius (az+b)/(cz+d)",
        category: "complex",
        params: vec![
            ParamDef { key: "a", default: 1.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "b", default: 0.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "c", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "d", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::mobius,
    });
    m.insert("zPlusC", OperationDef {
        key: "zPlusC",
        label: "z + c",
        category: "complex",
        params: vec![
            ParamDef { key: "cx", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "cy", default: 0.2, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::z_plus_c,
    });

    // ----- polar / radial -----
    m.insert("spiral", OperationDef {
        key: "spiral",
        label: "spiral",
        category: "polar",
        params: vec![
            ParamDef { key: "k", default: 1.0, min: -5.0, max: 5.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::spiral,
    });
    m.insert("pinch", OperationDef {
        key: "pinch",
        label: "pinch / bulge",
        category: "polar",
        params: vec![
            ParamDef { key: "p", default: 1.5, min: 0.2, max: 4.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::pinch,
    });
    m.insert("swirl", OperationDef {
        key: "swirl",
        label: "swirl",
        category: "polar",
        params: vec![
            ParamDef { key: "k", default: 2.0, min: -8.0, max: 8.0, periodic: false },
            ParamDef { key: "sigma", default: 0.4, min: 0.05, max: 2.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::swirl,
    });
    m.insert("radialSine", OperationDef {
        key: "radialSine",
        label: "radial sine",
        category: "polar",
        params: vec![
            ParamDef { key: "amp", default: 0.1, min: 0.0, max: 0.5, periodic: false },
            ParamDef { key: "freq", default: 5.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_origin: false,
        apply: ops::radial_sine,
    });
    m.insert("angleWarp", OperationDef {
        key: "angleWarp",
        label: "angle warp",
        category: "polar",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: 0.0, max: 2.0, periodic: false },
            ParamDef { key: "k", default: 3.0, min: 1.0, max: 12.0, periodic: false },
        ],
        needs_origin: false,
        apply: ops::angle_warp,
    });

    // ----- wave / periodic -----
    m.insert("sineWarp", OperationDef {
        key: "sineWarp",
        label: "sine warp",
        category: "wave",
        params: vec![
            ParamDef { key: "amp", default: 0.15, min: 0.0, max: 0.6, periodic: false },
            ParamDef { key: "freq", default: 4.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_origin: false,
        apply: ops::sine_warp,
    });
    m.insert("ripple", OperationDef {
        key: "ripple",
        label: "ripple",
        category: "wave",
        params: vec![
            ParamDef { key: "amp", default: 0.1, min: 0.0, max: 0.5, periodic: false },
            ParamDef { key: "freq", default: 6.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_origin: false,
        apply: ops::ripple,
    });

    // ----- non-holomorphic -----
    m.insert("conjugate", OperationDef {
        key: "conjugate",
        label: "conjugate",
        category: "non-holo",
        params: vec![],
        needs_origin: false,
        apply: ops::conjugate,
    });
    m.insert("reflect", OperationDef {
        key: "reflect",
        label: "reflect",
        category: "non-holo",
        params: vec![
            ParamDef { key: "theta", default: 0.0, min: 0.0, max: PI, periodic: true },
        ],
        needs_origin: false,
        apply: ops::reflect,
    });
    m.insert("abs", OperationDef {
        key: "abs",
        label: "|z|",
        category: "non-holo",
        params: vec![],
        needs_origin: false,
        apply: ops::abs_op,
    });
    m.insert("mixed", OperationDef {
        key: "mixed",
        label: "mixed monomials",
        category: "non-holo",
        params: vec![],
        needs_origin: false,
        apply: ops::mixed,
    });

    // ----- utility -----
    m.insert("blend", OperationDef {
        key: "blend",
        label: "blend (utility)",
        category: "utility",
        params: vec![
            ParamDef { key: "amount", default: 0.5, min: 0.0, max: 1.0, periodic: false },
        ],
        needs_origin: true,
        apply: ops::blend_op,
    });

    m
}

// =============================================================================
// Lift / displacement types (15 types)
// =============================================================================

fn build_lift() -> HashMap<&'static str, LiftDef> {
    let mut m = HashMap::new();

    // ----- height-style (z only) -----
    m.insert("zMagnitude", LiftDef {
        key: "zMagnitude",
        label: "|.| -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_magnitude,
    });
    m.insert("zDisplacement", LiftDef {
        key: "zDisplacement",
        label: "|T|-|z| -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_displacement,
    });
    m.insert("zLogJac", LiftDef {
        key: "zLogJac",
        label: "log|det J| -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: true,
        apply: lift::z_log_jac,
    });
    m.insert("zSignedJac", LiftDef {
        key: "zSignedJac",
        label: "det J -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: true,
        apply: lift::z_signed_jac,
    });
    m.insert("zReal", LiftDef {
        key: "zReal",
        label: "Re(T) -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_real,
    });
    m.insert("zImag", LiftDef {
        key: "zImag",
        label: "Im(T) -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_imag,
    });
    m.insert("zArg", LiftDef {
        key: "zArg",
        label: "arg(T) -> z",
        category: "height",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_arg,
    });

    // ----- waves / bumps (z) -----
    m.insert("zSineWave", LiftDef {
        key: "zSineWave",
        label: "sine wave (z)",
        category: "wave",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "freq", default: 4.0, min: 0.1, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_jacobian: false,
        apply: lift::z_sine_wave,
    });
    m.insert("zSineXY", LiftDef {
        key: "zSineXY",
        label: "sine xy (z)",
        category: "wave",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "fx", default: 3.0, min: 0.1, max: 15.0, periodic: false },
            ParamDef { key: "fy", default: 3.0, min: 0.1, max: 15.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_sine_xy,
    });
    m.insert("zGaussian", LiftDef {
        key: "zGaussian",
        label: "gaussian bump (z)",
        category: "wave",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "sigma", default: 0.5, min: 0.05, max: 3.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::z_gaussian,
    });

    // ----- 3D displacement (full vector) -----
    m.insert("spherical", LiftDef {
        key: "spherical",
        label: "spherical projection",
        category: "3d",
        params: vec![
            ParamDef { key: "r", default: 1.0, min: 0.1, max: 3.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::spherical,
    });
    m.insert("conical", LiftDef {
        key: "conical",
        label: "conical lift",
        category: "3d",
        params: vec![
            ParamDef { key: "amp", default: 0.8, min: -2.0, max: 2.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::conical,
    });
    m.insert("helical", LiftDef {
        key: "helical",
        label: "helical twist",
        category: "3d",
        params: vec![
            ParamDef { key: "lift", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "twist", default: 2.0, min: -8.0, max: 8.0, periodic: false },
        ],
        needs_jacobian: false,
        apply: lift::helical,
    });
    m.insert("noise3D", LiftDef {
        key: "noise3D",
        label: "noise displacement",
        category: "3d",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "freq", default: 3.0, min: 0.5, max: 15.0, periodic: false },
            ParamDef { key: "seed", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_jacobian: false,
        apply: lift::noise_3d,
    });
    m.insert("ripple3D", LiftDef {
        key: "ripple3D",
        label: "ripple (z + xy)",
        category: "3d",
        params: vec![
            ParamDef { key: "amp", default: 0.2, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "freq", default: 6.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
        needs_jacobian: false,
        apply: lift::ripple_3d,
    });

    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_operation_count() {
        let reg = Registry::new();
        assert_eq!(reg.operations.len(), 25);
    }

    #[test]
    fn registry_lift_count() {
        let reg = Registry::new();
        assert_eq!(reg.lift.len(), 15);
    }

    #[test]
    fn all_operation_keys_match() {
        let expected = vec![
            "translate", "scale", "rotate", "shear", "matrix",
            "zPow", "zInv", "zExp", "zLog", "zSin", "zCos", "mobius", "zPlusC",
            "spiral", "pinch", "swirl", "radialSine", "angleWarp",
            "sineWarp", "ripple",
            "conjugate", "reflect", "abs", "mixed",
            "blend",
        ];
        let reg = Registry::new();
        for key in &expected {
            assert!(reg.operations.contains_key(key), "missing operation: {key}");
        }
    }

    #[test]
    fn all_lift_keys_match() {
        let expected = vec![
            "zMagnitude", "zDisplacement", "zLogJac", "zSignedJac",
            "zReal", "zImag", "zArg",
            "zSineWave", "zSineXY", "zGaussian",
            "spherical", "conical", "helical", "noise3D", "ripple3D",
        ];
        let reg = Registry::new();
        for key in &expected {
            assert!(reg.lift.contains_key(key), "missing lift: {key}");
        }
    }

    #[test]
    fn operation_functions_callable() {
        let reg = Registry::new();
        let params = HashMap::new();
        // Call each operation to ensure no panic
        for (_key, def) in &reg.operations {
            let _ = (def.apply)(1.0, 1.0, &params, 0.0, 0.0);
        }
    }

    #[test]
    fn lift_functions_callable() {
        let reg = Registry::new();
        let params = HashMap::new();
        // Call each lift to ensure no panic
        for (_key, def) in &reg.lift {
            let _ = (def.apply)(1.0, 1.0, &params, 0.0, 0.0, 1.0);
        }
    }

    #[test]
    fn operation_labels_nonempty() {
        let reg = Registry::new();
        for (key, def) in &reg.operations {
            assert!(!def.label.is_empty(), "empty label for operation: {key}");
        }
    }

    #[test]
    fn lift_labels_nonempty() {
        let reg = Registry::new();
        for (key, def) in &reg.lift {
            assert!(!def.label.is_empty(), "empty label for lift: {key}");
        }
    }

    #[test]
    fn only_blend_needs_origin() {
        let reg = Registry::new();
        for (key, def) in &reg.operations {
            if *key == "blend" {
                assert!(def.needs_origin);
            } else {
                assert!(!def.needs_origin, "{key} should not need origin");
            }
        }
    }

    #[test]
    fn only_jacobian_lifts_need_jacobian() {
        let reg = Registry::new();
        for (key, def) in &reg.lift {
            if *key == "zLogJac" || *key == "zSignedJac" {
                assert!(def.needs_jacobian, "{key} should need jacobian");
            } else {
                assert!(!def.needs_jacobian, "{key} should not need jacobian");
            }
        }
    }
}
