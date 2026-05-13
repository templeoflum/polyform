use crate::preset::Preset;
use crate::preset_io::import_preset_json;
use crate::registry::Registry;

/// Return all built-in factory presets as (name, Preset) pairs.
/// Presets are parsed from embedded JSON and reconciled against the registry.
pub fn factory_presets() -> Vec<(String, Preset)> {
    let registry = Registry::new();
    let mut presets = Vec::new();

    for (name, json) in PRESET_SOURCES {
        match import_preset_json(json, &registry) {
            Ok(result) => {
                presets.push((name.to_string(), result.preset));
            }
            Err(e) => {
                eprintln!("Failed to parse factory preset \"{name}\": {e}");
            }
        }
    }

    presets
}

/// (name, json) pairs for all factory presets.
const PRESET_SOURCES: &[(&str, &str)] = &[
    // =========================================================================
    // Foundations — showcase individual operation families
    // =========================================================================
    ("conformal zoo", CONFORMAL_ZOO),
    ("mobius dance", MOBIUS_DANCE),
    ("spiral galaxy", SPIRAL_GALAXY),
    ("sine lattice", SINE_LATTICE),
    ("kaleidoscope", KALEIDOSCOPE),
    // =========================================================================
    // Complex compositions — multiple operations interacting
    // =========================================================================
    ("crystal", CRYSTAL),
    ("warp field", WARP_FIELD),
    ("pixel drift", PIXEL_DRIFT),
    ("event horizon", EVENT_HORIZON),
    ("reef", REEF),
    // =========================================================================
    // Camera-focused — demonstrate the camera as a performable layer
    // =========================================================================
    ("tunnel", TUNNEL),
    ("flyover", FLYOVER),
    ("lens", LENS),
    // =========================================================================
    // Phosphor / rendering character
    // =========================================================================
    ("glow", GLOW),
    ("strobe", STROBE),
];

// =============================================================================
// Foundations
// =============================================================================

