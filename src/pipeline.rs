use std::collections::HashMap;

use crate::layer::Layer;
use crate::ops::OperationFn;
use crate::lift::LiftFn;
use crate::registry::Registry;
use crate::types::Param;

// =============================================================================
// Helpers
// =============================================================================

/// Flatten a layer's param map to raw f64 values.
fn flatten_params(params: &HashMap<String, Param>) -> HashMap<String, f64> {
    params.iter().map(|(k, p)| (k.clone(), p.value)).collect()
}

// =============================================================================
// Pipeline application (non-flattened)
// =============================================================================

/// Apply the operation pipeline to a single point.
/// Returns the transformed (x, y).
pub fn apply_pipeline(x0: f64, y0: f64, layers: &[Layer], registry: &Registry) -> (f64, f64) {
    let mut x = x0;
    let mut y = y0;
    for layer in layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.operations.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        let flat = flatten_params(&layer.params);
        let (out_x, out_y) = (def.apply)(x, y, &flat, x0, y0);
        let b = layer.blend.value;
        x = x + (out_x - x) * b;
        y = y + (out_y - y) * b;
    }
    (x, y)
}

/// Apply the lift pipeline to a single point.
/// `j_signed_fn` is called lazily only when a layer with `needs_jacobian` is encountered.
/// Returns relative displacement (dx, dy, dz).
pub fn apply_lift_pipeline<F>(
    x: f64,
    y: f64,
    x0: f64,
    y0: f64,
    layers: &[Layer],
    registry: &Registry,
    j_signed_fn: F,
) -> (f64, f64, f64)
where
    F: FnOnce() -> f64,
{
    let mut cx = x;
    let mut cy = y;
    let mut cz = 0.0;
    let mut j_signed: Option<f64> = None;
    let mut j_fn = Some(j_signed_fn);

    for layer in layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.lift.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        if def.needs_jacobian && j_signed.is_none() {
            if let Some(f) = j_fn.take() {
                j_signed = Some(f());
            }
        }
        let flat = flatten_params(&layer.params);
        let j = j_signed.unwrap_or(0.0);
        let (dx, dy, dz) = (def.apply)(cx, cy, &flat, x0, y0, j);
        let b = layer.blend.value;
        cx += dx * b;
        cy += dy * b;
        cz += dz * b;
    }
    (cx - x, cy - y, cz)
}

// =============================================================================
// Pre-flattened hot-path structs
// =============================================================================

/// Pre-computed operation data for the hot path (avoids per-point allocation).
pub struct FlatOperation {
    pub apply: OperationFn,
    pub params: HashMap<String, f64>,
    pub blend: f64,
    pub needs_origin: bool,
}

/// Pre-computed lift data for the hot path.
pub struct FlatLift {
    pub apply: LiftFn,
    pub params: HashMap<String, f64>,
    pub blend: f64,
    pub needs_jacobian: bool,
}

/// Pre-flatten enabled operation layers into a Vec suitable for the hot path.
pub fn pre_flatten_operations(layers: &[Layer], registry: &Registry) -> Vec<FlatOperation> {
    let mut result = Vec::new();
    for layer in layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.operations.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        result.push(FlatOperation {
            apply: def.apply,
            params: flatten_params(&layer.params),
            blend: layer.blend.value,
            needs_origin: def.needs_origin,
        });
    }
    result
}

/// Pre-flatten enabled lift layers into a Vec suitable for the hot path.
pub fn pre_flatten_lifts(layers: &[Layer], registry: &Registry) -> Vec<FlatLift> {
    let mut result = Vec::new();
    for layer in layers {
        if !layer.enabled {
            continue;
        }
        let def = match registry.lift.get(layer.type_key.as_str()) {
            Some(d) => d,
            None => continue,
        };
        result.push(FlatLift {
            apply: def.apply,
            params: flatten_params(&layer.params),
            blend: layer.blend.value,
            needs_jacobian: def.needs_jacobian,
        });
    }
    result
}

