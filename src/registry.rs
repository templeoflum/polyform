use std::collections::HashMap;
use std::f64::consts::PI;

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
// LayerTypeDef — metadata for a layer type (operation or lift)
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct LayerTypeDef {
    pub key: &'static str,
    pub params: Vec<ParamDef>,
}

// =============================================================================
// Registry — all operation and lift type metadata
// =============================================================================

#[derive(Debug, Clone)]
pub struct Registry {
    pub operations: HashMap<&'static str, LayerTypeDef>,
    pub lift: HashMap<&'static str, LayerTypeDef>,
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

fn build_operations() -> HashMap<&'static str, LayerTypeDef> {
    let mut m = HashMap::new();

    // ----- linear / affine -----
    m.insert("translate", LayerTypeDef {
        key: "translate",
        params: vec![
            ParamDef { key: "tx", default: 0.1, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "ty", default: 0.0, min: -1.0, max: 1.0, periodic: false },
        ],
    });
    m.insert("scale", LayerTypeDef {
        key: "scale",
        params: vec![
            ParamDef { key: "sx", default: 1.2, min: 0.1, max: 3.0, periodic: false },
            ParamDef { key: "sy", default: 1.0, min: 0.1, max: 3.0, periodic: false },
        ],
    });
    m.insert("rotate", LayerTypeDef {
        key: "rotate",
        params: vec![
            ParamDef { key: "theta", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });
    m.insert("shear", LayerTypeDef {
        key: "shear",
        params: vec![
            ParamDef { key: "kx", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "ky", default: 0.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("matrix", LayerTypeDef {
        key: "matrix",
        params: vec![
            ParamDef { key: "a", default: 1.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "b", default: 0.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "c", default: 0.0, min: -3.0, max: 3.0, periodic: false },
            ParamDef { key: "d", default: 1.0, min: -3.0, max: 3.0, periodic: false },
        ],
    });

    // ----- complex / conformal -----
    m.insert("zPow", LayerTypeDef {
        key: "zPow",
        params: vec![
            ParamDef { key: "n", default: 2.0, min: 0.1, max: 5.0, periodic: false },
        ],
    });
    m.insert("zInv", LayerTypeDef {
        key: "zInv",
        params: vec![],
    });
    m.insert("zExp", LayerTypeDef {
        key: "zExp",
        params: vec![],
    });
    m.insert("zLog", LayerTypeDef {
        key: "zLog",
        params: vec![],
    });
    m.insert("zSin", LayerTypeDef {
        key: "zSin",
        params: vec![],
    });
    m.insert("zCos", LayerTypeDef {
        key: "zCos",
        params: vec![],
    });
    m.insert("mobius", LayerTypeDef {
        key: "mobius",
        params: vec![
            ParamDef { key: "a", default: 1.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "b", default: 0.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "c", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "d", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zPlusC", LayerTypeDef {
        key: "zPlusC",
        params: vec![
            ParamDef { key: "cx", default: 0.3, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "cy", default: 0.2, min: -2.0, max: 2.0, periodic: false },
        ],
    });

    // ----- polar / radial -----
    m.insert("spiral", LayerTypeDef {
        key: "spiral",
        params: vec![
            ParamDef { key: "k", default: 1.0, min: -5.0, max: 5.0, periodic: false },
        ],
    });
    m.insert("pinch", LayerTypeDef {
        key: "pinch",
        params: vec![
            ParamDef { key: "p", default: 1.5, min: 0.2, max: 4.0, periodic: false },
        ],
    });
    m.insert("swirl", LayerTypeDef {
        key: "swirl",
        params: vec![
            ParamDef { key: "k", default: 2.0, min: -8.0, max: 8.0, periodic: false },
            ParamDef { key: "sigma", default: 0.4, min: 0.05, max: 2.0, periodic: false },
        ],
    });
    m.insert("radialSine", LayerTypeDef {
        key: "radialSine",
        params: vec![
            ParamDef { key: "amp", default: 0.1, min: 0.0, max: 0.5, periodic: false },
            ParamDef { key: "freq", default: 5.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });
    m.insert("angleWarp", LayerTypeDef {
        key: "angleWarp",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: 0.0, max: 2.0, periodic: false },
            ParamDef { key: "k", default: 3.0, min: 1.0, max: 12.0, periodic: false },
        ],
    });

    // ----- wave / periodic -----
    m.insert("sineWarp", LayerTypeDef {
        key: "sineWarp",
        params: vec![
            ParamDef { key: "amp", default: 0.15, min: 0.0, max: 0.6, periodic: false },
            ParamDef { key: "freq", default: 4.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });
    m.insert("ripple", LayerTypeDef {
        key: "ripple",
        params: vec![
            ParamDef { key: "amp", default: 0.1, min: 0.0, max: 0.5, periodic: false },
            ParamDef { key: "freq", default: 6.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });

    // ----- non-holomorphic -----
    m.insert("conjugate", LayerTypeDef {
        key: "conjugate",
        params: vec![],
    });
    m.insert("reflect", LayerTypeDef {
        key: "reflect",
        params: vec![
            ParamDef { key: "theta", default: 0.0, min: 0.0, max: PI, periodic: true },
        ],
    });
    m.insert("abs", LayerTypeDef {
        key: "abs",
        params: vec![],
    });
    m.insert("mixed", LayerTypeDef {
        key: "mixed",
        params: vec![],
    });

    // ----- utility -----
    m.insert("blend", LayerTypeDef {
        key: "blend",
        params: vec![
            ParamDef { key: "amount", default: 0.5, min: 0.0, max: 1.0, periodic: false },
        ],
    });

    m
}

// =============================================================================
// Lift / displacement types (15 types)
// =============================================================================

fn build_lift() -> HashMap<&'static str, LayerTypeDef> {
    let mut m = HashMap::new();

    // ----- height-style (z only) -----
    m.insert("zMagnitude", LayerTypeDef {
        key: "zMagnitude",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zDisplacement", LayerTypeDef {
        key: "zDisplacement",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zLogJac", LayerTypeDef {
        key: "zLogJac",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zSignedJac", LayerTypeDef {
        key: "zSignedJac",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zReal", LayerTypeDef {
        key: "zReal",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zImag", LayerTypeDef {
        key: "zImag",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("zArg", LayerTypeDef {
        key: "zArg",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -2.0, max: 2.0, periodic: false },
        ],
    });

    // ----- waves / bumps (z) -----
    m.insert("zSineWave", LayerTypeDef {
        key: "zSineWave",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "freq", default: 4.0, min: 0.1, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });
    m.insert("zSineXY", LayerTypeDef {
        key: "zSineXY",
        params: vec![
            ParamDef { key: "amp", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "fx", default: 3.0, min: 0.1, max: 15.0, periodic: false },
            ParamDef { key: "fy", default: 3.0, min: 0.1, max: 15.0, periodic: false },
        ],
    });
    m.insert("zGaussian", LayerTypeDef {
        key: "zGaussian",
        params: vec![
            ParamDef { key: "amp", default: 1.0, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "sigma", default: 0.5, min: 0.05, max: 3.0, periodic: false },
        ],
    });

    // ----- 3D displacement (full vector) -----
    m.insert("spherical", LayerTypeDef {
        key: "spherical",
        params: vec![
            ParamDef { key: "r", default: 1.0, min: 0.1, max: 3.0, periodic: false },
        ],
    });
    m.insert("conical", LayerTypeDef {
        key: "conical",
        params: vec![
            ParamDef { key: "amp", default: 0.8, min: -2.0, max: 2.0, periodic: false },
        ],
    });
    m.insert("helical", LayerTypeDef {
        key: "helical",
        params: vec![
            ParamDef { key: "lift", default: 0.5, min: -2.0, max: 2.0, periodic: false },
            ParamDef { key: "twist", default: 2.0, min: -8.0, max: 8.0, periodic: false },
        ],
    });
    m.insert("noise3D", LayerTypeDef {
        key: "noise3D",
        params: vec![
            ParamDef { key: "amp", default: 0.3, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "freq", default: 3.0, min: 0.5, max: 15.0, periodic: false },
            ParamDef { key: "seed", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
    });
    m.insert("ripple3D", LayerTypeDef {
        key: "ripple3D",
        params: vec![
            ParamDef { key: "amp", default: 0.2, min: -1.0, max: 1.0, periodic: false },
            ParamDef { key: "freq", default: 6.0, min: 0.5, max: 20.0, periodic: false },
            ParamDef { key: "phase", default: 0.0, min: 0.0, max: PI * 2.0, periodic: true },
        ],
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
}
