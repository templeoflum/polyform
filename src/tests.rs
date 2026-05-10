use crate::preset::{Preset, SCHEMA_NAME, SCHEMA_VERSION};

// =============================================================================
// Round-trip test
// =============================================================================

#[test]
fn roundtrip_default_preset() {
    let original = Preset::default();
    let json = serde_json::to_string_pretty(&original).unwrap();
    let deserialized: Preset = serde_json::from_str(&json).unwrap();
    assert_eq!(original, deserialized);
}

// =============================================================================
// Schema compliance test
// =============================================================================

#[test]
fn schema_compliance() {
    let preset = Preset::default();
    let value: serde_json::Value = serde_json::to_value(&preset).unwrap();
    let obj = value.as_object().unwrap();

    // Top-level keys
    let expected_keys = [
        "schema", "version", "metadata", "tempo", "defaults",
        "palette", "pipeline", "camera", "quaternion", "sampling", "display",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "missing top-level key: {key}");
    }

    // Schema and version
    assert_eq!(obj["schema"].as_str().unwrap(), SCHEMA_NAME);
    assert_eq!(obj["version"].as_u64().unwrap(), SCHEMA_VERSION as u64);

    // Pipeline structure
    let pipeline = obj["pipeline"].as_object().unwrap();
    assert!(pipeline.contains_key("operations"));
    assert!(pipeline.contains_key("lift"));
    assert!(pipeline["operations"].as_array().unwrap().is_empty());
    assert!(pipeline["lift"].as_array().unwrap().is_empty());

    // Camera structure
    let camera = obj["camera"].as_object().unwrap();
    assert!(camera.contains_key("position"));
    assert!(camera.contains_key("lens"));
    assert!(camera.contains_key("distortion"));

    // No snake_case keys anywhere
    check_no_snake_case(&value, "");
}

fn check_no_snake_case(value: &serde_json::Value, path: &str) {
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let full_path = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            assert!(
                !key.contains('_'),
                "snake_case key found at {full_path}: \"{key}\""
            );
            check_no_snake_case(val, &full_path);
        }
    } else if let Some(arr) = value.as_array() {
        for (i, val) in arr.iter().enumerate() {
            check_no_snake_case(val, &format!("{path}[{i}]"));
        }
    }
}

// =============================================================================
// Default values match JS
// =============================================================================

#[test]
fn default_values_match_js() {
    let p = Preset::default();

    // Schema
    assert_eq!(p.schema, "polyform.preset");
    assert_eq!(p.version, 1);

    // Metadata
    assert_eq!(p.metadata.name, "Untitled");
    assert_eq!(p.metadata.description, "");
    assert_eq!(p.metadata.author, "");
    assert!(p.metadata.tags.is_empty());

    // Tempo
    assert_eq!(p.tempo.value, 0.3);
    assert_eq!(p.tempo.min, 0.0);
    assert_eq!(p.tempo.max, 2.0);
    assert!(!p.tempo.anim);

    // Defaults
    assert_eq!(p.defaults.blend_time, 1.0);

    // Palette
    assert_eq!(p.palette, "acid");

    // Pipeline
    assert!(p.pipeline.operations.is_empty());
    assert!(p.pipeline.lift.is_empty());

    // Camera position
    assert!(p.camera.position.orbit_x.on);
    assert_eq!(p.camera.position.orbit_x.param.value, 0.0);
    assert_eq!(p.camera.position.orbit_x.param.max, std::f64::consts::TAU);

    assert!(p.camera.position.orbit_y.on);
    assert_eq!(p.camera.position.orbit_y.param.value, 0.1);
    let inv_phi = 1.0 / crate::types::PHI;
    assert!((p.camera.position.orbit_y.param.ratio - inv_phi).abs() < 1e-15);

    assert!(p.camera.position.distance.on);
    assert_eq!(p.camera.position.distance.param.value, 3.5);

    assert!(!p.camera.position.target_x.on);
    assert!(!p.camera.position.target_y.on);
    assert!(!p.camera.position.roll.on);

    // Camera lens
    assert!(p.camera.lens.focal_length.on);
    assert_eq!(p.camera.lens.focal_length.param.value, 35.0);
    assert!(p.camera.lens.sensor_size.on);
    assert_eq!(p.camera.lens.sensor_size.param.value, 36.0);

    // Distortion
    assert!(!p.camera.distortion.k1.on);
    assert!(!p.camera.distortion.k2.on);

    // Quaternion
    assert!(!p.quaternion.on);
    assert!(p.quaternion.anim);
    assert_eq!(p.quaternion.angle, 0.0);
    assert_eq!(p.quaternion.axes.ax.value, 0.5);
    assert!((p.quaternion.axes.ax.ratio - crate::types::PHI).abs() < 1e-15);
    assert_eq!(p.quaternion.axes.ay.value, 1.0);
    assert_eq!(p.quaternion.axes.az.value, 0.3);
    assert!((p.quaternion.axes.az.ratio - inv_phi).abs() < 1e-15);
    assert_eq!(p.quaternion.drift_speed, 0.1);

    // Sampling
    assert!(p.sampling.scale.on);
    assert_eq!(p.sampling.scale.param.value, 0.6);
    assert!(p.sampling.density.on);
    assert_eq!(p.sampling.density.param.value, 27.0);

    // Display
    assert!(p.display.show_formula);
    assert!(p.display.show_dim_count);
    assert!(p.display.show_active_layers);
    assert!(p.display.show_param_values);
    assert!(!p.display.show_fps);
}