// =============================================================================
// Pre-flattened pipeline application
// =============================================================================

/// Apply pre-flattened operation pipeline to a single point.
pub fn apply_pipeline_flat(x0: f64, y0: f64, flat_layers: &[FlatOperation]) -> (f64, f64) {
    let mut x = x0;
    let mut y = y0;
    for fl in flat_layers {
        let (out_x, out_y) = (fl.apply)(x, y, &fl.params, x0, y0);
        let b = fl.blend;
        x = x + (out_x - x) * b;
        y = y + (out_y - y) * b;
    }
    (x, y)
}

/// Apply pre-flattened lift pipeline to a single point.
/// Returns relative displacement (dx, dy, dz).
pub fn apply_lift_pipeline_flat<F>(
    x: f64,
    y: f64,
    x0: f64,
    y0: f64,
    flat_layers: &[FlatLift],
    j_signed_fn: F,
) -> (f64, f64, f64)
where
    F: FnOnce() -> f64,
{
    let mut cx = x;
    let mut cy = y;
    let mut cz = 0.0;
    let mut j_signed: Option<f64> = None;
    let mut j_fn = Some(j_signed_fn);

    for fl in flat_layers {
        if fl.needs_jacobian && j_signed.is_none() {
            if let Some(f) = j_fn.take() {
                j_signed = Some(f());
            }
        }
        let j = j_signed.unwrap_or(0.0);
        let (dx, dy, dz) = (fl.apply)(cx, cy, &fl.params, x0, y0, j);
        let b = fl.blend;
        cx += dx * b;
        cy += dy * b;
        cz += dz * b;
    }
    (cx - x, cy - y, cz)
}

// =============================================================================
// Numerical Jacobian
// =============================================================================

