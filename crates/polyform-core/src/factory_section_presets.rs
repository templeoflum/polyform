use crate::preset_io::{PresetFile, import_preset_file_json};

/// Return all built-in factory section presets as (name, PresetFile) pairs.
/// Section presets are parsed from embedded JSON with a `kind` tag.
pub fn factory_section_presets() -> Vec<(String, PresetFile)> {
    let mut presets = Vec::new();

    for (name, json) in SECTION_PRESET_SOURCES {
        match import_preset_file_json(json) {
            Ok(file) => {
                presets.push((name.to_string(), file));
            }
            Err(e) => {
                eprintln!("Failed to parse factory section preset \"{name}\": {e}");
            }
        }
    }

    presets
}

/// (name, json) pairs for all factory section presets.
const SECTION_PRESET_SOURCES: &[(&str, &str)] = &[
    // =========================================================================
    // Operations (15)
    // =========================================================================
    ("slow fold", SLOW_FOLD),
    ("breathing", BREATHING),
    ("whirlpool", WHIRLPOOL),
    ("ripple pool", RIPPLE_POOL),
    ("mirror hall", MIRROR_HALL),
    ("drifting grid", DRIFTING_GRID),
    ("tidal", TIDAL),
    ("singularity", SINGULARITY),
    ("conformal stack", CONFORMAL_STACK),
    ("fabric", FABRIC),
    ("helix", HELIX),
    ("flutter", FLUTTER),
    ("tile cascade", TILE_CASCADE),
    ("liquid", LIQUID),
    ("shatter", SHATTER),
    // =========================================================================
    // Lift (10)
    // =========================================================================
    ("terrain", TERRAIN),
    ("ocean", OCEAN),
    ("mountain", MOUNTAIN),
    ("helix wrap", HELIX_WRAP),
    ("torus", TORUS),
    ("sphere", SPHERE),
    ("noise field", NOISE_FIELD),
    ("angular", ANGULAR),
    ("jacobian", JACOBIAN),
    ("staircase", STAIRCASE),
    // =========================================================================
    // Position (4)
    // =========================================================================
    ("orbit", ORBIT),
    ("flythrough", FLYTHROUGH),
    ("top down", TOP_DOWN),
    ("barrel roll", BARREL_ROLL),
    // =========================================================================
    // Lens (3)
    // =========================================================================
    ("anamorphic", ANAMORPHIC),
    ("vertigo", VERTIGO),
    ("telephoto", TELEPHOTO),
    // =========================================================================
    // Distortion (3)
    // =========================================================================
    ("fisheye", FISHEYE),
    ("barrel", BARREL),
    ("pincushion", PINCUSHION),
    // =========================================================================
    // Quaternion (3)
    // =========================================================================
    ("tumble", TUMBLE),
    ("slow drift", SLOW_DRIFT),
    ("wobble", WOBBLE),
    // =========================================================================
    // Phosphor (6)
    // =========================================================================
    ("clean", CLEAN),
    ("afterglow", AFTERGLOW),
    ("neon", NEON),
    ("crt", CRT),
    ("stark", STARK),
    ("ethereal", ETHEREAL),
    // =========================================================================
    // Sampling (6)
    // =========================================================================
    ("fine cartesian", FINE_CARTESIAN),
    ("polar grid", POLAR_GRID),
    ("hex mesh", HEX_MESH),
    ("log spiral", LOG_SPIRAL),
    ("bipolar field", BIPOLAR_FIELD),
    ("coarse", COARSE),
];

// =============================================================================
// Operations (15)
// =============================================================================

const SLOW_FOLD: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "sf-1", "type": "zPow", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "n": { "value": 2.5, "min": 1.5, "max": 3.5, "anim": true, "ratio": 0.25, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "sf-2", "type": "rotate", "enabled": true,
            "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 }
            }
        }
    ]
}"#;

const BREATHING: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "br-1", "type": "scale", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "sx": { "value": 1.0, "min": 0.6, "max": 1.6, "anim": true, "ratio": 0.5, "offset": 0 },
                "sy": { "value": 1.0, "min": 0.7, "max": 1.4, "anim": true, "ratio": 0.618, "offset": 0.25 }
            }
        }
    ]
}"#;