const CONFORMAL_ZOO: &str = r#"{
    "metadata": {
        "name": "conformal zoo",
        "description": "z², z³, exp(z), sin(z) layered with different blends + conical lift",
        "tags": ["foundations", "complex", "conformal"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "cz-1", "type": "zPow", "enabled": true,
                "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "n": { "value": 2.0, "min": 0.1, "max": 5.0, "anim": true, "ratio": 0.5, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "cz-2", "type": "zExp", "enabled": true,
                "blend": { "value": 0.3, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.618, "offset": 0.25 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "cz-3", "type": "zSin", "enabled": true,
                "blend": { "value": 0.4, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "cx": { "value": 0.2, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "cz-l1", "type": "conical", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.4, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.5, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 },
            "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const MOBIUS_DANCE: &str = r#"{
    "metadata": {
        "name": "mobius dance",
        "description": "Mobius transform with animated a,c params + spherical lift",
        "tags": ["foundations", "complex", "mobius"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "md-1", "type": "mobius", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "a": { "value": 1.0, "min": -2, "max": 2, "anim": true, "ratio": 1.0, "offset": 0 },
                    "b": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "c": { "value": 0.3, "min": -2, "max": 2, "anim": true, "ratio": 0.618, "offset": 0.33 },
                    "d": { "value": 1.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "md-2", "type": "rotate", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 }
                }
            }
        ],
        "lift": [
            {
                "id": "md-l1", "type": "spherical", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "r": { "value": 1.5, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.3, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
            "orbitY": { "on": true, "value": 0.8, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 4.5, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const SPIRAL_GALAXY: &str = r#"{
    "metadata": {
        "name": "spiral galaxy",
        "description": "Spiral + pinch + radialSine + zMagnitude lift",
        "tags": ["foundations", "polar", "spiral"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "sg-1", "type": "spiral", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "k": { "value": 2.0, "min": -5, "max": 5, "anim": true, "ratio": 0.5, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "sg-2", "type": "pinch", "enabled": true,
                "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "p": { "value": 1.3, "min": 0.2, "max": 4, "anim": true, "ratio": 0.382, "offset": 0.5 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "sg-3", "type": "radialSine", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.08, "min": 0, "max": 0.5, "anim": false, "ratio": 1 },
                    "freq": { "value": 8.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 2.0, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "sg-l1", "type": "zMagnitude", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.5, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
            "orbitY": { "on": true, "value": 0.5, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 5.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.7, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "polar"
    }
}"#;

const SINE_LATTICE: &str = r#"{
    "metadata": {
        "name": "sine lattice",
        "description": "sineWarp + ripple layered + zSineXY lift",
        "tags": ["foundations", "wave"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "sl-1", "type": "sineWarp", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.12, "min": 0, "max": 0.6, "anim": true, "ratio": 0.5, "offset": 0 },
                    "freq": { "value": 5.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 }
                }
            },
            {
                "id": "sl-2", "type": "ripple", "enabled": true,
                "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.08, "min": 0, "max": 0.5, "anim": false, "ratio": 1 },
                    "freq": { "value": 8.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.618, "offset": 0.5 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "sl-l1", "type": "zSineXY", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.4, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "fx": { "value": 4.0, "min": 0.1, "max": 15, "anim": false, "ratio": 1 },
                    "fy": { "value": 4.0, "min": 0.1, "max": 15, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.4, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
            "orbitY": { "on": true, "value": 0.7, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.25, "offset": 0.15 },
            "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const KALEIDOSCOPE: &str = r#"{
    "metadata": {
        "name": "kaleidoscope",
        "description": "Kaleidoscope(n=8) + rotate with animated theta + zArg lift",
        "tags": ["foundations", "polar", "kaleidoscope"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "kal-1", "type": "kaleidoscope", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "n": { "value": 8.0, "min": 2, "max": 16, "anim": false, "ratio": 1 },
                    "rotation": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "kal-2", "type": "rotate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0 }
                }
            }
        ],
        "lift": [
            {
                "id": "kal-l1", "type": "zArg", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
            "orbitY": { "on": true, "value": 0.4, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 4.5, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "hex"
    }
}"#;

// =============================================================================
// Complex compositions
// =============================================================================

const CRYSTAL: &str = r#"{
    "metadata": {
        "name": "crystal",
        "description": "Kaleidoscope + joukowski + scale + zTerrace lift",
        "tags": ["composition", "kaleidoscope", "joukowski"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "cr-1", "type": "kaleidoscope", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "n": { "value": 6.0, "min": 2, "max": 16, "anim": false, "ratio": 1 },
                    "rotation": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "cr-2", "type": "joukowski", "enabled": true,
                "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "a": { "value": 0.6, "min": 0, "max": 3, "anim": true, "ratio": 0.382, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "cr-3", "type": "scale", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "sx": { "value": 0.9, "min": 0.1, "max": 3, "anim": false, "ratio": 1 },
                    "sy": { "value": 0.9, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "cr-l1", "type": "zTerrace", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.4, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "steps": { "value": 8.0, "min": 1, "max": 20, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.2, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
            "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 5.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "hex"
    }
}"#;

const WARP_FIELD: &str = r#"{
    "metadata": {
        "name": "warp field",
        "description": "Noise + sineWarp + wrap + zGaussian lift",
        "tags": ["composition", "noise", "wave"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "wf-1", "type": "noise", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.15, "min": 0, "max": 0.5, "anim": false, "ratio": 1 },
                    "freq": { "value": 4.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                    "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 }
                }
            },
            {
                "id": "wf-2", "type": "sineWarp", "enabled": true,
                "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.1, "min": 0, "max": 0.6, "anim": false, "ratio": 1 },
                    "freq": { "value": 3.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 }
                }
            },
            {
                "id": "wf-3", "type": "wrap", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "width": { "value": 3.0, "min": 0.1, "max": 8, "anim": false, "ratio": 1 },
                    "height": { "value": 3.0, "min": 0.1, "max": 8, "anim": false, "ratio": 1 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "wf-l1", "type": "zGaussian", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.8, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "sigma": { "value": 0.8, "min": 0.05, "max": 3, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
            "orbitY": { "on": true, "value": 0.5, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.25, "offset": 0 },
            "distance": { "on": true, "value": 4.5, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.7, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const PIXEL_DRIFT: &str = r#"{
    "metadata": {
        "name": "pixel drift",
        "description": "Quantize + rotate + translate animated + conical lift",
        "tags": ["composition", "utility", "quantize"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "pd-1", "type": "quantize", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "step_x": { "value": 0.2, "min": 0.01, "max": 2, "anim": true, "ratio": 0.25, "offset": 0 },
                    "step_y": { "value": 0.2, "min": 0.01, "max": 2, "anim": true, "ratio": 0.382, "offset": 0.5 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "pd-2", "type": "rotate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0, "shape": "sawtooth" }
                }
            },
            {
                "id": "pd-3", "type": "translate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "tx": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.618, "offset": 0 },
                    "ty": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 1.0, "offset": 0.25 }
                }
            }
        ],
        "lift": [
            {
                "id": "pd-l1", "type": "conical", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.3, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
            "orbitY": { "on": true, "value": 0.5, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 25, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const EVENT_HORIZON: &str = r#"{
    "metadata": {
        "name": "event horizon",
        "description": "zInv + spiral + clamp + zLogJac lift",
        "tags": ["composition", "complex", "singularity"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "eh-1", "type": "zInv", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.382, "offset": 0 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.618, "offset": 0.25 }
                }
            },
            {
                "id": "eh-2", "type": "spiral", "enabled": true,
                "blend": { "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "k": { "value": 1.5, "min": -5, "max": 5, "anim": true, "ratio": 0.25, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "eh-3", "type": "clamp", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "radius": { "value": 3.0, "min": 0.1, "max": 10, "anim": false, "ratio": 1 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ],
        "lift": [
            {
                "id": "eh-l1", "type": "zLogJac", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 },
            "orbitY": { "on": true, "value": 0.4, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 5.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.7, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const REEF: &str = r#"{
    "metadata": {
        "name": "reef",
        "description": "zSin + kaleidoscope + noise + spherical lift",
        "tags": ["composition", "complex", "organic"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "rf-1", "type": "zSin", "enabled": true,
                "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.382, "offset": 0 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.618, "offset": 0.33 }
                }
            },
            {
                "id": "rf-2", "type": "kaleidoscope", "enabled": true,
                "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "n": { "value": 5.0, "min": 2, "max": 16, "anim": false, "ratio": 1 },
                    "rotation": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "rf-3", "type": "noise", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.08, "min": 0, "max": 0.5, "anim": false, "ratio": 1 },
                    "freq": { "value": 5.0, "min": 0.5, "max": 15, "anim": false, "ratio": 1 },
                    "seed": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.618, "offset": 0 }
                }
            }
        ],
        "lift": [
            {
                "id": "rf-l1", "type": "spherical", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "r": { "value": 1.8, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.382, "offset": 0 },
            "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 5.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "polar"
    }
}"#;

// =============================================================================
// Camera-focused
// =============================================================================

const TUNNEL: &str = r#"{
    "metadata": {
        "name": "tunnel",
        "description": "Rotate + camera orbit_x animated + high distortion k1 + phosphor persistence",
        "tags": ["camera", "distortion", "phosphor"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "tn-1", "type": "rotate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0, "shape": "sawtooth" }
                }
            },
            {
                "id": "tn-2", "type": "scale", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "sx": { "value": 1.1, "min": 0.1, "max": 3, "anim": true, "ratio": 0.5, "offset": 0 },
                    "sy": { "value": 1.1, "min": 0.1, "max": 3, "anim": true, "ratio": 0.5, "offset": 0 }
                }
            }
        ],
        "lift": []
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0, "shape": "sawtooth" },
            "orbitY": { "on": true, "value": 1.2, "min": 0.15, "max": 2.8, "anim": true, "ratio": 0.618, "offset": 0 },
            "distance": { "on": true, "value": 3.0, "min": 0.8, "max": 12, "anim": true, "ratio": 0.382, "offset": 0 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        },
        "distortion": {
            "k1": { "on": true, "value": 0.4, "min": -1, "max": 1, "anim": true, "ratio": 0.5, "offset": 0 },
            "k2": { "on": true, "value": -0.1, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "k3": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "k4": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "p1": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
            "p2": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
            "centerX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "centerY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "squeeze": { "on": false, "value": 1, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
        }
    },
    "phosphor": {
        "persistence": { "on": true, "value": 0.6, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 2.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": false, "value": 0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.2, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.1, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": false, "value": 0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": false, "value": 2, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": false, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 1, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const FLYOVER: &str = r#"{
    "metadata": {
        "name": "flyover",
        "description": "Scale + helical lift + camera distance animated + quaternion drift",
        "tags": ["camera", "3d", "quaternion"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "fo-1", "type": "scale", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "sx": { "value": 1.0, "min": 0.1, "max": 3, "anim": false, "ratio": 1 },
                    "sy": { "value": 1.0, "min": 0.1, "max": 3, "anim": false, "ratio": 1 }
                }
            },
            {
                "id": "fo-2", "type": "sineWarp", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.08, "min": 0, "max": 0.6, "anim": false, "ratio": 1 },
                    "freq": { "value": 3.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1.0, "offset": 0 }
                }
            }
        ],
        "lift": [
            {
                "id": "fo-l1", "type": "helical", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "lift": { "value": 0.8, "min": -2, "max": 2, "anim": false, "ratio": 1 },
                    "twist": { "value": 3.0, "min": -8, "max": 8, "anim": true, "ratio": 0.25, "offset": 0 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
            "orbitY": { "on": true, "value": 0.6, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 3.5, "min": 0.8, "max": 12, "anim": true, "ratio": 0.382, "offset": 0 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "quaternion": {
        "on": true,
        "mode": "drift",
        "angle": { "value": 0.5, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 1, "offset": 0 },
        "ax": { "value": 0.5, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "ay": { "value": 1.0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "az": { "value": 0.3, "min": -1, "max": 1, "anim": false, "ratio": 1 },
        "driftSpeed": 0.08
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const LENS: &str = r#"{
    "metadata": {
        "name": "lens",
        "description": "zPow(n=3) + heavy Brown-Conrady k1/k2 + anamorphic squeeze",
        "tags": ["camera", "distortion", "optics"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "ln-1", "type": "zPow", "enabled": true,
                "blend": { "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "n": { "value": 3.0, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.382, "offset": 0 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 0.618, "offset": 0.25 }
                }
            },
            {
                "id": "ln-2", "type": "rotate", "enabled": true,
                "blend": { "value": 0.5, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 }
                }
            }
        ],
        "lift": []
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 },
            "orbitY": { "on": true, "value": 0.1, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 3.5, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        },
        "lens": {
            "projection": "perspective",
            "focalLength": { "on": true, "value": 35, "min": 8, "max": 200, "anim": false, "ratio": 1 },
            "sensorSize": { "on": true, "value": 36, "min": 4, "max": 80, "anim": false, "ratio": 1 },
            "orthoZoom": { "on": true, "value": 2, "min": 0.3, "max": 6, "anim": false, "ratio": 1 },
            "anamorphic": { "on": true, "value": 1.5, "min": 0.3, "max": 3, "anim": true, "ratio": 0.25, "offset": 0 },
            "shiftX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "shiftY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 }
        },
        "distortion": {
            "k1": { "on": true, "value": 0.5, "min": -1, "max": 1, "anim": true, "ratio": 0.618, "offset": 0 },
            "k2": { "on": true, "value": -0.3, "min": -1, "max": 1, "anim": true, "ratio": 0.382, "offset": 0.5 },
            "k3": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "k4": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "p1": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
            "p2": { "on": false, "value": 0, "min": -0.5, "max": 0.5, "anim": false, "ratio": 1 },
            "centerX": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "centerY": { "on": false, "value": 0, "min": -1, "max": 1, "anim": false, "ratio": 1 },
            "squeeze": { "on": true, "value": 1.5, "min": 0.1, "max": 3, "anim": true, "ratio": 0.25, "offset": 0 }
        }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 30, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

// =============================================================================
// Phosphor / rendering character
// =============================================================================

const GLOW: &str = r#"{
    "metadata": {
        "name": "glow",
        "description": "Simple rotate + translate + high persistence + bloom + warm palette",
        "tags": ["phosphor", "ambient", "bloom"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "gl-1", "type": "rotate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 }
                }
            },
            {
                "id": "gl-2", "type": "translate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "tx": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.618, "offset": 0 },
                    "ty": { "value": 0.0, "min": -1, "max": 1, "anim": true, "ratio": 0.382, "offset": 0.25 }
                }
            }
        ],
        "lift": [
            {
                "id": "gl-l1", "type": "zSaddle", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.3, "min": -2, "max": 2, "anim": false, "ratio": 1 }
                }
            }
        ]
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.3, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.25, "offset": 0 },
            "orbitY": { "on": true, "value": 0.5, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 4.0, "min": 0.8, "max": 12, "anim": false, "ratio": 1 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "phosphor": {
        "persistence": { "on": true, "value": 0.85, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 2.5, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.3, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 1.5, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": true, "value": 0.8, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": true, "value": 4.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": true, "value": 0.2, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 0.9, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.6, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 27, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

const STROBE: &str = r#"{
    "metadata": {
        "name": "strobe",
        "description": "Fast tempo + many animated params + low persistence + high beam intensity",
        "tags": ["phosphor", "energetic", "fast"]
    },
    "pipeline": {
        "operations": [
            {
                "id": "st-1", "type": "rotate", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "theta": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 2.0, "offset": 0, "shape": "sawtooth" }
                }
            },
            {
                "id": "st-2", "type": "scale", "enabled": true,
                "blend": { "value": 1.0, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "sx": { "value": 1.0, "min": 0.1, "max": 3, "anim": true, "ratio": 3.0, "offset": 0 },
                    "sy": { "value": 1.0, "min": 0.1, "max": 3, "anim": true, "ratio": 3.0, "offset": 0 }
                }
            },
            {
                "id": "st-3", "type": "sineWarp", "enabled": true,
                "blend": { "value": 0.8, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "amp": { "value": 0.1, "min": 0, "max": 0.6, "anim": true, "ratio": 2.0, "offset": 0 },
                    "freq": { "value": 6.0, "min": 0.5, "max": 20, "anim": false, "ratio": 1 },
                    "phase": { "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 4.0, "offset": 0 }
                }
            },
            {
                "id": "st-4", "type": "zTan", "enabled": true,
                "blend": { "value": 0.3, "min": 0, "max": 1, "anim": false, "ratio": 1 },
                "params": {
                    "cx": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 1.618, "offset": 0 },
                    "cy": { "value": 0.0, "min": -2, "max": 2, "anim": true, "ratio": 2.618, "offset": 0.5 }
                }
            }
        ],
        "lift": []
    },
    "camera": {
        "position": {
            "orbitX": { "on": true, "value": 0.0, "min": 0, "max": 6.283185307179586, "anim": true, "ratio": 0.5, "offset": 0 },
            "orbitY": { "on": true, "value": 0.1, "min": 0.15, "max": 2.8, "anim": false, "ratio": 1 },
            "distance": { "on": true, "value": 3.5, "min": 0.8, "max": 12, "anim": true, "ratio": 1.0, "offset": 0 },
            "targetX": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "targetY": { "on": false, "value": 0, "min": -2, "max": 2, "anim": false, "ratio": 1 },
            "roll": { "on": false, "value": 0, "min": 0, "max": 6.283185307179586, "anim": false, "ratio": 1 }
        }
    },
    "phosphor": {
        "persistence": { "on": true, "value": 0.15, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "beamWidth": { "on": true, "value": 3.0, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "beamIntensity": { "on": true, "value": 0.7, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "gain": { "on": true, "value": 2.0, "min": 0.1, "max": 5, "anim": false, "ratio": 1 },
        "blendMix": { "on": true, "value": 1, "min": 0, "max": 1, "anim": false, "ratio": 1 },
        "saturation": { "on": true, "value": 1.5, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomIntensity": { "on": false, "value": 0, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "bloomRadius": { "on": false, "value": 2, "min": 0.5, "max": 8, "anim": false, "ratio": 1 },
        "bloomThreshold": { "on": false, "value": 0.3, "min": 0, "max": 2, "anim": false, "ratio": 1 },
        "gamma": { "on": true, "value": 1.1, "min": 0.5, "max": 3, "anim": false, "ratio": 1 }
    },
    "sampling": {
        "scale": { "on": true, "value": 0.5, "min": 0.1, "max": 1.5, "anim": false, "ratio": 1 },
        "density": { "on": true, "value": 25, "min": 7, "max": 51, "anim": false, "ratio": 1 },
        "latticeType": "cartesian"
    }
}"#;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_factory_presets_parse() {
        let presets = factory_presets();
        assert_eq!(presets.len(), 15);
        for (name, preset) in &presets {
            assert!(!name.is_empty(), "preset has empty name");
            assert_eq!(preset.metadata.name, *name, "metadata.name mismatch for {name}");
        }
    }

    #[test]
    fn factory_presets_have_operations() {
        let presets = factory_presets();
        for (name, preset) in &presets {
            assert!(
                !preset.pipeline.operations.is_empty(),
                "preset \"{name}\" has no operations"
            );
        }
    }

    #[test]
    fn factory_presets_roundtrip() {
        let registry = crate::registry::Registry::new();
        let presets = factory_presets();
        for (name, preset) in &presets {
            let json = crate::preset_io::export_preset_json(preset, false);
            let result = crate::preset_io::import_preset_json(&json, &registry).unwrap();
            assert!(
                result.warnings.is_empty(),
                "roundtrip warnings for \"{name}\": {:?}",
                result.warnings
            );
        }
    }
}
