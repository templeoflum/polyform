use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::types::{Param, ProjectionMode, QuaternionMode, ToggleParam, PHI};

pub const SCHEMA_NAME: &str = "polyform.preset";
pub const SCHEMA_VERSION: u32 = 1;

// =============================================================================
// Metadata
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub modified: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_name() -> String {
    "Untitled".into()
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: "Untitled".into(),
            description: String::new(),
            author: String::new(),
            created: String::new(),
            modified: String::new(),
            tags: Vec::new(),
        }
    }
}

// =============================================================================
// Defaults
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Defaults {
    #[serde(default = "default_blend_time")]
    pub blend_time: f64,
}

fn default_blend_time() -> f64 {
    1.0
}

impl Default for Defaults {
    fn default() -> Self {
        Self { blend_time: 1.0 }
    }
}

// =============================================================================
// Pipeline
// =============================================================================

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pipeline {
    #[serde(default)]
    pub operations: Vec<Layer>,
    #[serde(default)]
    pub lift: Vec<Layer>,
}

// =============================================================================
// Camera
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraPosition {
    pub orbit_x: ToggleParam,
    pub orbit_y: ToggleParam,
    pub distance: ToggleParam,
    pub target_x: ToggleParam,
    pub target_y: ToggleParam,
    pub roll: ToggleParam,
}

impl Default for CameraPosition {
    fn default() -> Self {
        Self {
            orbit_x: ToggleParam::new(true, 0.0, 0.0, std::f64::consts::TAU, true),
            orbit_y: ToggleParam::new(
                true,
                0.1,
                0.1,
                std::f64::consts::PI - 0.1,
                true,
            )
            .with_ratio(1.0 / PHI),
            distance: ToggleParam::new(true, 3.5, 0.8, 12.0, false),
            target_x: ToggleParam::new(false, 0.0, -2.0, 2.0, false),
            target_y: ToggleParam::new(false, 0.0, -2.0, 2.0, false),
            roll: ToggleParam::new(false, 0.0, 0.0, std::f64::consts::TAU, true),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraLens {
    #[serde(default = "default_projection")]
    pub projection: ProjectionMode,
    pub focal_length: ToggleParam,
    pub sensor_size: ToggleParam,
    pub ortho_zoom: ToggleParam,
    pub anamorphic: ToggleParam,
    pub shift_x: ToggleParam,
    pub shift_y: ToggleParam,
}

fn default_projection() -> ProjectionMode {
    ProjectionMode::Perspective
}

impl Default for CameraLens {
    fn default() -> Self {
        Self {
            projection: ProjectionMode::Perspective,
            focal_length: ToggleParam::new(true, 35.0, 8.0, 200.0, false),
            sensor_size: ToggleParam::new(true, 36.0, 4.0, 80.0, false),
            ortho_zoom: ToggleParam::new(true, 2.0, 0.3, 6.0, false),
            anamorphic: ToggleParam::new(false, 1.0, 0.3, 3.0, false),
            shift_x: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
            shift_y: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraDistortion {
    pub k1: ToggleParam,
    pub k2: ToggleParam,
}

impl Default for CameraDistortion {
    fn default() -> Self {
        Self {
            k1: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
            k2: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    #[serde(default)]
    pub position: CameraPosition,
    #[serde(default)]
    pub lens: CameraLens,
    #[serde(default)]
    pub distortion: CameraDistortion,
}

// =============================================================================
// Quaternion tumble — now uses full Param for angle and axes
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quaternion {
    #[serde(default)]
    pub on: bool,
    #[serde(default = "default_quat_mode")]
    pub mode: QuaternionMode,
    #[serde(default = "default_quat_angle")]
    pub angle: Param,
    #[serde(default = "default_quat_ax")]
    pub ax: Param,
    #[serde(default = "default_quat_ay")]
    pub ay: Param,
    #[serde(default = "default_quat_az")]
    pub az: Param,
    #[serde(default = "default_drift_speed")]
    pub drift_speed: f64,
}

fn default_quat_mode() -> QuaternionMode {
    QuaternionMode::Fixed
}

fn default_drift_speed() -> f64 {
    0.1
}

fn default_quat_angle() -> Param {
    let mut p = Param::new(0.0, 0.0, std::f64::consts::TAU, true);
    p.anim = true;
    p
}

fn default_quat_ax() -> Param {
    let mut p = Param::new(0.5, -1.0, 1.0, false);
    p.ratio = PHI;
    p
}

fn default_quat_ay() -> Param {
    Param::new(1.0, -1.0, 1.0, false)
}

fn default_quat_az() -> Param {
    let mut p = Param::new(0.3, -1.0, 1.0, false);
    p.ratio = 1.0 / PHI;
    p
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            on: false,
            mode: QuaternionMode::Fixed,
            angle: default_quat_angle(),
            ax: default_quat_ax(),
            ay: default_quat_ay(),
            az: default_quat_az(),
            drift_speed: 0.1,
        }
    }
}

// =============================================================================
// Sampling
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sampling {
    pub scale: ToggleParam,
    pub density: ToggleParam,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            scale: ToggleParam::new(true, 0.6, 0.1, 1.5, false),
            density: ToggleParam::new(true, 27.0, 7.0, 51.0, false),
        }
    }
}

// =============================================================================
// Display toggles
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    #[serde(default = "default_true")]
    pub show_formula: bool,
    #[serde(default = "default_true")]
    pub show_dim_count: bool,
    #[serde(default = "default_true")]
    pub show_active_layers: bool,
    #[serde(default = "default_true")]
    pub show_param_values: bool,
    #[serde(default)]
    pub show_fps: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Display {
    fn default() -> Self {
        Self {
            show_formula: true,
            show_dim_count: true,
            show_active_layers: true,
            show_param_values: true,
            show_fps: false,
        }
    }
}

// =============================================================================
// Top-level Preset
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default = "default_tempo")]
    pub tempo: Param,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default = "default_palette")]
    pub palette: String,
    #[serde(default)]
    pub pipeline: Pipeline,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default)]
    pub quaternion: Quaternion,
    #[serde(default)]
    pub sampling: Sampling,
    #[serde(default)]
    pub display: Display,
}

fn default_schema() -> String {
    SCHEMA_NAME.into()
}

fn default_version() -> u32 {
    SCHEMA_VERSION
}

fn default_tempo() -> Param {
    Param::new(0.3, 0.0, 2.0, false)
}

fn default_palette() -> String {
    "phosphor".into()
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            schema: SCHEMA_NAME.into(),
            version: SCHEMA_VERSION,
            metadata: Metadata::default(),
            tempo: default_tempo(),
            defaults: Defaults::default(),
            palette: "phosphor".into(),
            pipeline: Pipeline::default(),
            camera: Camera::default(),
            quaternion: Quaternion::default(),
            sampling: Sampling::default(),
            display: Display::default(),
        }
    }
}