const WHIRLPOOL: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "wp-1", "type": "spiral", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "k": { "value": 2.0, "min": -4, "max": 4, "anim": true, "ratio": 0.5, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "wp-2", "type": "pinch", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "p": { "value": 1.5, "min": 0.5, "max": 3.0, "anim": true, "ratio": 0.382, "offset": 0.5 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const RIPPLE_POOL: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "rp-1", "type": "ripple", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.1, "min": 0, "max": 0.3, "anim": false, "ratio": 1 },
                "freq": { "value": 6.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "rp-2", "type": "radialSine", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.08, "min": 0, "max": 0.5, "anim": false, "ratio": 1 },
                "freq": { "value": 8.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.618, "offset": 0.33 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const MIRROR_HALL: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "mh-1", "type": "kaleidoscope", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "n": { "value": 8.0, "min": 2, "max": 16, "anim": false, "ratio": 1 },
                "rotation": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "mh-2", "type": "angleWarp", "enabled": true,
            "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.3, "min": 0, "max": 1.5, "anim": true, "ratio": 0.618, "offset": 0.5 },
                "k": { "value": 4.0, "min": 1, "max": 12, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const DRIFTING_GRID: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "dg-1", "type": "quantize", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "step_x": { "value": 0.3, "min": 0.05, "max": 1.0, "anim": true, "ratio": 0.25, "offset": 0 },
                "step_y": { "value": 0.3, "min": 0.05, "max": 1.0, "anim": true, "ratio": 0.382, "offset": 0.25 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "dg-2", "type": "translate", "enabled": true,
            "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "tx": { "value": 0.0, "min": -0.5, "max": 0.5, "anim": true, "ratio": 0.618, "offset": 0 },
                "ty": { "value": 0.0, "min": -0.5, "max": 0.5, "anim": true, "ratio": 1.0, "offset": 0.5 }
            }
        }
    ]
}"#;

const TIDAL: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "td-1", "type": "sineWarp", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.12, "min": 0.02, "max": 0.4, "anim": true, "ratio": 0.5, "offset": 0 },
                "freq": { "value": 3.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 }
            }
        },
        {
            "id": "td-2", "type": "noise", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.1, "min": 0, "max": 0.3, "anim": false, "ratio": 1 },
                "freq": { "value": 4.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0.33 }
            }
        }
    ]
}"#;

const SINGULARITY: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "sg-1", "type": "zInv", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "cx": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.382, "offset": 0 },
                "cy": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.618, "offset": 0.25 }
            }
        },
        {
            "id": "sg-2", "type": "spiral", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "k": { "value": 1.0, "min": -3, "max": 3, "anim": true, "ratio": 0.25, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const CONFORMAL_STACK: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "cs-1", "type": "zExp", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "cx": { "value": 0.0, "min": -1.5, "max": 1.5, "anim": true, "ratio": 0.382, "offset": 0 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "cs-2", "type": "zSin", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "cx": { "value": 0.2, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "cs-3", "type": "rotate", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0.5 }
            }
        }
    ]
}"#;

const FABRIC: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "fb-1", "type": "sineWarp", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.1, "min": 0, "max": 0.4, "anim": false, "ratio": 1 },
                "freq": { "value": 6.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0 }
            }
        },
        {
            "id": "fb-2", "type": "wrap", "enabled": true,
            "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "width": { "value": 2.5, "min": 0.1, "max": 8, "anim": false, "ratio": 1 },
                "height": { "value": 2.5, "min": 0.1, "max": 8, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "fb-3", "type": "noise", "enabled": true,
            "blend": { "value": 0.4, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.08, "min": 0, "max": 0.3, "anim": false, "ratio": 1 },
                "freq": { "value": 5.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 }
            }
        }
    ]
}"#;

