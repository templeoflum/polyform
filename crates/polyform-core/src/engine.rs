use std::collections::HashMap;

use crate::animation::{resolve_param, safe_ratio, ParamPath, PipelineKind};
use crate::frame_state::{FlatLayerState, FrameState};
use crate::preset::Preset;
use crate::transition::{TransitionAction, TransitionManager, TransitionTarget};
use crate::types::{QuaternionMode, ToggleParam};

// =============================================================================
// AnimWeight — smooth weight transitions
// =============================================================================

#[derive(Debug, Clone)]
struct AnimWeight {
    current: f64,
    target: f64,
    rate: f64, // change per second (1/duration)
}

// =============================================================================
// AnimationEngine
// =============================================================================

/// Orchestrates all animation: master phase, weight transitions, and quaternion tumble.
///
/// The caller controls timing:
/// - **Live**: `dt = wall_clock_delta`, `wall_dt = wall_clock_delta`
/// - **Offline**: `dt = 1.0/fps`, `wall_dt = 1.0/fps`
/// - **Paused**: `dt = 0.0`, `wall_dt = wall_clock_delta`
///
/// All param values are pure functions of `master_phase` + config. No per-param accumulators.
pub struct AnimationEngine {
    /// Master phase M in [0, 1), wraps. Driven by dM/dt = tempo.
    pub master_phase: f64,
    /// Smooth weight transitions for anim on/off.
    anim_weights: HashMap<ParamPath, AnimWeight>,
    /// Cosine-eased transition manager (structural: blend fades, preset transitions).
    pub transitions: TransitionManager,
    /// Wall-clock time (drives transitions, runs even when paused).
    wall_clock: f64,
    /// Internal transition value overlay.
    transition_overlay: HashMap<TransitionTarget, f64>,
    /// Quaternion drift phase accumulator (deliberately decoupled from master phase).
    drift_phase: f64,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            master_phase: 0.0,
            anim_weights: HashMap::new(),
            transitions: TransitionManager::new(),
            wall_clock: 0.0,
            transition_overlay: HashMap::new(),
            drift_phase: 0.0,
        }
    }

    /// Clear all engine state (call on preset load).
    pub fn reset(&mut self) {
        self.master_phase = 0.0;
        self.anim_weights.clear();
        self.transitions = TransitionManager::new();
        self.wall_clock = 0.0;
        self.transition_overlay.clear();
        self.drift_phase = 0.0;
    }

    /// Deep clone for offline rendering.
    pub fn snapshot(&self) -> Self {
        Self {
            master_phase: self.master_phase,
            anim_weights: self.anim_weights.clone(),
            transitions: TransitionManager::new(),
            wall_clock: self.wall_clock,
            transition_overlay: HashMap::new(),
            drift_phase: self.drift_phase,
        }
    }

    // =========================================================================
    // Weight management
    // =========================================================================

    /// Get current weight for a path (default 1.0 if no entry).
    pub fn get_weight(&self, path: &ParamPath) -> f64 {
        self.anim_weights
            .get(path)
            .map_or(1.0, |w| w.current)
    }

    /// Start smooth transition of weight toward target over duration seconds.
    pub fn set_weight_target(&mut self, path: ParamPath, target: f64, duration: f64) {
        let current = self.get_weight(&path);
        let rate = if duration > 0.0 { 1.0 / duration } else { f64::INFINITY };
        self.anim_weights.insert(path, AnimWeight {
            current,
            target,
            rate,
        });
    }

    /// Advance all weights toward their targets. Clean up entries at default (1.0).
    fn advance_weights(&mut self, wall_dt: f64) {
        let mut to_remove = Vec::new();
        for (path, w) in self.anim_weights.iter_mut() {
            if (w.current - w.target).abs() < 1e-12 {
                w.current = w.target;
                // Clean up entries at default (target=1.0, current=1.0)
                if (w.target - 1.0).abs() < 1e-12 {
                    to_remove.push(path.clone());
                }
                continue;
            }
            let step = w.rate * wall_dt;
            if w.current < w.target {
                w.current = (w.current + step).min(w.target);
            } else {
                w.current = (w.current - step).max(w.target);
            }
        }
        for path in to_remove {
            self.anim_weights.remove(&path);
        }
    }

    // =========================================================================
    // Main tick
    // =========================================================================

    /// Single entry point called once per frame.
    ///
    /// - `dt`: animation time delta (0 when paused)
    /// - `wall_dt`: wall-clock time delta (always positive)
    pub fn tick(
        &mut self,
        preset: &Preset,
        dt: f64,
        wall_dt: f64,
    ) -> (FrameState, Vec<TransitionAction>) {
        // 1. Advance master phase
        self.master_phase =
            (self.master_phase + dt * safe_ratio(preset.tempo.value)) % 1.0;

        // 2. Advance wall clock and weights
        self.wall_clock += wall_dt;
        self.advance_weights(wall_dt);

        // 3. Process structural transitions
        let actions = self.process_transitions();

        // 4. Advance drift phase (for quaternion drift mode)
        self.drift_phase += dt * preset.quaternion.drift_speed;

        // 5. Resolve all params
        let op_resolved = self.resolve_pipeline_layers(preset, PipelineKind::Operations);
        let lift_resolved = self.resolve_pipeline_layers(preset, PipelineKind::Lift);
        let camera_resolved = self.resolve_camera_values(preset);
        let (quat_angle, quat_axis) = self.resolve_quaternion_values(preset);
        let sampling_resolved = self.resolve_sampling_values(preset);

        // 6. Build FrameState
        let frame_state = self.build_frame_state(
            preset,
            &op_resolved,
            &lift_resolved,
            &camera_resolved,
            quat_angle,
            quat_axis,
            &sampling_resolved,
        );

        (frame_state, actions)
    }

    // =========================================================================
    // Transitions
    // =========================================================================

    fn process_transitions(&mut self) -> Vec<TransitionAction> {
        self.transition_overlay
            .retain(|target, _| self.transitions.has_active(target));

        let (updates, actions) = self.transitions.tick(self.wall_clock);

        for update in &updates {
            self.transition_overlay
                .insert(update.target.clone(), update.value);
        }

        actions
    }

    // =========================================================================
    // Pipeline layer resolution
    // =========================================================================

    fn resolve_pipeline_layers(
        &self,
        preset: &Preset,
        kind: PipelineKind,
    ) -> Vec<ResolvedLayer> {
        let layers = match kind {
            PipelineKind::Operations => &preset.pipeline.operations,
            PipelineKind::Lift => &preset.pipeline.lift,
        };

        let mut result = Vec::with_capacity(layers.len());

        for layer in layers.iter() {
            let mut resolved_params = HashMap::new();

            // Resolve blend
            let blend_path = ParamPath::LayerBlend {
                kind,
                layer_id: layer.id.clone(),
            };
            let blend_weight = self.get_weight(&blend_path);
            let resolved_blend = if layer.enabled {
                resolve_param(&layer.blend, self.master_phase, blend_weight)
            } else {
                layer.blend.value
            };

            if layer.enabled {
                for (key, param) in &layer.params {
                    let path = ParamPath::LayerParam {
                        kind,
                        layer_id: layer.id.clone(),
                        key: key.clone(),
                    };
                    let weight = self.get_weight(&path);
                    let val = resolve_param(param, self.master_phase, weight);
                    resolved_params.insert(key.clone(), val);
                }
            } else {
                for (key, param) in &layer.params {
                    resolved_params.insert(key.clone(), param.value);
                }
            }

            result.push(ResolvedLayer {
                id: layer.id.clone(),
                type_key: layer.type_key.clone(),
                enabled: layer.enabled,
                blend: resolved_blend,
                params: resolved_params,
            });
        }

        result
    }

    // =========================================================================
    // Camera resolution
    // =========================================================================

    fn resolve_camera_values(&self, preset: &Preset) -> CameraResolved {
        CameraResolved {
            orbit_x: self.resolve_toggle_param(&preset.camera.position.orbit_x, "camera.position.orbitX"),
            orbit_y: self.resolve_toggle_param(&preset.camera.position.orbit_y, "camera.position.orbitY"),
            distance: self.resolve_toggle_param(&preset.camera.position.distance, "camera.position.distance"),
            target_x: self.resolve_toggle_param(&preset.camera.position.target_x, "camera.position.targetX"),
            target_y: self.resolve_toggle_param(&preset.camera.position.target_y, "camera.position.targetY"),
            roll: self.resolve_toggle_param(&preset.camera.position.roll, "camera.position.roll"),
            focal_length: self.resolve_toggle_param(&preset.camera.lens.focal_length, "camera.lens.focalLength"),
            sensor_size: self.resolve_toggle_param(&preset.camera.lens.sensor_size, "camera.lens.sensorSize"),
            ortho_zoom: self.resolve_toggle_param(&preset.camera.lens.ortho_zoom, "camera.lens.orthoZoom"),
            anamorphic: self.resolve_toggle_param(&preset.camera.lens.anamorphic, "camera.lens.anamorphic"),
            shift_x: self.resolve_toggle_param(&preset.camera.lens.shift_x, "camera.lens.shiftX"),
            shift_y: self.resolve_toggle_param(&preset.camera.lens.shift_y, "camera.lens.shiftY"),
            k1: self.resolve_toggle_param(&preset.camera.distortion.k1, "camera.distortion.k1"),
            k2: self.resolve_toggle_param(&preset.camera.distortion.k2, "camera.distortion.k2"),
        }
    }

    fn resolve_toggle_param(&self, tp: &ToggleParam, view_key: &str) -> f64 {
        if !tp.on {
            return tp.param.value;
        }
        let path = ParamPath::View(view_key.to_string());
        let weight = self.get_weight(&path);
        resolve_param(&tp.param, self.master_phase, weight)
    }

    // =========================================================================
    // Sampling resolution
    // =========================================================================

    fn resolve_sampling_values(&self, preset: &Preset) -> SamplingResolved {
        SamplingResolved {
            scale: self.resolve_toggle_param(&preset.sampling.scale, "sampling.scale"),
            density: self.resolve_toggle_param(&preset.sampling.density, "sampling.density"),
        }
    }

    // =========================================================================
    // Quaternion resolution
    // =========================================================================

    fn resolve_quaternion_values(&self, preset: &Preset) -> (f64, [f64; 3]) {
        let q = &preset.quaternion;
        if !q.on {
            return (q.angle.value, [q.ax.value, q.ay.value, q.az.value]);
        }

        // Angle — resolve as a normal param via master phase
        let angle_weight = self.get_weight(&ParamPath::View("quaternion.angle".to_string()));
        let angle = resolve_param(&q.angle, self.master_phase, angle_weight);

        // Axes
        let axis = match q.mode {
            QuaternionMode::Drift => {
                let t = self.drift_phase;
                [
                    (t * 0.7).sin() + 0.3 * (t * 1.3).cos(),
                    (t * 0.5).cos() + 0.3 * (t * 1.7).sin(),
                    (t * 0.9 + 1.2).sin(),
                ]
            }
            QuaternionMode::Fixed => {
                let ax_weight = self.get_weight(&ParamPath::View("quaternion.ax".to_string()));
                let ay_weight = self.get_weight(&ParamPath::View("quaternion.ay".to_string()));
                let az_weight = self.get_weight(&ParamPath::View("quaternion.az".to_string()));
                let ax = resolve_param(&q.ax, self.master_phase, ax_weight);
                let ay = resolve_param(&q.ay, self.master_phase, ay_weight);
                let az = resolve_param(&q.az, self.master_phase, az_weight);
                [ax, ay, az]
            }
        };

        (angle, axis)
    }

    // =========================================================================
    // FrameState construction
    // =========================================================================

    fn build_frame_state(
        &self,
        preset: &Preset,
        op_resolved: &[ResolvedLayer],
        lift_resolved: &[ResolvedLayer],
        camera_resolved: &CameraResolved,
        quat_angle: f64,
        quat_axis: [f64; 3],
        sampling_resolved: &SamplingResolved,
    ) -> FrameState {
        let operations: Vec<FlatLayerState> = op_resolved
            .iter()
            .map(|sl| {
                let blend = self.resolve_blend(sl, PipelineKind::Operations);
                FlatLayerState {
                    type_key: sl.type_key.clone(),
                    blend,
                    params: sl.params.clone(),
                }
            })
            .collect();

        let lifts: Vec<FlatLayerState> = lift_resolved
            .iter()
            .map(|sl| {
                let blend = self.resolve_blend(sl, PipelineKind::Lift);
                FlatLayerState {
                    type_key: sl.type_key.clone(),
                    blend,
                    params: sl.params.clone(),
                }
            })
            .collect();

        let jacobian_lifts = ["zLogJac", "zSignedJac"];
        let any_lift_needs_jacobian = lifts
            .iter()
            .any(|l| l.blend > 0.0 && jacobian_lifts.contains(&l.type_key.as_str()));

        let has_active_layers = operations.iter().any(|l| l.blend > 0.0)
            || lifts.iter().any(|l| l.blend > 0.0);

        let orbit_x = self.resolve_view_transition("camera.position.orbitX", camera_resolved.orbit_x);
        let orbit_y = self.resolve_view_transition("camera.position.orbitY", camera_resolved.orbit_y);
        let distance = self.resolve_view_transition("camera.position.distance", camera_resolved.distance);
        let target_x = self.resolve_view_transition("camera.position.targetX", camera_resolved.target_x);
        let target_y = self.resolve_view_transition("camera.position.targetY", camera_resolved.target_y);
        let roll = self.resolve_view_transition("camera.position.roll", camera_resolved.roll);
        let focal_length = self.resolve_view_transition("camera.lens.focalLength", camera_resolved.focal_length);
        let sensor_size = self.resolve_view_transition("camera.lens.sensorSize", camera_resolved.sensor_size);
        let ortho_zoom = self.resolve_view_transition("camera.lens.orthoZoom", camera_resolved.ortho_zoom);
        let anamorphic = self.resolve_view_transition("camera.lens.anamorphic", camera_resolved.anamorphic);
        let shift_x = self.resolve_view_transition("camera.lens.shiftX", camera_resolved.shift_x);
        let shift_y = self.resolve_view_transition("camera.lens.shiftY", camera_resolved.shift_y);
        let k1 = self.resolve_view_transition("camera.distortion.k1", camera_resolved.k1);
        let k2 = self.resolve_view_transition("camera.distortion.k2", camera_resolved.k2);

        FrameState {
            operations,
            lifts,
            any_lift_needs_jacobian,
            orbit_x,
            orbit_y,
            distance,
            target_x,
            target_y,
            roll,
            projection: preset.camera.lens.projection,
            focal_length,
            sensor_size,
            ortho_zoom,
            anamorphic,
            shift_x,
            shift_y,
            k1,
            k2,
            quat_on: preset.quaternion.on,
            quat_angle,
            quat_axis,
            scale: sampling_resolved.scale,
            density: sampling_resolved.density,
            palette_key: preset.palette.clone(),
            has_active_layers,
        }
    }

    fn resolve_blend(&self, sl: &ResolvedLayer, kind: PipelineKind) -> f64 {
        if !sl.enabled {
            return 0.0;
        }
        let target = TransitionTarget::LayerBlend {
            kind,
            layer_id: sl.id.clone(),
        };
        if let Some(&v) = self.transition_overlay.get(&target) {
            v
        } else {
            sl.blend
        }
    }

    fn resolve_view_transition(&self, name: &str, resolved_value: f64) -> f64 {
        let target = TransitionTarget::ViewParam(name.to_string());
        if let Some(&v) = self.transition_overlay.get(&target) {
            v
        } else {
            resolved_value
        }
    }

    // =========================================================================
    // Convenience methods (structural mutations)
    // =========================================================================

    /// Toggle a layer on/off with a blend transition.
    pub fn toggle_layer(
        &mut self,
        preset: &mut Preset,
        kind: PipelineKind,
        layer_id: &str,
        blend_time: f64,
    ) {
        let layers = match kind {
            PipelineKind::Operations => &mut preset.pipeline.operations,
            PipelineKind::Lift => &mut preset.pipeline.lift,
        };

        let layer = match layers.iter_mut().find(|l| l.id == layer_id) {
            Some(l) => l,
            None => return,
        };

        let target = TransitionTarget::LayerBlend {
            kind,
            layer_id: layer_id.to_string(),
        };

        self.transitions.cancel_for(&target);
        self.transition_overlay.remove(&target);

        let duration = layer.blend_time_override.unwrap_or(blend_time);

        if layer.enabled {
            // --- DISABLE ---
            let blend_path = ParamPath::LayerBlend {
                kind,
                layer_id: layer_id.to_string(),
            };
            let blend_weight = self.get_weight(&blend_path);
            let current_visual = self
                .transition_overlay
                .get(&target)
                .copied()
                .unwrap_or_else(|| {
                    resolve_param(&layer.blend, self.master_phase, blend_weight)
                });

            self.transitions.start(
                target,
                current_visual,
                0.0,
                duration,
                Some(TransitionAction::DisableLayer {
                    kind,
                    layer_id: layer_id.to_string(),
                }),
                Some(self.wall_clock),
            );
        } else {
            // --- ENABLE ---
            let restore_value = if layer.blend.value > 0.001 {
                layer.blend.value
            } else {
                1.0
            };
            layer.enabled = true;
            layer.blend.value = 0.0;

            self.transitions.start(
                target,
                0.0,
                restore_value,
                duration,
                Some(TransitionAction::RestoreBlendValue {
                    kind,
                    layer_id: layer_id.to_string(),
                    value: restore_value,
                }),
                Some(self.wall_clock),
            );
        }
    }

    /// Cancel any active blend transition for a layer and clear its overlay entry.
    pub fn cancel_blend_transition(&mut self, kind: PipelineKind, layer_id: &str) {
        let target = TransitionTarget::LayerBlend {
            kind,
            layer_id: layer_id.to_string(),
        };
        self.transitions.cancel_for(&target);
        self.transition_overlay.remove(&target);
    }

    /// Set a param value directly (for slider drag — no transition).
    pub fn set_param_direct(
        &mut self,
        preset: &mut Preset,
        path: &ParamPath,
        value: f64,
    ) {
        match path {
            ParamPath::LayerParam {
                kind,
                layer_id,
                key,
            } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut preset.pipeline.operations,
                    PipelineKind::Lift => &mut preset.pipeline.lift,
                };
                if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                    if let Some(p) = layer.params.get_mut(key) {
                        p.value = value;
                    }
                }
            }
            ParamPath::LayerBlend { kind, layer_id } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut preset.pipeline.operations,
                    PipelineKind::Lift => &mut preset.pipeline.lift,
                };
                if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                    layer.blend.value = value;
                }
            }
            ParamPath::View(name) => {
                if let Some(tp) = Self::resolve_view_param_mut(preset, name) {
                    tp.param.value = value;
                }
            }
            ParamPath::Tempo => {
                preset.tempo.value = value;
            }
        }
    }

    /// Apply structural actions returned by tick() to the preset.
    pub fn apply_actions(
        &mut self,
        preset: &mut Preset,
        actions: &[TransitionAction],
    ) {
        for action in actions {
            match action {
                TransitionAction::DisableLayer { kind, layer_id } => {
                    let layers = match kind {
                        PipelineKind::Operations => &mut preset.pipeline.operations,
                        PipelineKind::Lift => &mut preset.pipeline.lift,
                    };
                    if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.enabled = false;
                    }
                }
                TransitionAction::RestoreBlendValue {
                    kind,
                    layer_id,
                    value,
                } => {
                    let layers = match kind {
                        PipelineKind::Operations => &mut preset.pipeline.operations,
                        PipelineKind::Lift => &mut preset.pipeline.lift,
                    };
                    if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                        layer.blend.value = *value;
                    }
                }
            }
        }
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn resolve_view_param_mut<'a>(
        preset: &'a mut Preset,
        name: &str,
    ) -> Option<&'a mut ToggleParam> {
        match name {
            "camera.position.orbitX" => Some(&mut preset.camera.position.orbit_x),
            "camera.position.orbitY" => Some(&mut preset.camera.position.orbit_y),
            "camera.position.distance" => Some(&mut preset.camera.position.distance),
            "camera.position.targetX" => Some(&mut preset.camera.position.target_x),
            "camera.position.targetY" => Some(&mut preset.camera.position.target_y),
            "camera.position.roll" => Some(&mut preset.camera.position.roll),
            "camera.lens.focalLength" => Some(&mut preset.camera.lens.focal_length),
            "camera.lens.sensorSize" => Some(&mut preset.camera.lens.sensor_size),
            "camera.lens.orthoZoom" => Some(&mut preset.camera.lens.ortho_zoom),
            "camera.lens.anamorphic" => Some(&mut preset.camera.lens.anamorphic),
            "camera.lens.shiftX" => Some(&mut preset.camera.lens.shift_x),
            "camera.lens.shiftY" => Some(&mut preset.camera.lens.shift_y),
            "camera.distortion.k1" => Some(&mut preset.camera.distortion.k1),
            "camera.distortion.k2" => Some(&mut preset.camera.distortion.k2),
            "sampling.scale" => Some(&mut preset.sampling.scale),
            "sampling.density" => Some(&mut preset.sampling.density),
            _ => None,
        }
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Internal resolved-value structs (not part of public API)
// =============================================================================

