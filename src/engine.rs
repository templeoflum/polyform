use std::collections::HashMap;
use std::f64::consts::TAU;

use crate::animation::{safe_ratio, sweep_param, LoopState, ParamPath, PipelineKind, ResyncState};
use crate::preset::Preset;
use crate::transition::{
    TransitionAction, TransitionManager, TransitionTarget, TransitionUpdate,
};
use crate::types::{QuaternionMode, ToggleParam};

// =============================================================================
// AnimationEngine
// =============================================================================

/// Orchestrates all animation: param sweeping, transitions, and quaternion tumble.
///
/// The caller controls timing:
/// - **Live**: `dt = wall_clock_delta`, `wall_dt = wall_clock_delta`
/// - **Offline**: `dt = 1.0/fps`, `wall_dt = 1.0/fps`
/// - **Paused**: `dt = 0.0`, `wall_dt = wall_clock_delta`
pub struct AnimationEngine {
    /// Monotonic animation time (drives sweep oscillations).
    pub anim_clock: f64,
    /// Per-param loop accumulators, keyed by ParamPath.
    loop_states: HashMap<ParamPath, LoopState>,
    /// Resync blend states.
    resync_states: HashMap<ParamPath, ResyncState>,
    /// Quaternion angle accumulator (radians, wraps at TAU).
    quat_angle: f64,
    /// When drift mode started (for pseudo-random axis walk).
    drift_start_clock: f64,
    /// Cosine-eased transition manager.
    pub transitions: TransitionManager,
    /// Wall-clock time (drives transitions, runs even when paused).
    wall_clock: f64,
}

impl AnimationEngine {
    pub fn new() -> Self {
        Self {
            anim_clock: 0.0,
            loop_states: HashMap::new(),
            resync_states: HashMap::new(),
            quat_angle: 0.0,
            drift_start_clock: 0.0,
            transitions: TransitionManager::new(),
            wall_clock: 0.0,
        }
    }

    /// Clear all engine state (call on preset load).
    pub fn reset(&mut self) {
        self.anim_clock = 0.0;
        self.loop_states.clear();
        self.resync_states.clear();
        self.quat_angle = 0.0;
        self.drift_start_clock = 0.0;
        self.transitions = TransitionManager::new();
        self.wall_clock = 0.0;
    }

    /// Deep clone for offline rendering.
    pub fn snapshot(&self) -> Self {
        Self {
            anim_clock: self.anim_clock,
            loop_states: self.loop_states.clone(),
            resync_states: self.resync_states.clone(),
            quat_angle: self.quat_angle,
            drift_start_clock: self.drift_start_clock,
            transitions: TransitionManager::new(), // Transitions don't carry over
            wall_clock: self.wall_clock,
        }
    }

    // =========================================================================
    // Main tick
    // =========================================================================

    /// Single entry point called once per frame.
    ///
    /// - `dt`: animation time delta (0 when paused)
    /// - `wall_dt`: wall-clock time delta (always positive)
    pub fn tick(&mut self, preset: &mut Preset, dt: f64, wall_dt: f64) {
        self.anim_clock += dt;
        self.wall_clock += wall_dt;

        let base_rate = safe_ratio(preset.tempo.value);

        // 1. Process transitions (wall-clock based)
        self.process_transitions(preset);

        // 2. Sweep operation layer params
        self.sweep_pipeline_layers(preset, dt, base_rate, PipelineKind::Operations);

        // 3. Sweep lift layer params
        self.sweep_pipeline_layers(preset, dt, base_rate, PipelineKind::Lift);

        // 4. Sweep camera ToggleParams
        self.sweep_camera(preset, dt, base_rate);

        // 5. Sweep quaternion
        self.sweep_quaternion(preset, dt, base_rate);

        // 6. Sweep sampling params
        self.sweep_sampling(preset, dt, base_rate);

        // 7. Expire completed resyncs
        self.expire_resyncs();
    }

    // =========================================================================
    // Transitions
    // =========================================================================