const HELIX: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "hx-1", "type": "spiral", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "k": { "value": 2.0, "min": -5, "max": 5, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "hx-2", "type": "rotate", "enabled": true,
            "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0, "shape": "sawtooth" }
            }
        },
        {
            "id": "hx-3", "type": "swirl", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "k": { "value": 3.0, "min": -6, "max": 6, "anim": true, "ratio": 0.618, "offset": 0.25 },
                "sigma": { "value": 0.6, "min": 0.05, "max": 2, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const FLUTTER: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "fl-1", "type": "zSin", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "cx": { "value": 0.0, "min": -1.5, "max": 1.5, "anim": true, "ratio": 0.5, "offset": 0 },
                "cy": { "value": 0.0, "min": -1.5, "max": 1.5, "anim": true, "ratio": 0.618, "offset": 0.25 }
            }
        },
        {
            "id": "fl-2", "type": "zCos", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "cx": { "value": 0.0, "min": -1.5, "max": 1.5, "anim": true, "ratio": 0.382, "offset": 0.5 },
                "cy": { "value": 0.0, "min": -1.5, "max": 1.5, "anim": true, "ratio": 0.25, "offset": 0.75 }
            }
        }
    ]
}"#;

const TILE_CASCADE: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "tc-1", "type": "wrap", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "width": { "value": 2.0, "min": 0.5, "max": 5.0, "anim": true, "ratio": 0.25, "offset": 0 },
                "height": { "value": 2.0, "min": 0.5, "max": 5.0, "anim": true, "ratio": 0.382, "offset": 0.25 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "tc-2", "type": "rotate", "enabled": true,
            "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 }
            }
        },
        {
            "id": "tc-3", "type": "quantize", "enabled": true,
            "blend": { "value": 0.3, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "step_x": { "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
                "step_y": { "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const LIQUID: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "lq-1", "type": "joukowski", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "a": { "value": 0.5, "min": 0.1, "max": 1.5, "anim": true, "ratio": 0.382, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "lq-2", "type": "pinch", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "p": { "value": 1.3, "min": 0.5, "max": 3, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "lq-3", "type": "noise", "enabled": true,
            "blend": { "value": 0.4, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.08, "min": 0, "max": 0.3, "anim": false, "ratio": 1 },
                "freq": { "value": 3.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0.5 }
            }
        }
    ]
}"#;

const SHATTER: &str = r#"{
    "kind": "operations",
    "layers": [
        {
            "id": "sh-1", "type": "clamp", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "radius": { "value": 2.5, "min": 0.5, "max": 5, "anim": false, "ratio": 1 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "sh-2", "type": "kaleidoscope", "enabled": true,
            "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "n": { "value": 6.0, "min": 2, "max": 16, "anim": false, "ratio": 1 },
                "rotation": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 },
                "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "sh-3", "type": "scale", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "sx": { "value": 1.0, "min": 0.5, "max": 2.0, "anim": true, "ratio": 0.382, "offset": 0.25 },
                "sy": { "value": 1.0, "min": 0.5, "max": 2.0, "anim": true, "ratio": 0.618, "offset": 0.5 }
            }
        }
    ]
}"#;

// =============================================================================
// Lift (10)
// =============================================================================

const TERRAIN: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "tr-1", "type": "zMagnitude", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.6, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "tr-2", "type": "zSaddle", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const OCEAN: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "oc-1", "type": "zSineWave", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.4, "min": -1.5, "max": 1.5, "anim": false, "ratio": 1 },
                "freq": { "value": 3.0, "min": 0.1, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 }
            }
        },
        {
            "id": "oc-2", "type": "ripple3D", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.15, "min": -1, "max": 1, "anim": false, "ratio": 1 },
                "freq": { "value": 5.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0.33 }
            }
        }
    ]
}"#;

const MOUNTAIN: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "mt-1", "type": "zGaussian", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 1.2, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "sigma": { "value": 0.8, "min": 0.05, "max": 3, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "mt-2", "type": "zTerrace", "enabled": true,
            "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.5, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "steps": { "value": 8.0, "min": 1, "max": 20, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const HELIX_WRAP: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "hw-1", "type": "helical", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "lift": { "value": 0.6, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "twist": { "value": 3.0, "min": -8, "max": 8, "anim": true, "ratio": 0.25, "offset": 0 }
            }
        }
    ]
}"#;

