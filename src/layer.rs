use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::types::{default_blend, Blend, Param};

// =============================================================================
// Layer — a single pipeline entry (operation or lift)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    /// Stable, unique id; persists across saves and reorderings.
    pub id: String,

    /// Registry key (e.g. "zPow", "zDisplacement").
    #[serde(rename = "type")]
    pub type_key: String,

    /// User-facing on/off.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Per-layer blend amount in [0, 1].
    #[serde(default = "default_blend")]
    pub blend: Blend,

    /// Per-layer override for blend time, in seconds.
    /// null in JSON means use the global default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend_time_override: Option<f64>,

    /// Layer-specific params keyed by registry-defined names.
    #[serde(default)]
    pub params: HashMap<String, Param>,
}

fn default_true() -> bool {
    true
}
