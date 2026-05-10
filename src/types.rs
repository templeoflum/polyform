use serde::{Deserialize, Serialize};

/// Golden ratio constant.
pub const PHI: f64 = 1.618033988749895; // (1 + sqrt(5)) / 2

// =============================================================================
// AnimMode
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimMode {
    Loop,
    Sweep,
}

// =============================================================================
// ProjectionMode
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectionMode {
    Perspective,
    Orthographic,
}

// =============================================================================
// QuaternionMode
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuaternionMode {
    Fixed,
    Drift,
}

// =============================================================================
// Param — every animatable scalar uses this shape
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Param {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    #[serde(default)]
    pub anim: bool,
    #[serde(default = "default_anim_mode")]
    pub anim_mode: AnimMode,
    #[serde(default = "default_ratio")]
    pub ratio: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase_offset: Option<Box<Param>>,
}

fn default_anim_mode() -> AnimMode {
    AnimMode::Sweep
}

fn default_ratio() -> f64 {
    1.0
}

impl Param {
    /// Create a param with the given value/min/max. anim=false, animMode based on periodic flag.
    pub fn new(value: f64, min: f64, max: f64, periodic: bool) -> Self {
        Self {
            value,
            min,
            max,
            anim: false,
            anim_mode: if periodic { AnimMode::Loop } else { AnimMode::Sweep },
            ratio: 1.0,
            phase_offset: None,
        }
    }

    /// Default phase offset param: value=0, min=0, max=1, sweep, ratio=1.
    pub fn default_phase_offset() -> Self {
        Self::new(0.0, 0.0, 1.0, false)
    }
}

impl Default for Param {
    fn default() -> Self {
        Self::new(0.0, 0.0, 1.0, false)
    }
}

// =============================================================================
// ToggleParam — a Param with an on/off switch
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleParam {
    pub on: bool,
    #[serde(flatten)]
    pub param: Param,
}

impl ToggleParam {
    pub fn new(on: bool, value: f64, min: f64, max: f64, periodic: bool) -> Self {
        Self {
            on,
            param: Param::new(value, min, max, periodic),
        }
    }

    /// Create with a custom ratio.
    pub fn with_ratio(mut self, ratio: f64) -> Self {
        self.param.ratio = ratio;
        self
    }
}

impl Default for ToggleParam {
    fn default() -> Self {
        Self::new(false, 0.0, 0.0, 1.0, false)
    }
}

// =============================================================================
// Blend — type alias for Param in [0, 1]
// =============================================================================

pub type Blend = Param;

/// Default blend: value=1.0, min=0, max=1, anim=false, sweep, ratio=1.
pub fn default_blend() -> Blend {
    Param::new(1.0, 0.0, 1.0, false)
}
