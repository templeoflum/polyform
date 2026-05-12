use std::collections::HashMap;

use crate::types::ProjectionMode;

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

    // Quaternion
    pub quat_on: bool,
    pub quat_angle: f64,
    pub quat_axis: [f64; 3],

    // Sampling
    pub scale: f64,
    pub density: f64,

    // Display
    pub palette_key: String,
    pub has_active_layers: bool,
}

/// Flattened layer state: type key, blend amount, and all params as resolved f64 values.
#[derive(Debug, Clone)]
pub struct FlatLayerState {
    pub type_key: String,
    pub blend: f64,
    pub params: HashMap<String, f64>,
}
