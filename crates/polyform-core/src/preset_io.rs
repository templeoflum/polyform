use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::preset::{Camera, Phosphor, Preset, Quaternion, Sampling};
use crate::registry::Registry;
use crate::types::{default_blend, Param, PhaseModulator, ShapeFunction};

// =============================================================================
// Public types
// =============================================================================

/// Result of importing a preset with possible warnings.
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub preset: Preset,
    pub warnings: Vec<String>,
}

/// Error when import fails entirely.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    InvalidJson(String),
    NotAnObject,
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::InvalidJson(msg) => write!(f, "Invalid JSON: {msg}"),
            ImportError::NotAnObject => write!(f, "Preset JSON must be an object"),
        }
    }
}

impl std::error::Error for ImportError {}

// =============================================================================
// PresetFile — tagged union for full + section presets
// =============================================================================

/// Discriminator for preset file kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresetKind {
    Full,
    Operations,
    Lift,
    Camera,
    Phosphor,
    Sampling,
}

impl PresetKind {
    /// Subdirectory name in the preset library.
    pub fn dir_name(&self) -> &'static str {
        match self {
            PresetKind::Full => "full",
            PresetKind::Operations => "operations",
            PresetKind::Lift => "lift",
            PresetKind::Camera => "camera",
            PresetKind::Phosphor => "phosphor",
            PresetKind::Sampling => "sampling",
        }
    }
}

/// A preset file that self-describes what it contains via the `kind` tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PresetFile {
    #[serde(rename = "full")]
    Full(Preset),
    #[serde(rename = "operations")]
    Operations { layers: Vec<Layer> },
    #[serde(rename = "lift")]
    Lift { layers: Vec<Layer> },
    #[serde(rename = "camera")]
    Camera {
        camera: Camera,
        quaternion: Quaternion,
    },
    #[serde(rename = "phosphor")]
    Phosphor { phosphor: Phosphor },
    #[serde(rename = "sampling")]
    Sampling { sampling: Sampling },
}

impl PresetFile {
    pub fn kind(&self) -> PresetKind {
        match self {
            PresetFile::Full(_) => PresetKind::Full,
            PresetFile::Operations { .. } => PresetKind::Operations,
            PresetFile::Lift { .. } => PresetKind::Lift,
            PresetFile::Camera { .. } => PresetKind::Camera,
            PresetFile::Phosphor { .. } => PresetKind::Phosphor,
            PresetFile::Sampling { .. } => PresetKind::Sampling,
        }
    }

    /// Extract from a live preset.
    pub fn from_preset(preset: &Preset, kind: PresetKind) -> Self {
        match kind {
            PresetKind::Full => PresetFile::Full(preset.clone()),
            PresetKind::Operations => PresetFile::Operations {
                layers: preset.pipeline.operations.clone(),
            },
            PresetKind::Lift => PresetFile::Lift {
                layers: preset.pipeline.lift.clone(),
            },
            PresetKind::Camera => PresetFile::Camera {
                camera: preset.camera.clone(),
                quaternion: preset.quaternion.clone(),
            },
            PresetKind::Phosphor => PresetFile::Phosphor {
                phosphor: preset.phosphor.clone(),
            },
            PresetKind::Sampling => PresetFile::Sampling {
                sampling: preset.sampling.clone(),
            },
        }
    }

    /// Apply section data into a live preset.
    pub fn apply_to_preset(self, preset: &mut Preset) {
        match self {
            PresetFile::Full(p) => *preset = p,
            PresetFile::Operations { layers } => preset.pipeline.operations = layers,
            PresetFile::Lift { layers } => preset.pipeline.lift = layers,
            PresetFile::Camera { camera, quaternion } => {
                preset.camera = camera;
                preset.quaternion = quaternion;
            }
            PresetFile::Phosphor { phosphor } => preset.phosphor = phosphor,
            PresetFile::Sampling { sampling } => preset.sampling = sampling,
        }
    }
}

/// Serialize a PresetFile to JSON.
pub fn export_preset_file_json(file: &PresetFile, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(file).unwrap()
    } else {
        serde_json::to_string(file).unwrap()
    }
}

