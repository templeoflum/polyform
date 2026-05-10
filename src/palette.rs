use serde::{Deserialize, Serialize};

// =============================================================================
// Palette — a set of four hex colors
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Palette {
    pub bg: String,
    pub grid: String,
    pub warp: String,
    pub glow: String,
}

/// Look up a palette by key. Returns None for unknown keys.
pub fn get_palette(key: &str) -> Option<Palette> {
    match key {
        "acid" => Some(Palette {
            bg: "#000".into(),
            grid: "#7fff5a".into(),
            warp: "#caff3a".into(),
            glow: "#7fff5a".into(),
        }),
        "cyan" => Some(Palette {
            bg: "#000".into(),
            grid: "#5be3d4".into(),
            warp: "#bff7ee".into(),
            glow: "#5be3d4".into(),
        }),
        "blush" => Some(Palette {
            bg: "#000".into(),
            grid: "#ff5e8a".into(),
            warp: "#ffb3c8".into(),
            glow: "#ff5e8a".into(),
        }),
        "amber" => Some(Palette {
            bg: "#0a0500".into(),
            grid: "#ffb347".into(),
            warp: "#fff1c2".into(),
            glow: "#ffb347".into(),
        }),
        "void" => Some(Palette {
            bg: "#0a0a12".into(),
            grid: "#b39dff".into(),
            warp: "#e8defc".into(),
            glow: "#b39dff".into(),
        }),
        _ => None,
    }
}

/// All palette keys in canonical order.
pub const PALETTE_KEYS: &[&str] = &["acid", "cyan", "blush", "amber", "void"];
