use crate::layer::Layer;
use crate::registry::Registry;

// =============================================================================
// Formula symbol table
// =============================================================================

/// Map operation/lift labels to mathematical notation symbols.
pub fn formula_sym(label: &str) -> &str {
    match label {
        // Operations (R2 -> R2)
        "translate" => "T",
        "scale" => "S",
        "rotate" => "R",
        "shear" => "K",
        "2x2 matrix" => "GL(2)",
        "z^n" => "z^n",
        "1/z" => "z^-1",
        "exp(z)" => "e^z",
        "log(z)" => "log z",
        "sin(z)" => "sin z",
        "cos(z)" => "cos z",
        "mobius (az+b)/(cz+d)" => "(az+b)/(cz+d)",
        "z + c" => "z+c",
        "spiral" => "r e^(ik ln r)",
        "pinch / bulge" => "r^p",
        "swirl" => "e^(-r^2) d/dth",
        "radial sine" => "r+a sin(wr)",
        "angle warp" => "th+a sin(n th)",
        "sine warp" => "sin(kx, ky)",
        "ripple" => "r+a sin(kr)",
        "conjugate" => "z*",
        "reflect" => "-x",
        "|z|" => "|z|",
        "mixed monomials" => "x^a y^b",
        "blend (utility)" => "I",
        // Lift (displacements)
        "|.| -> z" => "|T| -> z",
        "|T|-|z| -> z" => "|T|-|z|",
        "log|det J| -> z" => "ln|J|",
        "det J -> z" => "det J",
        "Re(T) -> z" => "Re(T)",
        "Im(T) -> z" => "Im(T)",
        "arg(T) -> z" => "arg(T)",
        "sine wave (z)" => "sin(kx) -> z",
        "sine xy (z)" => "sin(kx,ky) -> z",
        "gaussian bump (z)" => "e^(-r^2) -> z",
        "spherical projection" => "S^2",
        "conical lift" => "C_r,th",
        "helical twist" => "H_th,p",
        "noise displacement" => "N_f,a",
        "ripple (z + xy)" => "sin(kr) -> xyz",
        // Fallback: return as-is
        other => other,
    }
}

// =============================================================================
// Camera flags for composition string
// =============================================================================

/// Flags indicating which camera/view parameters are active.
pub struct CameraFlags {
    pub quat: bool,
    pub az: bool,
    pub el: bool,
    pub d: bool,
    pub tx: bool,
    pub ty: bool,
    pub roll: bool,
    pub f: bool,
    pub sensor: bool,
    pub ortho: bool,
    pub anam: bool,
    pub sx: bool,
    pub sy: bool,
    pub k1: bool,
    pub k2: bool,
}

impl Default for CameraFlags {
    fn default() -> Self {
        Self {
            quat: false,
            az: false,
            el: false,
            d: false,
            tx: false,
            ty: false,
            roll: false,
            f: false,
            sensor: false,
            ortho: false,
            anam: false,
            sx: false,
            sy: false,
            k1: false,
            k2: false,
        }
    }
}

// =============================================================================
// Composition string
// =============================================================================

/// Build the mathematical composition string for the current pipeline state.
pub fn composition_string(
    op_layers: &[Layer],
    lift_layers: &[Layer],
    registry: &Registry,
    flags: &CameraFlags,
) -> String {
    // Innermost: lattice
    let mut core = String::from("L");

    // Operations: composed right-to-left (math convention)
    let ops: Vec<&str> = op_layers
        .iter()
        .filter(|l| l.enabled)
        .filter_map(|l| {
            registry
                .operations
                .get(l.type_key.as_str())
                .map(|d| formula_sym(d.label))
        })
        .collect();
    if !ops.is_empty() {
        let mut reversed = ops;
        reversed.reverse();
        core = format!("{}({})", reversed.join(" . "), core);
    }

    // Lift: additive
    let dz: Vec<&str> = lift_layers
        .iter()
        .filter(|l| l.enabled)
        .filter_map(|l| {
            registry
                .lift
                .get(l.type_key.as_str())
                .map(|d| formula_sym(d.label))
        })
        .collect();
    if dz.len() == 1 {
        core = format!("{} + {}", core, dz[0]);
    } else if dz.len() > 1 {
        core = format!("{} + Sum[{}]", core, dz.join(", "));
    }

    // Quaternion tumble
    if flags.quat {
        core = format!("Q({})", core);
    }

    // View transform
    let mut v_parts = Vec::new();
    if flags.az { v_parts.push("th"); }
    if flags.el { v_parts.push("ph"); }
    if flags.d { v_parts.push("r"); }
    if flags.tx { v_parts.push("tx"); }
    if flags.ty { v_parts.push("ty"); }
    if flags.roll { v_parts.push("rho"); }
    if !v_parts.is_empty() {
        core = format!("V_{{{}}}({})", v_parts.join(","), core);
    }

    // Projection
    let mut p_parts = Vec::new();
    if flags.f { p_parts.push("f"); }
    if flags.sensor { p_parts.push("s"); }
    if flags.ortho { p_parts.push("z_o"); }
    if flags.anam { p_parts.push("alpha"); }
    if flags.sx { p_parts.push("sx"); }
    if flags.sy { p_parts.push("sy"); }
    if !p_parts.is_empty() {
        core = format!("P_{{{}}}({})", p_parts.join(","), core);
    }

    // Distortion: outermost
    let mut d_parts = Vec::new();
    if flags.k1 { d_parts.push("k1"); }
    if flags.k2 { d_parts.push("k2"); }
    if !d_parts.is_empty() {
        core = format!("D_{{{}}}({})", d_parts.join(","), core);
    }

    core
}