struct ResolvedLayer {
    id: String,
    type_key: String,
    enabled: bool,
    blend: f64,
    params: HashMap<String, f64>,
}

#[allow(dead_code)]
struct CameraResolved {
    orbit_x: f64,
    orbit_y: f64,
    distance: f64,
    target_x: f64,
    target_y: f64,
    roll: f64,
    focal_length: f64,
    sensor_size: f64,
    ortho_zoom: f64,
    anamorphic: f64,
    shift_x: f64,
    shift_y: f64,
    k1: f64,
    k2: f64,
}

struct SamplingResolved {
    scale: f64,
    density: f64,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::TAU;
    use crate::layer::Layer;
    use crate::preset::Preset;
    use crate::types::{Param, ShapeFunction};

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    fn make_test_preset_with_animated_layer() -> Preset {
        let mut preset = Preset::default();
        let mut params = HashMap::new();
        params.insert(
            "theta".to_string(),
            Param {
                value: 0.0,
                min: 0.0,
                max: 1.0,
                anim: true,
                ratio: 1.0,
                offset: 0.0,
                shape: ShapeFunction::Cosine,
                modulator: None,
            },
        );
        preset.pipeline.operations.push(Layer {
            id: "test-layer-1".to_string(),
            type_key: "rotate".to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params,
        });
        preset
    }