const TORUS: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "to-1", "type": "toroidal", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "major": { "value": 1.5, "min": 0.5, "max": 4, "anim": false, "ratio": 1 },
                "minor": { "value": 0.5, "min": 0.1, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const SPHERE: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "sp-1", "type": "spherical", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "r": { "value": 1.5, "min": 0.5, "max": 3, "anim": true, "ratio": 0.25, "offset": 0 }
            }
        }
    ]
}"#;

const NOISE_FIELD: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "nf-1", "type": "noise3D", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.3, "min": -0.8, "max": 0.8, "anim": true, "ratio": 0.618, "offset": 0.25 },
                "freq": { "value": 4.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 }
            }
        }
    ]
}"#;

const ANGULAR: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "an-1", "type": "conical", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.6, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "an-2", "type": "zArg", "enabled": true,
            "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const JACOBIAN: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "jc-1", "type": "zLogJac", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.4, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        },
        {
            "id": "jc-2", "type": "zSignedJac", "enabled": true,
            "blend": { "value": 0.4, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.2, "min": -2, "max": 2, "anim": false, "ratio": 1 }
            }
        }
    ]
}"#;

const STAIRCASE: &str = r#"{
    "kind": "lift",
    "layers": [
        {
            "id": "st-1", "type": "zTerrace", "enabled": true,
            "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.5, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                "steps": { "value": 5.0, "min": 2, "max": 15, "anim": true, "ratio": 0.25, "offset": 0 }
            }
        },
        {
            "id": "st-2", "type": "zSineWave", "enabled": true,
            "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
            "params": {
                "amp": { "value": 0.3, "min": -1.5, "max": 1.5, "anim": false, "ratio": 1 },
                "freq": { "value": 3.0, "min": 0.1, "max": 20, "anim": false, "ratio": 1 },
                "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0.5 }
            }
        }
    ]
}"#;

// =============================================================================
// Position (4)
// =============================================================================

const ORBIT: &str = r#"{
    "kind": "position",
    "position": {
        "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
        "orbitY": { "on": true, "value": 0.5, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.382, "offset": 0 },
        "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
        "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
    }
}"#;

const FLYTHROUGH: &str = r#"{
    "kind": "position",
    "position": {
        "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
        "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
        "distance": { "on": true, "value": 3.0, "min": 1.5, "max": 8.0, "anim": true, "ratio": 0.25, "offset": 0 },
        "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
    }
}"#;

const TOP_DOWN: &str = r#"{
    "kind": "position",
    "position": {
        "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 },
        "orbitY": { "on": true, "value": 0.05, "min": 0.01, "max": 0.3, "anim": false, "ratio": 1 },
        "distance": { "on": true, "value": 5.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
        "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "roll": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 }
    }
}"#;

const BARREL_ROLL: &str = r#"{
    "kind": "position",
    "position": {
        "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
        "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
        "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
        "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
        "roll": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0, "shape": "sawtooth" }
    }
}"#;

// =============================================================================
// Lens (3)
// =============================================================================

const ANAMORPHIC: &str = r#"{
    "kind": "lens",
    "lens": {
        "projection": "perspective",
        "focalLength": { "on": true, "value": 35, "min": 8, "max": 200, "anim": false, "ratio": 1 },
        "sensorSize": { "on": true, "value": 36, "min": 4, "max": 80, "anim": false, "ratio": 1 },
        "orthoZoom": { "on": true, "value": 2, "min": 0.3, "max": 6, "anim": false, "ratio": 1 },
        "anamorphic": { "on": true, "value": 1.5, "min": 0.5, "max": 2.5, "anim": true, "ratio": 0.382, "offset": 0 },
        "shiftX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "shiftY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 }
    }
}"#;

const VERTIGO: &str = r#"{
    "kind": "lens",
    "lens": {
        "projection": "perspective",
        "focalLength": { "on": true, "value": 50, "min": 15, "max": 120, "anim": true, "ratio": 0.382, "offset": 0.5 },
        "sensorSize": { "on": true, "value": 36, "min": 4, "max": 80, "anim": false, "ratio": 1 },
        "orthoZoom": { "on": true, "value": 2, "min": 0.3, "max": 6, "anim": false, "ratio": 1 },
        "anamorphic": { "on": false, "value": 1, "min": 0.3, "max": 3, "anim": false, "ratio": 1 },
        "shiftX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "shiftY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 }
    }
}"#;