// =============================================================================
// DOF counting
// =============================================================================

/// A single entry in the DOF breakdown.
#[derive(Debug, Clone, PartialEq)]
pub struct DofEntry {
    pub label: String,
    pub structural: usize,
    pub temporal: usize,
}

/// Result of DOF counting.
#[derive(Debug, Clone, PartialEq)]
pub struct DofResult {
    pub active: Vec<DofEntry>,
    pub structural: usize,
    pub temporal: usize,
    pub operators: usize,
    pub total: usize,
}

/// A view parameter descriptor for DOF counting.
pub struct ViewParam {
    pub label: String,
    pub on: bool,
    pub structural: usize,
    pub anim: bool,
}

/// Count structural and temporal degrees of freedom.
pub fn count_dofs(
    op_layers: &[Layer],
    lift_layers: &[Layer],
    view: &[ViewParam],
    registry: &Registry,
) -> DofResult {
    let mut active = vec![DofEntry {
        label: "base 2-manifold".to_string(),
        structural: 2,
        temporal: 0,
    }];
    let mut structural: usize = 2;
    let mut temporal: usize = 0;

    // Tally operation layers
    for layer in op_layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.operations.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        let s = def.params.len() + 1; // +1 for blend
        let mut t: usize = 0;

        // Check blend animation
        if layer.blend.anim {
            t += 1;
        }
        if let Some(ref po) = layer.blend.phase_offset {
            if po.anim {
                t += 1;
            }
        }

        // Check param animations
        for param in layer.params.values() {
            if param.anim {
                t += 1;
            }
            if let Some(ref po) = param.phase_offset {
                if po.anim {
                    t += 1;
                }
            }
        }

        active.push(DofEntry {
            label: def.label.to_string(),
            structural: s,
            temporal: t,
        });
        structural += s;
        temporal += t;
    }

    // Tally lift layers
    for layer in lift_layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.lift.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        let s = def.params.len() + 1;
        let mut t: usize = 0;

        if layer.blend.anim {
            t += 1;
        }
        if let Some(ref po) = layer.blend.phase_offset {
            if po.anim {
                t += 1;
            }
        }

        for param in layer.params.values() {
            if param.anim {
                t += 1;
            }
            if let Some(ref po) = param.phase_offset {
                if po.anim {
                    t += 1;
                }
            }
        }

        active.push(DofEntry {
            label: def.label.to_string(),
            structural: s,
            temporal: t,
        });
        structural += s;
        temporal += t;
    }

    // Tally view params
    for v in view {
        if !v.on {
            continue;
        }
        let t = if v.anim { 1 } else { 0 };
        active.push(DofEntry {
            label: v.label.clone(),
            structural: v.structural,
            temporal: t,
        });
        structural += v.structural;
        temporal += t;
    }

    let operators = active.len() - 1; // exclude base manifold
    let total = structural + temporal;
    DofResult {
        active,
        structural,
        temporal,
        operators,
        total,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::Layer;
    use crate::registry::Registry;
    use crate::types::Param;
    use std::collections::HashMap;

    fn make_test_layer(type_key: &str, param_keys: &[&str]) -> Layer {
        let mut params = HashMap::new();
        for k in param_keys {
            params.insert(k.to_string(), Param::new(1.0, -10.0, 10.0, false));
        }
        Layer {
            id: "test".to_string(),
            type_key: type_key.to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params,
        }
    }

    #[test]
    fn empty_pipeline_returns_l() {
        let reg = Registry::new();
        let result = composition_string(&[], &[], &reg, &CameraFlags::default());
        assert_eq!(result, "L");
    }

    #[test]
    fn single_operation() {
        let reg = Registry::new();
        let layer = make_test_layer("rotate", &["theta"]);
        let result = composition_string(&[layer], &[], &reg, &CameraFlags::default());
        assert_eq!(result, "R(L)");
    }

    #[test]
    fn two_operations_reversed() {
        let reg = Registry::new();
        let r = make_test_layer("rotate", &["theta"]);
        let s = make_test_layer("scale", &["sx", "sy"]);
        let result = composition_string(&[r, s], &[], &reg, &CameraFlags::default());
        assert_eq!(result, "S . R(L)");
    }

    #[test]
    fn single_displacement() {
        let reg = Registry::new();
        let d = make_test_layer("zMagnitude", &["amp"]);
        let result = composition_string(&[], &[d], &reg, &CameraFlags::default());
        assert_eq!(result, "L + |T| -> z");
    }

    #[test]
    fn disabled_layers_excluded() {
        let reg = Registry::new();
        let mut layer = make_test_layer("rotate", &["theta"]);
        layer.enabled = false;
        let result = composition_string(&[layer], &[], &reg, &CameraFlags::default());
        assert_eq!(result, "L");
    }

    #[test]
    fn dof_empty_pipeline() {
        let reg = Registry::new();
        let result = count_dofs(&[], &[], &[], &reg);
        assert_eq!(result.structural, 2);
        assert_eq!(result.temporal, 0);
    }

    #[test]
    fn dof_single_1_param_layer() {
        let reg = Registry::new();
        let layer = make_test_layer("zPow", &["n"]);
        let result = count_dofs(&[layer], &[], &[], &reg);
        // base=2, + params(1) + blend(1) = 4
        assert_eq!(result.structural, 4);
        assert_eq!(result.temporal, 0);
    }

    #[test]
    fn dof_single_2_param_layer() {
        let reg = Registry::new();
        let layer = make_test_layer("scale", &["sx", "sy"]);
        let result = count_dofs(&[layer], &[], &[], &reg);
        // base=2, + params(2) + blend(1) = 5
        assert_eq!(result.structural, 5);
        assert_eq!(result.temporal, 0);
    }

    #[test]
    fn dof_animated_param() {
        let reg = Registry::new();
        let mut layer = make_test_layer("rotate", &["theta"]);
        layer.params.get_mut("theta").unwrap().anim = true;
        let result = count_dofs(&[layer], &[], &[], &reg);
        assert_eq!(result.temporal, 1);
    }

    #[test]
    fn dof_disabled_layer_not_counted() {
        let reg = Registry::new();
        let mut layer = make_test_layer("rotate", &["theta"]);
        layer.enabled = false;
        let result = count_dofs(&[layer], &[], &[], &reg);
        assert_eq!(result.structural, 2);
        assert_eq!(result.temporal, 0);
    }

    #[test]
    fn composition_with_quaternion() {
        let reg = Registry::new();
        let flags = CameraFlags { quat: true, ..CameraFlags::default() };
        let result = composition_string(&[], &[], &reg, &flags);
        assert_eq!(result, "Q(L)");
    }

    #[test]
    fn composition_with_view_and_projection() {
        let reg = Registry::new();
        let flags = CameraFlags {
            az: true,
            el: true,
            f: true,
            ..CameraFlags::default()
        };
        let result = composition_string(&[], &[], &reg, &flags);
        assert_eq!(result, "P_{f}(V_{th,ph}(L))");
    }

    #[test]
    fn composition_with_distortion() {
        let reg = Registry::new();
        let flags = CameraFlags {
            k1: true,
            k2: true,
            ..CameraFlags::default()
        };
        let result = composition_string(&[], &[], &reg, &flags);
        assert_eq!(result, "D_{k1,k2}(L)");
    }

    #[test]
    fn dof_view_params() {
        let reg = Registry::new();
        let view = vec![
            ViewParam { label: "azimuth".to_string(), on: true, structural: 1, anim: true },
            ViewParam { label: "elevation".to_string(), on: true, structural: 1, anim: false },
            ViewParam { label: "disabled".to_string(), on: false, structural: 1, anim: true },
        ];
        let result = count_dofs(&[], &[], &view, &reg);
        // base=2, + az(1) + el(1) = 4
        assert_eq!(result.structural, 4);
        // only az is animated
        assert_eq!(result.temporal, 1);
        // 2 active operators (az + el), disabled excluded
        assert_eq!(result.operators, 2);
    }

    #[test]
    fn multiple_displacements() {
        let reg = Registry::new();
        let d1 = make_test_layer("zMagnitude", &["amp"]);
        let d2 = make_test_layer("zReal", &["amp"]);
        let result = composition_string(&[], &[d1, d2], &reg, &CameraFlags::default());
        assert_eq!(result, "L + Sum[|T| -> z, Re(T)]");
    }
}
