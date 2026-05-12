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
        // P1 phosphor — the canonical oscilloscope green.
        // Tektronix 465, HP 54600. Grid dim, trace bright yellow-green.
        "phosphor" => Some(Palette {
            bg: "#000".into(),
            grid: "#1a3a12".into(),
            warp: "#33ff00".into(),
            glow: "#33ff00".into(),
        }),
        // P31 phosphor — blue-green CRT used on Tektronix 2465 and radar.
        // Cooler than P1, faint cyan grid, bright cyan-white trace.
        "p31" => Some(Palette {
            bg: "#000".into(),
            grid: "#0e2a2a".into(),
            warp: "#00e5cc".into(),
            glow: "#00e5cc".into(),
        }),
        // Amber monitor — the IBM 5151, Wyse WY-50, Hercules graphics.
        // Warm amber phosphor on black. Trace goes pale gold.
        "amber" => Some(Palette {
            bg: "#000".into(),
            grid: "#3d2800".into(),
            warp: "#ffb000".into(),
            glow: "#ffb000".into(),
        }),
        // White phosphor — Fairlight CMI, early vector displays, P4.
        // Cool white trace on deep black. Grid faint blue-grey.
        "white" => Some(Palette {
            bg: "#000".into(),
            grid: "#1a1a22".into(),
            warp: "#d4d4e0".into(),
            glow: "#d4d4e0".into(),
        }),
        // Rutt-Etra blue — the video synthesizer aesthetic.
        // Electric blue scan lines on black.
        "rutt-etra" => Some(Palette {
            bg: "#000".into(),
            grid: "#0a0a30".into(),
            warp: "#4488ff".into(),
            glow: "#4488ff".into(),
        }),
        // Backward compat aliases for v1 presets
        "acid" => get_palette("phosphor"),
        "cyan" => get_palette("p31"),
        "blush" | "void" => get_palette("white"),
        _ => None,
    }
}

/// All palette keys in canonical order.
pub const PALETTE_KEYS: &[&str] = &["phosphor", "p31", "amber", "white", "rutt-etra"];