/// Deserialize a PresetFile from JSON.
pub fn import_preset_file_json(json: &str) -> Result<PresetFile, ImportError> {
    serde_json::from_str(json).map_err(|e| ImportError::InvalidJson(e.to_string()))
}

// =============================================================================
// reconcile_preset — drop unknown types, fill missing params, drop orphaned params
// =============================================================================

/// Mutates the preset in place, returns warnings.
///
/// For both `pipeline.operations` and `pipeline.lift`:
/// 1. Drop unknown types — if `layer.type_key` not in registry → remove layer, push warning
/// 2. Fill missing params — for each registry ParamDef not in layer.params → insert default
/// 3. Drop orphaned params — for each key in layer.params not in registry → remove, push warning
pub fn reconcile_preset(preset: &mut Preset, registry: &Registry) -> Vec<String> {
    let mut warnings = Vec::new();

    // Reconcile operations
    let mut i = 0;
    while i < preset.pipeline.operations.len() {
        let layer = &preset.pipeline.operations[i];
        if let Some(def) = registry.operations.get(layer.type_key.as_str()) {
            let type_key = layer.type_key.clone();

            // Fill missing params from registry defaults
            for param_def in &def.params {
                if !preset.pipeline.operations[i].params.contains_key(param_def.key) {
                    let param = Param::new(
                        param_def.default,
                        param_def.min,
                        param_def.max,
                        param_def.periodic,
                    );
                    preset.pipeline.operations[i]
                        .params
                        .insert(param_def.key.to_string(), param);
                }
            }

            // Collect registry param keys for this type
            let valid_keys: Vec<&str> = def.params.iter().map(|p| p.key).collect();

            // Drop orphaned params
            let layer_keys: Vec<String> = preset.pipeline.operations[i]
                .params
                .keys()
                .cloned()
                .collect();
            for key in layer_keys {
                if !valid_keys.contains(&key.as_str()) {
                    preset.pipeline.operations[i].params.remove(&key);
                    warnings.push(format!(
                        "operation \"{type_key}\" has unknown param \"{key}\" - dropped."
                    ));
                }
            }

            i += 1;
        } else {
            let type_key = preset.pipeline.operations[i].type_key.clone();
            warnings.push(format!("Unknown operation type \"{type_key}\" - layer dropped."));
            preset.pipeline.operations.remove(i);
        }
    }

    // Reconcile lift
    let mut i = 0;
    while i < preset.pipeline.lift.len() {
        let layer = &preset.pipeline.lift[i];
        if let Some(def) = registry.lift.get(layer.type_key.as_str()) {
            let type_key = layer.type_key.clone();

            // Fill missing params from registry defaults
            for param_def in &def.params {
                if !preset.pipeline.lift[i].params.contains_key(param_def.key) {
                    let param = Param::new(
                        param_def.default,
                        param_def.min,
                        param_def.max,
                        param_def.periodic,
                    );
                    preset.pipeline.lift[i]
                        .params
                        .insert(param_def.key.to_string(), param);
                }
            }

            // Collect registry param keys for this type
            let valid_keys: Vec<&str> = def.params.iter().map(|p| p.key).collect();

            // Drop orphaned params
            let layer_keys: Vec<String> =
                preset.pipeline.lift[i].params.keys().cloned().collect();
            for key in layer_keys {
                if !valid_keys.contains(&key.as_str()) {
                    preset.pipeline.lift[i].params.remove(&key);
                    warnings.push(format!(
                        "lift \"{type_key}\" has unknown param \"{key}\" - dropped."
                    ));
                }
            }

            i += 1;
        } else {
            let type_key = preset.pipeline.lift[i].type_key.clone();
            warnings.push(format!("Unknown lift type \"{type_key}\" - layer dropped."));
            preset.pipeline.lift.remove(i);
        }
    }

    warnings
}

// =============================================================================
// import_preset_json — parse + reconcile
// =============================================================================

