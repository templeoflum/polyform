use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::types::{LatticeType, Param, ProjectionMode, QuaternionMode, ToggleParam, PHI};

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
                0.0,
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
    #[serde(default = "default_distortion_k3")]
    pub k3: ToggleParam,
    #[serde(default = "default_distortion_k4")]
    pub k4: ToggleParam,
    #[serde(default = "default_distortion_p1")]
    pub p1: ToggleParam,
    #[serde(default = "default_distortion_p2")]
    pub p2: ToggleParam,
    #[serde(default = "default_distortion_center_x")]
    pub center_x: ToggleParam,
    #[serde(default = "default_distortion_center_y")]
    pub center_y: ToggleParam,
    #[serde(default = "default_distortion_squeeze")]
    pub squeeze: ToggleParam,
}

fn default_distortion_k3() -> ToggleParam { ToggleParam::new(false, 0.0, -1.0, 1.0, false) }
fn default_distortion_k4() -> ToggleParam { ToggleParam::new(false, 0.0, -1.0, 1.0, false) }
fn default_distortion_p1() -> ToggleParam { ToggleParam::new(false, 0.0, -0.5, 0.5, false) }
fn default_distortion_p2() -> ToggleParam { ToggleParam::new(false, 0.0, -0.5, 0.5, false) }
fn default_distortion_center_x() -> ToggleParam { ToggleParam::new(false, 0.0, -1.0, 1.0, false) }
fn default_distortion_center_y() -> ToggleParam { ToggleParam::new(false, 0.0, -1.0, 1.0, false) }
fn default_distortion_squeeze() -> ToggleParam { ToggleParam::new(false, 1.0, 0.1, 3.0, false) }

impl Default for CameraDistortion {
    fn default() -> Self {
        Self {
            k1: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
            k2: ToggleParam::new(false, 0.0, -1.0, 1.0, false),
            k3: default_distortion_k3(),
            k4: default_distortion_k4(),
            p1: default_distortion_p1(),
            p2: default_distortion_p2(),
            center_x: default_distortion_center_x(),
            center_y: default_distortion_center_y(),
            squeeze: default_distortion_squeeze(),
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
// LatticeParams
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatticeParams {
    #[serde(default = "default_spoke_count")]
    pub spoke_count: u32,
    #[serde(default = "default_inner_radius")]
    pub inner_radius: f64,
    #[serde(default = "default_focus_distance")]
    pub focus_distance: f64,
    #[serde(default = "default_tau_count")]
    pub tau_count: u32,
}

fn default_spoke_count() -> u32 { 24 }
fn default_inner_radius() -> f64 { 0.05 }
fn default_focus_distance() -> f64 { 1.0 }
fn default_tau_count() -> u32 { 16 }

impl Default for LatticeParams {
    fn default() -> Self {
        Self {
            spoke_count: 24,
            inner_radius: 0.05,
            focus_distance: 1.0,
            tau_count: 16,
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
    #[serde(default)]
    pub lattice_type: LatticeType,
    #[serde(default)]
    pub lattice_params: LatticeParams,
}

impl Default for Sampling {
    fn default() -> Self {
        Self {
            scale: ToggleParam::new(true, 0.6, 0.1, 1.5, false),
            density: ToggleParam::new(true, 27.0, 7.0, 51.0, false),
            lattice_type: LatticeType::Cartesian,
            lattice_params: LatticeParams::default(),
        }
    }
}

// =============================================================================
// Phosphor
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Phosphor {
    #[serde(default = "default_phosphor_persistence")]
    pub persistence: ToggleParam,
    #[serde(default = "default_phosphor_beam_width")]
    pub beam_width: ToggleParam,
    #[serde(default = "default_phosphor_beam_intensity")]
    pub beam_intensity: ToggleParam,
    #[serde(default = "default_phosphor_gain")]
    pub gain: ToggleParam,
    #[serde(default = "default_phosphor_blend_mix")]
    pub blend_mix: ToggleParam,
    #[serde(default = "default_phosphor_saturation")]
    pub saturation: ToggleParam,
    #[serde(default = "default_phosphor_bloom_intensity")]
    pub bloom_intensity: ToggleParam,
    #[serde(default = "default_phosphor_bloom_radius")]
    pub bloom_radius: ToggleParam,
    #[serde(default = "default_phosphor_bloom_threshold")]
    pub bloom_threshold: ToggleParam,
    #[serde(default = "default_phosphor_gamma")]
    pub gamma: ToggleParam,
}

fn default_phosphor_persistence() -> ToggleParam { ToggleParam::new(false, 0.0, 0.0, 1.0, false) }
fn default_phosphor_beam_width() -> ToggleParam { ToggleParam::new(true, 1.5, 0.5, 8.0, false) }
fn default_phosphor_beam_intensity() -> ToggleParam { ToggleParam::new(false, 0.0, 0.0, 1.0, false) }
fn default_phosphor_gain() -> ToggleParam { ToggleParam::new(true, 1.0, 0.1, 5.0, false) }
fn default_phosphor_blend_mix() -> ToggleParam { ToggleParam::new(true, 1.0, 0.0, 1.0, false) }
fn default_phosphor_saturation() -> ToggleParam { ToggleParam::new(true, 1.0, 0.0, 2.0, false) }
fn default_phosphor_bloom_intensity() -> ToggleParam { ToggleParam::new(false, 0.0, 0.0, 2.0, false) }
fn default_phosphor_bloom_radius() -> ToggleParam { ToggleParam::new(false, 2.0, 0.5, 8.0, false) }
fn default_phosphor_bloom_threshold() -> ToggleParam { ToggleParam::new(false, 0.3, 0.0, 2.0, false) }
fn default_phosphor_gamma() -> ToggleParam { ToggleParam::new(true, 1.0, 0.5, 3.0, false) }

impl Default for Phosphor {
    fn default() -> Self {
        Self {
            persistence: default_phosphor_persistence(),
            beam_width: default_phosphor_beam_width(),
            beam_intensity: default_phosphor_beam_intensity(),
            gain: default_phosphor_gain(),
            blend_mix: default_phosphor_blend_mix(),
            saturation: default_phosphor_saturation(),
            bloom_intensity: default_phosphor_bloom_intensity(),
            bloom_radius: default_phosphor_bloom_radius(),
            bloom_threshold: default_phosphor_bloom_threshold(),
            gamma: default_phosphor_gamma(),
        }
    }
}

// =============================================================================
// Top-level Preset
// =============================================================================

/// The instrument patch: pipeline + camera + quaternion + sampling + phosphor.
/// Ephemeral session state (tempo, palette, display, viewport) lives in the App.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub pipeline: Pipeline,
    #[serde(default)]
    pub camera: Camera,
    #[serde(default)]
    pub quaternion: Quaternion,
    #[serde(default)]
    pub sampling: Sampling,
    #[serde(default)]
    pub phosphor: Phosphor,
}

impl Default for Preset {
    fn default() -> Self {
        Self {
            metadata: Metadata::default(),
            pipeline: Pipeline::default(),
            camera: Camera::default(),
            quaternion: Quaternion::default(),
            sampling: Sampling::default(),
            phosphor: Phosphor::default(),
        }
    }
}