// =============================================================================
// Cross-format test: deserialize a JS-exported preset
// =============================================================================

#[test]
fn deserialize_js_fixture() {
    let json = include_str!("../tests/fixtures/js-export.json");
    let preset: Preset = serde_json::from_str(json).unwrap();

    // Verify basic fields
    assert_eq!(preset.schema, "polyform.preset");
    assert_eq!(preset.version, 1);
    assert_eq!(preset.metadata.name, "Test Fixture");
    assert_eq!(preset.palette, "acid");

    // Verify pipeline layers
    assert_eq!(preset.pipeline.operations.len(), 2);
    assert_eq!(preset.pipeline.operations[0].type_key, "rotate");
    assert_eq!(preset.pipeline.operations[0].id, "a1b2c3d4");
    assert!(preset.pipeline.operations[0].enabled);
    assert_eq!(preset.pipeline.operations[1].type_key, "zPow");
    assert_eq!(preset.pipeline.operations[1].id, "e5f6a7b8");

    // Verify layer params
    let theta = &preset.pipeline.operations[0].params["theta"];
    assert_eq!(theta.value, 0.0);
    assert_eq!(theta.max, std::f64::consts::TAU);

    let n = &preset.pipeline.operations[1].params["n"];
    assert_eq!(n.value, 2.0);
    assert_eq!(n.min, 0.1);
    assert_eq!(n.max, 5.0);

    // Verify lift layer
    assert_eq!(preset.pipeline.lift.len(), 1);
    assert_eq!(preset.pipeline.lift[0].type_key, "zMagnitude");
    assert_eq!(preset.pipeline.lift[0].id, "c1d2e3f4");

    // Verify camera values deserialized correctly
    assert!(preset.camera.position.orbit_x.on);
    assert_eq!(preset.camera.position.orbit_y.param.value, 0.1);

    // Verify quaternion
    assert!(!preset.quaternion.on);
    assert!(preset.quaternion.anim);
}

// =============================================================================
// Forward compatibility: unknown fields are silently ignored
// =============================================================================

#[test]
fn unknown_fields_ignored() {
    let json = r#"{
        "schema": "polyform.preset",
        "version": 1,
        "futureField": "should be ignored",
        "metadata": { "name": "Test", "extraStuff": 42 },
        "tempo": { "value": 0.5, "min": 0, "max": 2, "anim": false, "animMode": "sweep", "ratio": 1 },
        "display": { "showFormula": true, "showDimCount": true, "showActiveLayers": true, "showParamValues": true, "showFps": false, "showNewThing": true }
    }"#;

    let preset: Preset = serde_json::from_str(json).unwrap();
    assert_eq!(preset.metadata.name, "Test");
    assert_eq!(preset.tempo.value, 0.5);
}

// =============================================================================
// Serialization produces valid JSON matching expected structure
// =============================================================================

#[test]
fn serialized_json_structure() {
    let preset = Preset::default();
    let json = serde_json::to_string_pretty(&preset).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    // ToggleParam fields should be flat (on alongside value/min/max)
    let orbit_x = &value["camera"]["position"]["orbitX"];
    assert!(orbit_x["on"].is_boolean());
    assert!(orbit_x["value"].is_number());
    assert!(orbit_x["min"].is_number());
    assert!(orbit_x["max"].is_number());
    assert!(orbit_x["anim"].is_boolean());
    assert!(orbit_x["animMode"].is_string());
    assert!(orbit_x["ratio"].is_number());

    // Layer blend should not have phaseOffset when None
    // (this is for the empty preset which has no layers, so check via a constructed value)
    let blend_json = serde_json::to_value(crate::types::default_blend()).unwrap();
    assert!(blend_json.get("phaseOffset").is_none());
}