/// Import a preset from JSON with tolerant deserialization and registry reconciliation.
///
/// 1. Parse JSON into serde_json::Value
/// 2. Attempt strict serde deserialization; on failure, fall back to tolerant validation
/// 3. Call reconcile_preset
/// 4. Return ImportResult { preset, warnings }
pub fn import_preset_json(json: &str, registry: &Registry) -> Result<ImportResult, ImportError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| ImportError::InvalidJson(e.to_string()))?;

    if !value.is_object() {
        return Err(ImportError::NotAnObject);
    }

    // Try strict deserialization first (zero-cost for well-formed presets)
    let (mut preset, mut warnings) = match serde_json::from_value::<Preset>(value.clone()) {
        Ok(p) => (p, Vec::new()),
        Err(_) => {
            // Fall back to tolerant deserialization
            validate_preset_value(&value)
        }
    };

    // Registry reconciliation
    let reconcile_warnings = reconcile_preset(&mut preset, registry);
    warnings.extend(reconcile_warnings);

    Ok(ImportResult { preset, warnings })
}

// =============================================================================
// make_layer — factory function
// =============================================================================

/// Construct a new layer from a registry type key.
///
/// Looks up in operations first, then lift. Returns None if type_key not found.
pub fn make_layer(type_key: &str, registry: &Registry) -> Option<Layer> {
    if let Some(def) = registry.operations.get(type_key) {
        let mut params = HashMap::new();
        for param_def in &def.params {
            params.insert(
                param_def.key.to_string(),
                Param::new(param_def.default, param_def.min, param_def.max, param_def.periodic),
            );
        }
        Some(Layer {
            id: uuid::Uuid::new_v4().to_string(),
            type_key: type_key.to_string(),
            enabled: true,
            blend: default_blend(),
            blend_time_override: None,
            params,
        })
    } else if let Some(def) = registry.lift.get(type_key) {
        let mut params = HashMap::new();
        for param_def in &def.params {
            params.insert(
                param_def.key.to_string(),
                Param::new(param_def.default, param_def.min, param_def.max, param_def.periodic),
            );
        }
        Some(Layer {
            id: uuid::Uuid::new_v4().to_string(),
            type_key: type_key.to_string(),
            enabled: true,
            blend: default_blend(),
            blend_time_override: None,
            params,
        })
    } else {
        None
    }
}

// =============================================================================
// export_preset_json — serialize
// =============================================================================

/// Serialize a preset to JSON.
pub fn export_preset_json(preset: &Preset, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(preset).unwrap()
    } else {
        serde_json::to_string(preset).unwrap()
    }
}

// =============================================================================
// Tolerant deserialization — fallback when strict serde fails
// =============================================================================

/// Walk the JSON value tree manually, extracting a Preset with fallbacks for type mismatches.
/// Returns a fully-populated Preset + accumulated warnings.
fn validate_preset_value(value: &serde_json::Value) -> (Preset, Vec<String>) {
    let mut warnings = Vec::new();
    let mut preset = Preset::default();

    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            warnings.push("Preset is not an object, using defaults.".to_string());
            return (preset, warnings);
        }
    };

    // metadata
    if let Some(v) = obj.get("metadata") {
        if let Some(m) = v.as_object() {
            if let Some(v) = m.get("name") {
                preset.metadata.name = as_string(v, "Untitled");
            }
            if let Some(v) = m.get("description") {
                preset.metadata.description = as_string(v, "");
            }
            if let Some(v) = m.get("author") {
                preset.metadata.author = as_string(v, "");
            }
            if let Some(v) = m.get("created") {
                preset.metadata.created = as_string(v, "");
            }
            if let Some(v) = m.get("modified") {
                preset.metadata.modified = as_string(v, "");
            }
            if let Some(v) = m.get("tags") {
                if let Some(arr) = v.as_array() {
                    preset.metadata.tags = arr
                        .iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        } else {
            warnings.push("metadata is not an object, using defaults.".to_string());
        }
    }

    // Note: schema, version, tempo, defaults, palette, display, viewport are
    // ephemeral session state — silently ignored on import.

    // pipeline
    if let Some(v) = obj.get("pipeline") {
        if let Some(p) = v.as_object() {
            if let Some(ops) = p.get("operations") {
                if let Some(arr) = ops.as_array() {
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(layer) =
                            read_layer(item, &mut warnings, &format!("pipeline.operations[{i}]"))
                        {
                            preset.pipeline.operations.push(layer);
                        }
                    }
                }
            }
            if let Some(lifts) = p.get("lift") {
                if let Some(arr) = lifts.as_array() {
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(layer) =
                            read_layer(item, &mut warnings, &format!("pipeline.lift[{i}]"))
                        {
                            preset.pipeline.lift.push(layer);
                        }
                    }
                }
            }
        }
    }

    // camera — attempt serde on the sub-object, fall back to default
    if let Some(v) = obj.get("camera") {
        match serde_json::from_value(v.clone()) {
            Ok(cam) => preset.camera = cam,
            Err(_) => {
                warnings.push("camera has invalid structure, using defaults.".to_string());
            }
        }
    }

    // quaternion
    if let Some(v) = obj.get("quaternion") {
        match serde_json::from_value(v.clone()) {
            Ok(q) => preset.quaternion = q,
            Err(_) => {
                warnings.push("quaternion has invalid structure, using defaults.".to_string());
            }
        }
    }

    // sampling
    if let Some(v) = obj.get("sampling") {
        match serde_json::from_value(v.clone()) {
            Ok(s) => preset.sampling = s,
            Err(_) => {
                warnings.push("sampling has invalid structure, using defaults.".to_string());
            }
        }
    }

    (preset, warnings)
}