    #[test]
    fn engine_resolves_animated_layer_param() {
        let preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        // Tick with dt that advances master phase
        // tempo=0.3, dt=0.25 -> master_phase = 0.25*0.3 = 0.075
        let (frame, _actions) = engine.tick(&preset, 0.25, 0.25);
        let theta = frame.operations[0].params["theta"];
        assert!(theta > 0.0, "theta should have moved from 0: {theta}");
        assert!(theta < 1.0, "theta should be less than max: {theta}");
    }

    #[test]
    fn layer_toggle_starts_blend_transition() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        assert!(preset.pipeline.operations[0].enabled);
        assert_near(preset.pipeline.operations[0].blend.value, 1.0, 1e-10);

        engine.toggle_layer(
            &mut preset,
            PipelineKind::Operations,
            "test-layer-1",
            0.5,
        );
        assert_eq!(engine.transitions.active_count(), 1);

        let (frame, _actions) = engine.tick(&preset, 0.0, 0.25);
        let blend = frame.operations[0].blend;
        assert!(blend < 1.0, "blend should have decreased: {blend}");
        assert!(blend > 0.0, "blend should not be 0 yet: {blend}");

        let (_frame, actions) = engine.tick(&preset, 0.0, 0.5);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            TransitionAction::DisableLayer {
                kind: PipelineKind::Operations,
                layer_id: "test-layer-1".to_string(),
            }
        );
        engine.apply_actions(&mut preset, &actions);
        assert!(!preset.pipeline.operations[0].enabled);
        assert_eq!(engine.transitions.active_count(), 0);
    }

    #[test]
    fn layer_toggle_on_enables_and_fades_in() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        preset.pipeline.operations[0].enabled = false;
        preset.pipeline.operations[0].blend.value = 0.0;

        engine.toggle_layer(
            &mut preset,
            PipelineKind::Operations,
            "test-layer-1",
            1.0,
        );
        assert!(preset.pipeline.operations[0].enabled);
        assert_near(preset.pipeline.operations[0].blend.value, 0.0, 1e-10);

        let (frame, _) = engine.tick(&preset, 0.0, 0.5);
        assert_near(frame.operations[0].blend, 0.5, 1e-10);

        let (frame, _) = engine.tick(&preset, 0.0, 0.5);
        assert_near(frame.operations[0].blend, 1.0, 1e-10);
    }

    #[test]
    fn deterministic_mode() {
        let preset1 = make_test_preset_with_animated_layer();
        let preset2 = preset1.clone();
        let mut engine1 = AnimationEngine::new();
        let mut engine2 = AnimationEngine::new();

        let dt = 1.0 / 60.0;
        let mut frame1 = None;
        let mut frame2 = None;
        for _ in 0..120 {
            frame1 = Some(engine1.tick(&preset1, dt, dt).0);
            frame2 = Some(engine2.tick(&preset2, dt, dt).0);
        }

        let v1 = frame1.unwrap().operations[0].params["theta"];
        let v2 = frame2.unwrap().operations[0].params["theta"];
        assert_near(v1, v2, 1e-12);
    }

    #[test]
    fn quaternion_drift_produces_values() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.mode = QuaternionMode::Drift;
        preset.quaternion.drift_speed = 1.0;

        let mut engine = AnimationEngine::new();
        let (frame, _) = engine.tick(&preset, 1.0, 1.0);

        let [ax, ay, az] = frame.quat_axis;

        let expected_ax = (0.7_f64).sin() + 0.3 * (1.3_f64).cos();
        let expected_ay = (0.5_f64).cos() + 0.3 * (1.7_f64).sin();
        let expected_az = (2.1_f64).sin();

        assert_near(ax, expected_ax, 1e-10);
        assert_near(ay, expected_ay, 1e-10);
        assert_near(az, expected_az, 1e-10);
    }

    #[test]
    fn quaternion_fixed_axis_animation() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.angle.anim = false;
        preset.quaternion.mode = QuaternionMode::Fixed;
        preset.quaternion.ax.anim = true;
        preset.quaternion.ax.ratio = 1.0;
        preset.quaternion.ax.shape = ShapeFunction::Cosine;

        let mut engine = AnimationEngine::new();
        // dt=0.25 -> master_phase = 0.25 * 0.3 = 0.075
        // ax: phase = 0.075*1.0 = 0.075, cosine -> t = 0.5-0.5*cos(0.075*TAU)
        // value = -1.0 + t * 2.0
        let (frame, _) = engine.tick(&preset, 0.25, 0.25);
        let master_phase = 0.25 * 0.3; // 0.075
        let expected_t = 0.5 - 0.5 * (master_phase * TAU).cos();
        let expected_ax = -1.0 + expected_t * 2.0;
        assert_near(frame.quat_axis[0], expected_ax, 1e-10);
    }

    #[test]
    fn reset_clears_all_state() {
        let preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        engine.tick(&preset, 1.0, 1.0);
        assert!(engine.master_phase > 0.0);

        engine.reset();

        assert_near(engine.master_phase, 0.0, 1e-15);
        assert_eq!(engine.transitions.active_count(), 0);
        assert!(engine.anim_weights.is_empty());
        assert_near(engine.drift_phase, 0.0, 1e-15);
    }

    #[test]
    fn camera_toggle_params_resolve_when_on_and_anim() {
        let mut preset = Preset::default();
        preset.camera.position.orbit_x.on = true;
        preset.camera.position.orbit_x.param.anim = true;
        preset.camera.position.orbit_x.param.shape = ShapeFunction::Cosine;
        preset.camera.position.orbit_x.param.ratio = 1.0;
        let initial = preset.camera.position.orbit_x.param.value;

        let mut engine = AnimationEngine::new();
        let (frame, _) = engine.tick(&preset, 0.5, 0.5);

        assert!(
            (frame.orbit_x - initial).abs() > 1e-6,
            "orbit_x should have changed: initial={initial}, resolved={}",
            frame.orbit_x
        );
    }

    #[test]
    fn disabled_layers_not_resolved() {
        let mut preset = make_test_preset_with_animated_layer();
        preset.pipeline.operations[0].enabled = false;
        let initial = preset.pipeline.operations[0].params["theta"].value;

        let mut engine = AnimationEngine::new();
        let (frame, _) = engine.tick(&preset, 0.5, 0.5);

        assert_near(frame.operations[0].params["theta"], initial, 1e-15);
        assert_near(frame.operations[0].blend, 0.0, 1e-15);
    }

    #[test]
    fn set_param_direct_sets_value() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();
        let path = ParamPath::LayerParam {
            kind: PipelineKind::Operations,
            layer_id: "test-layer-1".to_string(),
            key: "theta".to_string(),
        };
        engine.set_param_direct(&mut preset, &path, 0.42);
        assert_near(
            preset.pipeline.operations[0].params["theta"].value,
            0.42,
            1e-15,
        );
    }

    #[test]
    fn preset_not_mutated_after_tick() {
        let preset = make_test_preset_with_animated_layer();
        let preset_clone = preset.clone();
        let mut engine = AnimationEngine::new();

        for _ in 0..10 {
            engine.tick(&preset, 0.1, 0.1);
        }

        assert_eq!(preset, preset_clone);
    }

    #[test]
    fn frame_state_has_correct_resolved_values() {
        let preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        let (frame, _) = engine.tick(&preset, 0.25, 0.25);

        // tempo=0.3, dt=0.25 -> master_phase = 0.075
        // theta: ratio=1, offset=0, cosine shape
        // phase = 0.075, t = 0.5 - 0.5*cos(0.075*TAU)
        let master_phase = 0.25 * 0.3;
        let expected_t = 0.5 - 0.5 * (master_phase * TAU).cos();
        let expected_theta = 0.0 + expected_t * (1.0 - 0.0);
        assert_near(
            frame.operations[0].params["theta"],
            expected_theta,
            1e-10,
        );
    }

    #[test]
    fn weight_transition_smooth() {
        let mut engine = AnimationEngine::new();
        let path = ParamPath::View("test".to_string());

        // Set weight target from 1.0 to 0.0 over 1 second
        engine.set_weight_target(path.clone(), 0.0, 1.0);

        // Advance by 0.5s
        engine.advance_weights(0.5);
        assert_near(engine.get_weight(&path), 0.5, 1e-10);

        // Advance by 0.5s — should reach target
        engine.advance_weights(0.5);
        assert_near(engine.get_weight(&path), 0.0, 1e-10);
    }

    #[test]
    fn weight_default_is_one() {
        let engine = AnimationEngine::new();
        let path = ParamPath::View("nonexistent".to_string());
        assert_near(engine.get_weight(&path), 1.0, 1e-15);
    }
}