/// Compute the determinant of the Jacobian of the operation pipeline at (x, y)
/// using finite differences.
pub fn numerical_jacobian(x: f64, y: f64, flat_layers: &[FlatOperation]) -> f64 {
    let h = 1e-5;
    let (fx, fy) = apply_pipeline_flat(x, y, flat_layers);
    let (fxh, fyh) = apply_pipeline_flat(x + h, y, flat_layers);
    let (fxk, fyk) = apply_pipeline_flat(x, y + h, flat_layers);
    let dfdx = (fxh - fx) / h;
    let dfdy = (fxk - fx) / h;
    let dgdx = (fyh - fy) / h;
    let dgdy = (fyk - fy) / h;
    dfdx * dgdy - dfdy * dgdx
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;
    use crate::types::Param;

    fn make_test_layer(type_key: &str, params: &[(&str, f64)]) -> Layer {
        let mut param_map = HashMap::new();
        for (k, v) in params {
            param_map.insert(k.to_string(), Param::new(*v, -10.0, 10.0, false));
        }
        Layer {
            id: "test".to_string(),
            type_key: type_key.to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params: param_map,
        }
    }

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    #[test]
    fn single_layer_blend_one() {
        let reg = Registry::new();
        let layer = make_test_layer("scale", &[("sx", 2.0), ("sy", 3.0)]);
        let (x, y) = apply_pipeline(1.0, 1.0, &[layer], &reg);
        assert_near(x, 2.0, 1e-10);
        assert_near(y, 3.0, 1e-10);
    }

    #[test]
    fn single_layer_blend_half() {
        let reg = Registry::new();
        let mut layer = make_test_layer("scale", &[("sx", 3.0), ("sy", 3.0)]);
        layer.blend.value = 0.5;
        let (x, y) = apply_pipeline(1.0, 1.0, &[layer], &reg);
        // lerp(1, 3, 0.5) = 2
        assert_near(x, 2.0, 1e-10);
        assert_near(y, 2.0, 1e-10);
    }

    #[test]
    fn multi_layer_non_commutativity() {
        let reg = Registry::new();
        let scale_layer = make_test_layer("scale", &[("sx", 2.0), ("sy", 2.0)]);
        let translate_layer = make_test_layer("translate", &[("tx", 1.0), ("ty", 0.0)]);

        let (x1, _y1) = apply_pipeline(1.0, 0.0, &[scale_layer.clone(), translate_layer.clone()], &reg);
        let (x2, _y2) = apply_pipeline(1.0, 0.0, &[translate_layer, scale_layer], &reg);

        // scale then translate: (1,0) -> (2,0) -> (3,0)
        assert_near(x1, 3.0, 1e-10);
        // translate then scale: (1,0) -> (2,0) -> (4,0)
        assert_near(x2, 4.0, 1e-10);
        assert!((x1 - x2).abs() > 0.5);
    }

    #[test]
    fn disabled_layer_skipped() {
        let reg = Registry::new();
        let mut layer = make_test_layer("scale", &[("sx", 5.0), ("sy", 5.0)]);
        layer.enabled = false;
        let (x, y) = apply_pipeline(1.0, 1.0, &[layer], &reg);
        assert_near(x, 1.0, 1e-10);
        assert_near(y, 1.0, 1e-10);
    }

    #[test]
    fn pre_flatten_equivalence_operations() {
        let reg = Registry::new();
        let layer1 = make_test_layer("rotate", &[("theta", std::f64::consts::PI / 3.0)]);
        let layer2 = make_test_layer("scale", &[("sx", 1.5), ("sy", 0.8)]);
        let layers = vec![layer1, layer2];

        let (x1, y1) = apply_pipeline(1.0, 1.0, &layers, &reg);
        let flat = pre_flatten_operations(&layers, &reg);
        let (x2, y2) = apply_pipeline_flat(1.0, 1.0, &flat);

        assert_near(x2, x1, 1e-12);
        assert_near(y2, y1, 1e-12);
    }

    #[test]
    fn pre_flatten_equivalence_lifts() {
        let reg = Registry::new();
        let layer = make_test_layer("zMagnitude", &[("amp", 1.0)]);
        let layers = vec![layer];

        let (dx1, dy1, dz1) = apply_lift_pipeline(3.0, 4.0, 0.0, 0.0, &layers, &reg, || 1.0);
        let flat = pre_flatten_lifts(&layers, &reg);
        let (dx2, dy2, dz2) = apply_lift_pipeline_flat(3.0, 4.0, 0.0, 0.0, &flat, || 1.0);

        assert_near(dx2, dx1, 1e-12);
        assert_near(dy2, dy1, 1e-12);
        assert_near(dz2, dz1, 1e-12);
    }

    #[test]
    fn jacobian_identity() {
        let reg = Registry::new();
        let layer = make_test_layer("translate", &[("tx", 0.0), ("ty", 0.0)]);
        let flat = pre_flatten_operations(&[layer], &reg);
        let det = numerical_jacobian(1.0, 1.0, &flat);
        assert_near(det, 1.0, 1e-4);
    }

    #[test]
    fn jacobian_scale_2_2() {
        let reg = Registry::new();
        let layer = make_test_layer("scale", &[("sx", 2.0), ("sy", 2.0)]);
        let flat = pre_flatten_operations(&[layer], &reg);
        let det = numerical_jacobian(1.0, 1.0, &flat);
        assert_near(det, 4.0, 1e-4);
    }

    #[test]
    fn jacobian_rotate() {
        let reg = Registry::new();
        let layer = make_test_layer("rotate", &[("theta", std::f64::consts::PI / 4.0)]);
        let flat = pre_flatten_operations(&[layer], &reg);
        let det = numerical_jacobian(1.0, 1.0, &flat);
        assert_near(det, 1.0, 1e-4);
    }

    #[test]
    fn jacobian_scale_3_2() {
        let reg = Registry::new();
        let layer = make_test_layer("scale", &[("sx", 3.0), ("sy", 2.0)]);
        let flat = pre_flatten_operations(&[layer], &reg);
        let det = numerical_jacobian(1.0, 1.0, &flat);
        assert_near(det, 6.0, 1e-4);
    }
}