    fn process_transitions(&mut self, preset: &mut Preset) {
        let (updates, actions) = self.transitions.tick(self.wall_clock);

        for update in &updates {
            self.apply_transition_update(preset, update);
        }

        for action in &actions {
            self.apply_transition_action(preset, action);
        }
    }

    fn apply_transition_update(&self, preset: &mut Preset, update: &TransitionUpdate) {
        match &update.target {
            TransitionTarget::LayerBlend { kind, layer_id } => {
                let layers = match kind {
                    PipelineKind::Operations => &mut preset.pipeline.operations,
                    PipelineKind::Lift => &mut preset.pipeline.lift,
                };
                if let Some(layer) = layers.iter_mut().find(|l| l.id == *layer_id) {
                    layer.blend.value = update.value;
                }
            }
            TransitionTarget::ViewParam(name) => {
                if let Some(tp) = self.resolve_view_param_mut(preset, name) {
                    tp.param.value = update.value;
                }
            }
            TransitionTarget::Tempo => {
                preset.tempo.value = update.value;
            }
        }
    }

    fn apply_transition_action(&self, preset: &mut Preset, action: &TransitionAction) {
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
        }
    }

    // =========================================================================
    // Pipeline layer sweeping
    // =========================================================================

    fn sweep_pipeline_layers(
        &mut self,
        preset: &mut Preset,
        dt: f64,
        base_rate: f64,
        kind: PipelineKind,
    ) {
        let layers = match kind {
            PipelineKind::Operations => &mut preset.pipeline.operations,
            PipelineKind::Lift => &mut preset.pipeline.lift,
        };

        for layer in layers.iter_mut() {
            if !layer.enabled {
                continue;
            }

            // Collect keys first to avoid borrow conflict
            let keys: Vec<String> = layer.params.keys().cloned().collect();
            for key in keys {
                let path = ParamPath::LayerParam {
                    kind,
                    layer_id: layer.id.clone(),
                    key: key.clone(),
                };

                let loop_state = self.loop_states.entry(path.clone()).or_default();
                let resync = self.resync_states.get(&path);

                let param = &layer.params[&key];
                let new_val =
                    sweep_param(param, self.anim_clock, dt, base_rate, loop_state, resync);

                // Write back
                if let Some(p) = layer.params.get_mut(&key) {
                    p.value = new_val;
                }
            }
        }
    }

    // =========================================================================
    // Camera sweeping
    // =========================================================================

    fn sweep_camera(&mut self, preset: &mut Preset, dt: f64, base_rate: f64) {
        // Camera position
        self.sweep_toggle_param(
            &mut preset.camera.position.orbit_x,
            "camera.position.orbitX",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.position.orbit_y,
            "camera.position.orbitY",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.position.distance,
            "camera.position.distance",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.position.target_x,
            "camera.position.targetX",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.position.target_y,
            "camera.position.targetY",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.position.roll,
            "camera.position.roll",
            dt,
            base_rate,
        );

        // Camera lens
        self.sweep_toggle_param(
            &mut preset.camera.lens.focal_length,
            "camera.lens.focalLength",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.lens.sensor_size,
            "camera.lens.sensorSize",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.lens.ortho_zoom,
            "camera.lens.orthoZoom",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.lens.anamorphic,
            "camera.lens.anamorphic",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.lens.shift_x,
            "camera.lens.shiftX",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.lens.shift_y,
            "camera.lens.shiftY",
            dt,
            base_rate,
        );

        // Distortion
        self.sweep_toggle_param(
            &mut preset.camera.distortion.k1,
            "camera.distortion.k1",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.camera.distortion.k2,
            "camera.distortion.k2",
            dt,
            base_rate,
        );
    }

    /// Sweep a ToggleParam: only sweeps when `on && param.anim`.
    fn sweep_toggle_param(
        &mut self,
        tp: &mut ToggleParam,
        view_key: &str,
        dt: f64,
        base_rate: f64,
    ) {
        if !tp.on {
            return;
        }

        let path = ParamPath::View(view_key.to_string());
        let loop_state = self.loop_states.entry(path.clone()).or_default();
        let resync = self.resync_states.get(&path);

        let new_val = sweep_param(&tp.param, self.anim_clock, dt, base_rate, loop_state, resync);
        tp.param.value = new_val;
    }

    // =========================================================================
    // Sampling sweeping
    // =========================================================================

    fn sweep_sampling(&mut self, preset: &mut Preset, dt: f64, base_rate: f64) {
        self.sweep_toggle_param(
            &mut preset.sampling.scale,
            "sampling.scale",
            dt,
            base_rate,
        );
        self.sweep_toggle_param(
            &mut preset.sampling.density,
            "sampling.density",
            dt,
            base_rate,
        );
    }

    // =========================================================================
    // Quaternion animation
    // =========================================================================

    fn sweep_quaternion(&mut self, preset: &mut Preset, dt: f64, base_rate: f64) {
        let q = &mut preset.quaternion;
        if !q.on {
            return;
        }

        // Angle accumulation (loop-style)
        if q.anim {
            let speed = base_rate * safe_ratio(q.angle_ratio);
            self.quat_angle = (self.quat_angle + dt * speed * TAU) % TAU;
            q.angle = self.quat_angle;
        }

        // Axis animation
        match q.mode {
            QuaternionMode::Drift => {
                let t = (self.anim_clock - self.drift_start_clock) * q.drift_speed;
                q.axes.ax.value = (t * 0.7).sin() + 0.3 * (t * 1.3).cos();
                q.axes.ay.value = (t * 0.5).cos() + 0.3 * (t * 1.7).sin();
                q.axes.az.value = (t * 0.9 + 1.2).sin();
            }
            QuaternionMode::Fixed => {
                if q.axes.ax.anim {
                    q.axes.ax.value =
                        (self.anim_clock * base_rate * safe_ratio(q.axes.ax.ratio) * TAU).cos();
                }
                if q.axes.ay.anim {
                    q.axes.ay.value =
                        (self.anim_clock * base_rate * safe_ratio(q.axes.ay.ratio) * TAU).cos();
                }
                if q.axes.az.anim {
                    q.axes.az.value =
                        (self.anim_clock * base_rate * safe_ratio(q.axes.az.ratio) * TAU).cos();
                }
            }
        }
    }

    // =========================================================================
    // Convenience methods
    // =========================================================================

    /// Toggle a layer on/off with a blend transition.
    ///
    /// - Enabled layer: starts blend 1->0 transition, disables on completion.
    /// - Disabled layer: enables it with blend=0, starts blend 0->1 transition.
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

        // Cancel any existing blend transition for this layer
        self.transitions.cancel_for(&target);

        let duration = layer.blend_time_override.unwrap_or(blend_time);

        if layer.enabled {
            // Fade out: blend -> 0, then disable
            let current_blend = layer.blend.value;
            self.transitions.start(
                target,
                current_blend,
                0.0,
                duration,
                Some(TransitionAction::DisableLayer {
                    kind,
                    layer_id: layer_id.to_string(),
                }),
                Some(self.wall_clock),
            );
        } else {
            // Enable with blend=0, fade in: 0 -> 1
            layer.enabled = true;
            layer.blend.value = 0.0;
            self.transitions
                .start(target, 0.0, 1.0, duration, None, Some(self.wall_clock));
        }
    }

    /// Set a param value directly (for slider drag — no transition).
    pub fn set_param_direct(&mut self, preset: &mut Preset, path: &ParamPath, value: f64) {
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
                if let Some(tp) = self.resolve_view_param_mut(preset, name) {
                    tp.param.value = value;
                }
            }
            ParamPath::Tempo => {
                preset.tempo.value = value;
            }
            ParamPath::LayerParamPhaseOffset {
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
                        if let Some(po) = &mut p.phase_offset {
                            po.value = value;
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Resolve a view param name (e.g. "camera.position.orbitX") to a mutable ToggleParam.
    fn resolve_view_param_mut<'a>(
        &self,
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

    /// Remove expired resync states.
    fn expire_resyncs(&mut self) {
        self.resync_states
            .retain(|_, rs| self.anim_clock - rs.clock < 2.0);
    }
}

impl Default for AnimationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::Layer;
    use crate::preset::Preset;
    use crate::types::{AnimMode, Param};

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
                anim_mode: AnimMode::Sweep,
                ratio: 1.0,
                phase_offset: None,
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
    fn engine_sweeps_animated_layer_param() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        // Tick at dt=0.25 (quarter period for ratio=1, base_rate=tempo=0.3)
        // With tempo=0.3, speed = 0.3 * 1.0 = 0.3
        // At clock=0.25, phase = 0.25 * 0.3 = 0.075
        // t = 0.5 - 0.5*cos(0.075 * TAU) = 0.5 - 0.5*cos(0.4712..) ≈ 0.5 - 0.5*0.891 ≈ 0.0545
        engine.tick(&mut preset, 0.25, 0.25);
        let theta = preset.pipeline.operations[0].params["theta"].value;
        assert!(theta > 0.0, "theta should have moved from 0: {theta}");
        assert!(theta < 1.0, "theta should be less than max: {theta}");
    }

    #[test]
    fn layer_toggle_starts_blend_transition() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        assert!(preset.pipeline.operations[0].enabled);
        assert_near(preset.pipeline.operations[0].blend.value, 1.0, 1e-10);

        // Toggle off: should start blend 1->0 transition
        engine.toggle_layer(
            &mut preset,
            PipelineKind::Operations,
            "test-layer-1",
            0.5,
        );
        assert_eq!(engine.transitions.active_count(), 1);

        // Tick at midpoint of transition
        engine.tick(&mut preset, 0.0, 0.25);
        let blend = preset.pipeline.operations[0].blend.value;
        assert!(blend < 1.0, "blend should have decreased: {blend}");
        assert!(blend > 0.0, "blend should not be 0 yet: {blend}");

        // Tick past completion
        engine.tick(&mut preset, 0.0, 0.5);
        assert!(!preset.pipeline.operations[0].enabled);
        assert_eq!(engine.transitions.active_count(), 0);
    }

    #[test]
    fn layer_toggle_on_enables_and_fades_in() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        // Disable the layer first
        preset.pipeline.operations[0].enabled = false;
        preset.pipeline.operations[0].blend.value = 0.0;

        // Toggle on
        engine.toggle_layer(
            &mut preset,
            PipelineKind::Operations,
            "test-layer-1",
            1.0,
        );
        assert!(preset.pipeline.operations[0].enabled);
        assert_near(preset.pipeline.operations[0].blend.value, 0.0, 1e-10);

        // Tick to midpoint
        engine.tick(&mut preset, 0.0, 0.5);
        let blend = preset.pipeline.operations[0].blend.value;
        assert_near(blend, 0.5, 1e-10);

        // Tick to completion
        engine.tick(&mut preset, 0.0, 0.5);
        assert_near(preset.pipeline.operations[0].blend.value, 1.0, 1e-10);
    }

    #[test]
    fn deterministic_mode() {
        let mut preset1 = make_test_preset_with_animated_layer();
        let mut preset2 = preset1.clone();
        let mut engine1 = AnimationEngine::new();
        let mut engine2 = AnimationEngine::new();

        // Same dt sequence produces identical values
        let dt = 1.0 / 60.0;
        for _ in 0..120 {
            engine1.tick(&mut preset1, dt, dt);
            engine2.tick(&mut preset2, dt, dt);
        }

        let v1 = preset1.pipeline.operations[0].params["theta"].value;
        let v2 = preset2.pipeline.operations[0].params["theta"].value;
        assert_near(v1, v2, 1e-12);
    }

    #[test]
    fn quaternion_drift_produces_values() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.anim = true;
        preset.quaternion.mode = QuaternionMode::Drift;
        preset.quaternion.drift_speed = 1.0;

        let mut engine = AnimationEngine::new();
        engine.tick(&mut preset, 1.0, 1.0);

        // Drift should produce non-trivial axis values
        let ax = preset.quaternion.axes.ax.value;
        let ay = preset.quaternion.axes.ay.value;
        let az = preset.quaternion.axes.az.value;

        // At t=1.0*1.0=1.0:
        // ax = sin(0.7) + 0.3*cos(1.3)
        // ay = cos(0.5) + 0.3*sin(1.7)
        // az = sin(0.9+1.2) = sin(2.1)
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
        preset.quaternion.anim = false; // angle not animated
        preset.quaternion.mode = QuaternionMode::Fixed;
        preset.quaternion.axes.ax.anim = true;
        preset.quaternion.axes.ax.ratio = 1.0;

        let mut engine = AnimationEngine::new();
        // Tick to a known point
        // At clock=0.25, base_rate=0.3:
        // ax = cos(0.25 * 0.3 * 1.0 * TAU) = cos(0.075 * TAU)
        engine.tick(&mut preset, 0.25, 0.25);
        let expected_ax = (0.25 * 0.3 * 1.0 * TAU).cos();
        assert_near(preset.quaternion.axes.ax.value, expected_ax, 1e-10);
    }

    #[test]
    fn reset_clears_all_state() {
        let mut preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        // Build up some state
        engine.tick(&mut preset, 1.0, 1.0);
        engine.toggle_layer(
            &mut preset,
            PipelineKind::Operations,
            "test-layer-1",
            0.5,
        );

        assert!(engine.anim_clock > 0.0);
        assert!(engine.transitions.active_count() > 0);

        engine.reset();

        assert_near(engine.anim_clock, 0.0, 1e-15);
        assert_eq!(engine.transitions.active_count(), 0);
        assert!(engine.loop_states.is_empty());
        assert!(engine.resync_states.is_empty());
        assert_near(engine.quat_angle, 0.0, 1e-15);
    }

    #[test]
    fn camera_toggle_params_sweep_when_on_and_anim() {
        let mut preset = Preset::default();
        // Enable orbit_x animation
        preset.camera.position.orbit_x.on = true;
        preset.camera.position.orbit_x.param.anim = true;
        preset.camera.position.orbit_x.param.anim_mode = AnimMode::Sweep;
        preset.camera.position.orbit_x.param.ratio = 1.0;
        let initial = preset.camera.position.orbit_x.param.value;

        let mut engine = AnimationEngine::new();
        engine.tick(&mut preset, 0.5, 0.5);

        let swept = preset.camera.position.orbit_x.param.value;
        assert!(
            (swept - initial).abs() > 1e-6,
            "orbit_x should have changed: initial={initial}, swept={swept}"
        );
    }

    #[test]
    fn disabled_layers_not_swept() {
        let mut preset = make_test_preset_with_animated_layer();
        preset.pipeline.operations[0].enabled = false;
        let initial = preset.pipeline.operations[0].params["theta"].value;

        let mut engine = AnimationEngine::new();
        engine.tick(&mut preset, 0.5, 0.5);

        let after = preset.pipeline.operations[0].params["theta"].value;
        assert_near(after, initial, 1e-15);
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
        assert_near(preset.pipeline.operations[0].params["theta"].value, 0.42, 1e-15);
    }

    #[test]
    fn quaternion_angle_accumulates() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.anim = true;
        preset.quaternion.angle_ratio = 1.0;

        let mut engine = AnimationEngine::new();
        let dt = 0.1;
        let base_rate = safe_ratio(preset.tempo.value); // 0.3

        // After one tick: angle = dt * speed * TAU = 0.1 * 0.3 * TAU
        engine.tick(&mut preset, dt, dt);
        let expected = 0.1 * base_rate * TAU;
        assert_near(preset.quaternion.angle, expected, 1e-10);

        // After another tick: angle accumulates
        engine.tick(&mut preset, dt, dt);
        let expected2 = expected + 0.1 * base_rate * TAU;
        assert_near(preset.quaternion.angle, expected2, 1e-10);
    }
}
