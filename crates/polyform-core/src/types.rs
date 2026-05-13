use serde::{Deserialize, Serialize};

/// Golden ratio constant.
pub const PHI: f64 = 1.618033988749895; // (1 + sqrt(5)) / 2

// =============================================================================
// ShapeFunction
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeFunction {
    Sawtooth, // linear ramp 0->1 (replaces Loop)
    Cosine,   // ping-pong via 0.5 - 0.5*cos(phi*TAU) (replaces Sweep)
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
// LatticeType
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LatticeType {
    Cartesian,
    Polar,
    Hex,
    LogPolar,
    Bipolar,
}

impl Default for LatticeType {
    fn default() -> Self {
        Self::Cartesian
    }
}

// =============================================================================
// PhaseModulator — optional FM modulation per param
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseModulator {
    pub ratio: f64,
    pub depth: f64,
    pub shape: ShapeFunction,
}

impl Default for PhaseModulator {
    fn default() -> Self {
        Self {
            ratio: 1.0,
            depth: 0.0,
            shape: ShapeFunction::Cosine,
        }
    }
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
    #[serde(default = "default_ratio")]
    pub ratio: f64,
    #[serde(default)]
    pub offset: f64,
    #[serde(default = "default_shape")]
    pub shape: ShapeFunction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modulator: Option<PhaseModulator>,
}

fn default_ratio() -> f64 {
    1.0
}

fn default_shape() -> ShapeFunction {
    ShapeFunction::Cosine
}

impl Param {
    /// Create a param with the given value/min/max. anim=false, shape defaults to Cosine.
    pub fn new(value: f64, min: f64, max: f64, _periodic: bool) -> Self {
        Self {
            value,
            min,
            max,
            anim: false,
            ratio: 1.0,
            offset: 0.0,
            shape: ShapeFunction::Cosine,
            modulator: None,
        }
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

/// Default blend: value=1.0, min=0, max=1, anim=false, cosine, ratio=1.
pub fn default_blend() -> Blend {
    Param::new(1.0, 0.0, 1.0, false)
}