// =============================================================================
// Helper functions for tolerant reading
// =============================================================================

fn as_f64(v: &serde_json::Value, fallback: f64, warnings: &mut Vec<String>, path: &str) -> f64 {
    match v.as_f64() {
        Some(n) => n,
        None => {
            // Try integer
            if let Some(n) = v.as_i64() {
                return n as f64;
            }
            warnings.push(format!("{path}: expected number, using default {fallback}."));
            fallback
        }
    }
}

fn as_bool(v: &serde_json::Value, fallback: bool) -> bool {
    v.as_bool().unwrap_or(fallback)
}

fn as_string(v: &serde_json::Value, fallback: &str) -> String {
    v.as_str().unwrap_or(fallback).to_string()
}

fn read_param(
    v: &serde_json::Value,
    fallback: &Param,
    warnings: &mut Vec<String>,
    path: &str,
) -> Param {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            warnings.push(format!("{path}: expected param object, using default."));
            return fallback.clone();
        }
    };

    let value = obj
        .get("value")
        .map(|v| as_f64(v, fallback.value, warnings, &format!("{path}.value")))
        .unwrap_or(fallback.value);
    let min = obj
        .get("min")
        .map(|v| as_f64(v, fallback.min, warnings, &format!("{path}.min")))
        .unwrap_or(fallback.min);
    let max = obj
        .get("max")
        .map(|v| as_f64(v, fallback.max, warnings, &format!("{path}.max")))
        .unwrap_or(fallback.max);
    let anim = obj.get("anim").map(|v| as_bool(v, false)).unwrap_or(false);
    let ratio = obj
        .get("ratio")
        .map(|v| as_f64(v, 1.0, warnings, &format!("{path}.ratio")))
        .unwrap_or(1.0);
    let offset = obj
        .get("offset")
        .map(|v| as_f64(v, 0.0, warnings, &format!("{path}.offset")))
        .unwrap_or(0.0);

    // Shape: read from "shape" field
    let shape = if let Some(v) = obj.get("shape") {
        v.as_str()
            .map(|s| match s {
                "sawtooth" => ShapeFunction::Sawtooth,
                _ => ShapeFunction::Cosine,
            })
            .unwrap_or(fallback.shape)
    } else {
        fallback.shape
    };

    // Modulator: read from "modulator" field if present
    let modulator = obj.get("modulator").and_then(|v| {
        if v.is_null() {
            None
        } else if let Some(m) = v.as_object() {
            let m_ratio = m.get("ratio")
                .map(|v| as_f64(v, 1.0, warnings, &format!("{path}.modulator.ratio")))
                .unwrap_or(1.0);
            let m_depth = m.get("depth")
                .map(|v| as_f64(v, 0.0, warnings, &format!("{path}.modulator.depth")))
                .unwrap_or(0.0);
            let m_shape = m.get("shape")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "sawtooth" => ShapeFunction::Sawtooth,
                    _ => ShapeFunction::Cosine,
                })
                .unwrap_or(ShapeFunction::Cosine);
            Some(PhaseModulator {
                ratio: m_ratio,
                depth: m_depth,
                shape: m_shape,
            })
        } else {
            None
        }
    });

    Param {
        value,
        min,
        max,
        anim,
        ratio,
        offset,
        shape,
        modulator,
    }
}