const TELEPHOTO: &str = r#"{
    "kind": "lens",
    "lens": {
        "projection": "perspective",
        "focalLength": { "on": true, "value": 135, "min": 85, "max": 200, "anim": false, "ratio": 1 },
        "sensorSize": { "on": true, "value": 36, "min": 4, "max": 80, "anim": false, "ratio": 1 },
        "orthoZoom": { "on": true, "value": 2, "min": 0.3, "max": 6, "anim": false, "ratio": 1 },
        "anamorphic": { "on": false, "value": 1, "min": 0.3, "max": 3, "anim": false, "ratio": 1 },
        "shiftX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "shiftY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 }
    }
}"#;

// =============================================================================
// Distortion (3)
// =============================================================================

const FISHEYE: &str = r#"{
    "kind": "distortion",
    "distortion": {
        "k1": { "on": true, "value": 0.6, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k2": { "on": true, "value": 0.3, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k3": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k4": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "p1": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "p2": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "centerX": { "on": true, "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.618, "offset": 0 },
        "centerY": { "on": true, "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.382, "offset": 0.25 },
        "squeeze": { "on": false, "value": 1, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const BARREL: &str = r#"{
    "kind": "distortion",
    "distortion": {
        "k1": { "on": true, "value": 0.4, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k2": { "on": true, "value": 0.15, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k3": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k4": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "p1": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "p2": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "centerX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "centerY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "squeeze": { "on": false, "value": 1, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const PINCUSHION: &str = r#"{
    "kind": "distortion",
    "distortion": {
        "k1": { "on": true, "value": -0.4, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k2": { "on": true, "value": -0.15, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k3": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "k4": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "p1": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "p2": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
        "centerX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "centerY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "squeeze": { "on": false, "value": 1, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

// =============================================================================
// Quaternion (3)
// =============================================================================

const TUMBLE: &str = r#"{
    "kind": "quaternion",
    "quaternion": {
        "on": true,
        "mode": "drift",
        "angle": { "value": 0.5, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1, "offset": 0 },
        "ax": { "value": 0.4, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "ay": { "value": 0.8, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "az": { "value": 0.3, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "driftSpeed": 0.06
    }
}"#;

const SLOW_DRIFT: &str = r#"{
    "kind": "quaternion",
    "quaternion": {
        "on": true,
        "mode": "drift",
        "angle": { "value": 0.3, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 },
        "ax": { "value": 0.2, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "ay": { "value": 1.0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "az": { "value": 0.5, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "driftSpeed": 0.02
    }
}"#;

const WOBBLE: &str = r#"{
    "kind": "quaternion",
    "quaternion": {
        "on": true,
        "mode": "fixed",
        "angle": { "value": 0.0, "min": 0, "max": 0.6, "anim": true, "ratio": 0.618, "offset": 0 },
        "ax": { "value": 0.3, "min": -1, "max": 1, "anim": true, "ratio": 0.382, "offset": 0.25 },
        "ay": { "value": 0.7, "min": -1, "max": 1, "anim": true, "ratio": 0.25, "offset": 0.5 },
        "az": { "value": 0.5, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "driftSpeed": 0.1
    }
}"#;

// =============================================================================
// Phosphor (6)
// =============================================================================

const CLEAN: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": false, "value": 0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 1.5, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": false, "value": 0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.0, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": false, "value": 0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": false, "value": 2, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": false, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 1.0, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const AFTERGLOW: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": true, "value": 0.85, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 2.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.2, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.4, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": false, "value": 0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": false, "value": 2, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": false, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 0.9, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const NEON: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": true, "value": 0.2, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 2.5, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.8, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.6, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": true, "value": 1.2, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": true, "value": 4.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": true, "value": 0.15, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 0.85, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const CRT: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": true, "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 3.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.3, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.3, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.1, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": true, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": true, "value": 2.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": true, "value": 0.4, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 1.1, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const STARK: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": false, "value": 0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 1.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": false, "value": 0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 2.0, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": false, "value": 0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": false, "value": 2, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": false, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 0.7, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

const ETHEREAL: &str = r#"{
    "kind": "phosphor",
    "phosphor": {
        "persistence": { "on": true, "value": 0.9, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 2.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.15, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.2, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 0.7, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": true, "value": 1.0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": true, "value": 5.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": true, "value": 0.1, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 1.3, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    }
}"#;

// =============================================================================
// Sampling (6)
// =============================================================================

const FINE_CARTESIAN: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 40, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const POLAR_GRID: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "polar",
        "latticeParams": {
            "spokeCount": 32,
            "innerRadius": 0.05,
            "focusDistance": 1.0,
            "tauCount": 16
        }
    }
}"#;

const HEX_MESH: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "hex"
    }
}"#;

const LOG_SPIRAL: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "logpolar",
        "latticeParams": {
            "spokeCount": 24,
            "innerRadius": 0.05,
            "focusDistance": 1.0,
            "tauCount": 16
        }
    }
}"#;

const BIPOLAR_FIELD: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "bipolar",
        "latticeParams": {
            "spokeCount": 24,
            "innerRadius": 0.05,
            "focusDistance": 0.6,
            "tauCount": 16
        }
    }
}"#;

const COARSE: &str = r#"{
    "kind": "sampling",
    "sampling": {
        "scale": { "on": true, "value": 1.0, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 15, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preset_io::{PresetKind, export_preset_file_json};

    #[test]
    fn all_factory_section_presets_parse() {
        let presets = factory_section_presets();
        assert_eq!(presets.len(), 50);
        for (name, file) in &presets {
            assert!(!name.is_empty(), "preset has empty name");
            // Verify the kind is not Full (section presets only)
            assert_ne!(
                file.kind(),
                PresetKind::Full,
                "section preset \"{name}\" should not be Full"
            );
        }
    }

    #[test]
    fn factory_section_presets_kind_counts() {
        let presets = factory_section_presets();
        let ops = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Operations).count();
        let lift = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Lift).count();
        let pos = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Position).count();
        let lens = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Lens).count();
        let dist = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Distortion).count();
        let quat = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Quaternion).count();
        let phos = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Phosphor).count();
        let samp = presets.iter().filter(|(_, f)| f.kind() == PresetKind::Sampling).count();
        assert_eq!(ops, 15, "expected 15 operation presets");
        assert_eq!(lift, 10, "expected 10 lift presets");
        assert_eq!(pos, 4, "expected 4 position presets");
        assert_eq!(lens, 3, "expected 3 lens presets");
        assert_eq!(dist, 3, "expected 3 distortion presets");
        assert_eq!(quat, 3, "expected 3 quaternion presets");
        assert_eq!(phos, 6, "expected 6 phosphor presets");
        assert_eq!(samp, 6, "expected 6 sampling presets");
    }

    #[test]
    fn factory_section_presets_roundtrip() {
        let presets = factory_section_presets();
        for (name, file) in &presets {
            let json = export_preset_file_json(file, false);
            let parsed = import_preset_file_json(&json)
                .unwrap_or_else(|e| panic!("roundtrip failed for \"{name}\": {e}"));
            assert_eq!(
                parsed.kind(),
                file.kind(),
                "kind mismatch on roundtrip for \"{name}\""
            );
        }
    }

    #[test]
    fn operation_presets_have_layers() {
        let presets = factory_section_presets();
        for (name, file) in &presets {
            if let PresetFile::Operations { layers } = file {
                assert!(
                    !layers.is_empty(),
                    "operation preset \"{name}\" has no layers"
                );
            }
        }
    }

    #[test]
    fn lift_presets_have_layers() {
        let presets = factory_section_presets();
        for (name, file) in &presets {
            if let PresetFile::Lift { layers } = file {
                assert!(
                    !layers.is_empty(),
                    "lift preset \"{name}\" has no layers"
                );
            }
        }
    }
}
