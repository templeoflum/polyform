use std::collections::HashMap;

use crate::preset::LatticeParams;
use crate::types::{LatticeType, ProjectionMode};

// =============================================================================
// FrameState — all per-frame resolved values for rendering
// =============================================================================

/// Complete snapshot of all animated/resolved values for a single frame.
/// Produced by the engine from an immutable Preset + internal sweep state.
/// Consumed by the renderer (GPU uniforms, buffer uploads).
#[derive(Debug, Clone)]
pub struct FrameState {
    // Pipeline
    pub operations: Vec<FlatLayerState>,
    pub lifts: Vec<FlatLayerState>,
    pub any_lift_needs_jacobian: bool,

    // Camera (resolved f64 values)
    pub orbit_x: f64,
    pub orbit_y: f64,
    pub distance: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub roll: f64,
    pub projection: ProjectionMode,
    pub focal_length: f64,
    pub sensor_size: f64,
    pub ortho_zoom: f64,
    pub anamorphic: f64,
    pub shift_x: f64,
    pub shift_y: f64,
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
    pub k4: f64,
    pub p1: f64,
    pub p2: f64,
    pub distortion_cx: f64,
    pub distortion_cy: f64,
    pub squeeze: f64,

    // Quaternion
    pub quat_on: bool,
    pub quat: [f64; 4],            // final [w,x,y,z] for rendering (SLERP-blended)
    pub quat_angle: f64,           // resolved angle param (for UI display)
    pub quat_axis: [f64; 3],       // resolved axis values (for UI display)

    // Sampling
    pub scale: f64,
    pub density: f64,
    pub lattice_type: LatticeType,
    pub lattice_params: LatticeParams,

    // Phosphor
    pub persistence: f64,
    pub beam_width: f64,
    pub beam_intensity: f64,
    pub gain: f64,
    pub blend_mix: f64,
    pub saturation: f64,
    pub bloom_intensity: f64,
    pub bloom_radius: f64,
    pub bloom_threshold: f64,
    pub gamma: f64,

    // Display
    pub palette_key: String,
    pub has_active_layers: bool,
    pub show_caustic: bool,
}

/// Flattened layer state: type key, blend amount, and all params as resolved f64 values.
#[derive(Debug, Clone)]
pub struct FlatLayerState {
    pub type_key: String,
    pub blend: f64,
    pub params: HashMap<String, f64>,
}