fn read_layer(
    v: &serde_json::Value,
    warnings: &mut Vec<String>,
    path: &str,
) -> Option<Layer> {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            warnings.push(format!("{path}: not an object, skipping."));
            return None;
        }
    };

    let type_key = match obj.get("type").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => {
            warnings.push(format!("{path}: missing \"type\" field, skipping."));
            return None;
        }
    };

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let enabled = obj.get("enabled").map(|v| as_bool(v, true)).unwrap_or(true);

    let default_blend_param = default_blend();
    let blend = obj
        .get("blend")
        .map(|v| read_param(v, &default_blend_param, warnings, &format!("{path}.blend")))
        .unwrap_or_else(|| default_blend_param);

    let blend_time_override = obj
        .get("blendTimeOverride")
        .and_then(|v| {
            if v.is_null() {
                None
            } else {
                v.as_f64()
            }
        });

    let mut params = HashMap::new();
    if let Some(params_val) = obj.get("params") {
        if let Some(params_obj) = params_val.as_object() {
            let default_param = Param::default();
            for (key, val) in params_obj {
                let param = read_param(val, &default_param, warnings, &format!("{path}.params.{key}"));
                params.insert(key.clone(), param);
            }
        }
    }

    Some(Layer {
        id,
        type_key,
        enabled,
        blend,
        blend_time_override,
        params,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset::Preset;
    use crate::registry::Registry;

    #[test]
    fn roundtrip_export_import_same_preset() {
        let registry = Registry::new();

        // Build a preset with layers
        let mut preset = Preset::default();
        preset.pipeline.operations.push(make_layer("rotate", &registry).unwrap());
        preset.pipeline.lift.push(make_layer("zMagnitude", &registry).unwrap());

        let json = export_preset_json(&preset, true);
        let result = import_preset_json(&json, &registry).unwrap();

        // After reconciliation, same structure
        assert_eq!(result.preset.pipeline.operations.len(), 1);
        assert_eq!(result.preset.pipeline.lift.len(), 1);
        assert_eq!(result.preset.pipeline.operations[0].type_key, "rotate");
        assert_eq!(result.preset.pipeline.lift[0].type_key, "zMagnitude");
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn unknown_operation_type_dropped_with_warning() {
        let registry = Registry::new();
        let json = r#"{
            "pipeline": {
                "operations": [
                    { "id": "x1", "type": "futureType", "enabled": true, "params": {} }
                ],
                "lift": []
            }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        assert!(result.preset.pipeline.operations.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings[0],
            "Unknown operation type \"futureType\" - layer dropped."
        );
    }

    #[test]
    fn unknown_lift_type_dropped_with_warning() {
        let registry = Registry::new();
        let json = r#"{
            "pipeline": {
                "operations": [],
                "lift": [
                    { "id": "x2", "type": "futureDisp", "enabled": true, "params": {} }
                ]
            }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        assert!(result.preset.pipeline.lift.is_empty());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings[0],
            "Unknown lift type \"futureDisp\" - layer dropped."
        );
    }

    #[test]
    fn missing_param_filled_from_registry_defaults() {
        let registry = Registry::new();
        // rotate has "theta" param, but we provide none
        let json = r#"{
            "pipeline": {
                "operations": [
                    { "id": "r1", "type": "rotate", "enabled": true, "params": {} }
                ],
                "lift": []
            }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        assert_eq!(result.preset.pipeline.operations.len(), 1);
        let layer = &result.preset.pipeline.operations[0];
        assert!(layer.params.contains_key("theta"));
        let theta = &layer.params["theta"];
        assert_eq!(theta.value, 0.0);
        assert_eq!(theta.min, 0.0);
        assert_eq!(theta.max, std::f64::consts::PI * 2.0);
        assert_eq!(theta.shape, crate::types::ShapeFunction::Cosine);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn orphaned_param_removed_with_warning() {
        let registry = Registry::new();
        let json = r#"{
            "pipeline": {
                "operations": [],
                "lift": [
                    {
                        "id": "m1", "type": "zMagnitude", "enabled": true,
                        "params": {
                            "amp": { "value": 1.0, "min": -2, "max": 2, "anim": false, "shape": "cosine", "ratio": 1 },
                            "oldParam": { "value": 0, "min": 0, "max": 1, "anim": false, "shape": "cosine", "ratio": 1 }
                        }
                    }
                ]
            }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        let layer = &result.preset.pipeline.lift[0];
        assert!(layer.params.contains_key("amp"));
        assert!(!layer.params.contains_key("oldParam"));
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings[0],
            "lift \"zMagnitude\" has unknown param \"oldParam\" - dropped."
        );
    }

    #[test]
    fn empty_json_object_produces_valid_default_preset() {
        let registry = Registry::new();
        let result = import_preset_json("{}", &registry).unwrap();
        assert_eq!(result.preset.metadata.name, "Untitled");
        assert!(result.preset.pipeline.operations.is_empty());
        assert!(result.preset.pipeline.lift.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn invalid_json_returns_error() {
        let registry = Registry::new();
        let result = import_preset_json("not json at all", &registry);
        assert!(result.is_err());
        match result.unwrap_err() {
            ImportError::InvalidJson(_) => {}
            other => panic!("Expected InvalidJson, got {:?}", other),
        }
    }

    #[test]
    fn layer_ids_preserved_across_roundtrip() {
        let registry = Registry::new();
        let json = r#"{
            "pipeline": {
                "operations": [
                    {
                        "id": "my-custom-id-123",
                        "type": "rotate",
                        "enabled": true,
                        "params": {
                            "theta": { "value": 1.5, "min": 0, "max": 6.283185307179586, "anim": false, "shape": "sawtooth", "ratio": 1 }
                        }
                    }
                ],
                "lift": []
            }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        assert_eq!(result.preset.pipeline.operations[0].id, "my-custom-id-123");

        // Re-export and re-import
        let re_exported = export_preset_json(&result.preset, false);
        let result2 = import_preset_json(&re_exported, &registry).unwrap();
        assert_eq!(
            result2.preset.pipeline.operations[0].id,
            "my-custom-id-123"
        );
    }

    #[test]
    fn make_layer_produces_correct_defaults() {
        let registry = Registry::new();

        // Operation
        let layer = make_layer("zPow", &registry).unwrap();
        assert_eq!(layer.type_key, "zPow");
        assert!(layer.enabled);
        assert_eq!(layer.blend.value, 1.0);
        assert_eq!(layer.blend.min, 0.0);
        assert_eq!(layer.blend.max, 1.0);
        assert!(layer.params.contains_key("n"));
        assert_eq!(layer.params["n"].value, 2.0);
        assert_eq!(layer.params["n"].min, 0.1);
        assert_eq!(layer.params["n"].max, 5.0);
        assert!(!layer.id.is_empty());

        // Lift
        let layer = make_layer("zSineWave", &registry).unwrap();
        assert_eq!(layer.type_key, "zSineWave");
        assert!(layer.params.contains_key("amp"));
        assert!(layer.params.contains_key("freq"));
        assert!(layer.params.contains_key("phase"));
        assert_eq!(layer.params["phase"].shape, crate::types::ShapeFunction::Cosine);

        // Unknown
        assert!(make_layer("nonExistent", &registry).is_none());
    }

    #[test]
    fn tolerant_validation_ignores_unknown_top_level_fields() {
        let registry = Registry::new();
        // Unknown fields like tempo, palette, schema, version should be silently ignored
        let json = r#"{
            "tempo": { "value": 0.5, "min": 0, "max": 2, "anim": false, "shape": "cosine", "ratio": 1 },
            "palette": "acid",
            "schema": "polyform.preset",
            "version": 1,
            "metadata": { "name": "Test" }
        }"#;

        let result = import_preset_json(json, &registry).unwrap();
        assert_eq!(result.preset.metadata.name, "Test");
        // No warnings — old fields are silently ignored
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn reconciliation_fixture() {
        let registry = Registry::new();
        let json = include_str!("../tests/fixtures/reconciliation-test.json");
        let result = import_preset_json(json, &registry).unwrap();

        // The fixture has: rotate (valid, missing theta), futureType (unknown), zMagnitude (valid, orphaned "oldParam")
        // After reconciliation:
        // - rotate kept, theta filled from registry
        // - futureType dropped
        // - zMagnitude kept, oldParam dropped, amp preserved

        assert_eq!(result.preset.pipeline.operations.len(), 1);
        assert_eq!(result.preset.pipeline.operations[0].type_key, "rotate");
        assert!(result.preset.pipeline.operations[0].params.contains_key("theta"));

        assert_eq!(result.preset.pipeline.lift.len(), 1);
        assert_eq!(result.preset.pipeline.lift[0].type_key, "zMagnitude");
        assert!(result.preset.pipeline.lift[0].params.contains_key("amp"));
        assert!(!result.preset.pipeline.lift[0].params.contains_key("oldParam"));

        // Check warnings
        assert!(result
            .warnings
            .iter()
            .any(|w| w == "Unknown operation type \"futureType\" - layer dropped."));
        assert!(result
            .warnings
            .iter()
            .any(|w| w == "lift \"zMagnitude\" has unknown param \"oldParam\" - dropped."));
    }

    #[test]
    fn preset_file_full_roundtrip() {
        let preset = Preset::default();
        let file = PresetFile::from_preset(&preset, PresetKind::Full);
        let json = export_preset_file_json(&file, true);
        let parsed = import_preset_file_json(&json).unwrap();
        assert_eq!(parsed.kind(), PresetKind::Full);
        if let PresetFile::Full(p) = parsed {
            assert_eq!(p.metadata.name, "Untitled");
        } else {
            panic!("Expected Full variant");
        }
    }

    #[test]
    fn preset_file_section_roundtrip() {
        let registry = Registry::new();
        let mut preset = Preset::default();
        preset.pipeline.operations.push(make_layer("rotate", &registry).unwrap());

        // Operations section
        let file = PresetFile::from_preset(&preset, PresetKind::Operations);
        let json = export_preset_file_json(&file, false);
        let parsed = import_preset_file_json(&json).unwrap();
        assert_eq!(parsed.kind(), PresetKind::Operations);
        if let PresetFile::Operations { layers } = parsed {
            assert_eq!(layers.len(), 1);
            assert_eq!(layers[0].type_key, "rotate");
        } else {
            panic!("Expected Operations variant");
        }

        // Camera section
        let file = PresetFile::from_preset(&preset, PresetKind::Camera);
        let json = export_preset_file_json(&file, false);
        let parsed = import_preset_file_json(&json).unwrap();
        assert_eq!(parsed.kind(), PresetKind::Camera);

        // Phosphor section
        let file = PresetFile::from_preset(&preset, PresetKind::Phosphor);
        let json = export_preset_file_json(&file, false);
        let parsed = import_preset_file_json(&json).unwrap();
        assert_eq!(parsed.kind(), PresetKind::Phosphor);
    }

    #[test]
    fn preset_file_apply_to_preset() {
        let registry = Registry::new();
        let mut preset = Preset::default();

        // Create a section preset with a layer
        let mut ops_preset = Preset::default();
        ops_preset.pipeline.operations.push(make_layer("rotate", &registry).unwrap());
        let file = PresetFile::from_preset(&ops_preset, PresetKind::Operations);

        // Apply to empty preset
        assert!(preset.pipeline.operations.is_empty());
        file.apply_to_preset(&mut preset);
        assert_eq!(preset.pipeline.operations.len(), 1);
        assert_eq!(preset.pipeline.operations[0].type_key, "rotate");
    }
}
