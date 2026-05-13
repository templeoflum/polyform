use std::collections::HashMap;
// TAU used in tests
#[allow(unused_imports)]
use std::f64::consts::TAU;

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
    /// Master phase M (unbounded, grows monotonically). Driven by dM/dt = tempo.
    pub master_phase: f64,
    /// Smooth weight transitions for anim on/off.
    anim_weights: HashMap<ParamPath, AnimWeight>,
    /// Cosine-eased transition manager (structural: blend fades, preset transitions).
    pub transitions: TransitionManager,
    /// Wall-clock time (drives transitions, runs even when paused).
    wall_clock: f64,
    /// Transition value overlay (public for toggle-on cancel path).
    pub transition_overlay: HashMap<TransitionTarget, f64>,
    /// Quaternion drift phase accumulator (deliberately decoupled from master phase).
    drift_phase: f64,
    /// Stored quaternion for smooth drift↔fixed mode transitions.
    prev_mode_quat: Option<[f64; 4]>,
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
            prev_mode_quat: None,
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
        self.prev_mode_quat = None;
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
            prev_mode_quat: self.prev_mode_quat,
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

    /// Start weight transition from a specific value (not from the current weight).
    pub fn start_weight_transition(&mut self, path: ParamPath, from: f64, to: f64, duration: f64) {
        let rate = if duration > 0.0 { 1.0 / duration } else { f64::INFINITY };
        self.anim_weights.insert(path, AnimWeight {
            current: from,
            target: to,
            rate,
        });
    }

    /// Check whether an explicit weight entry exists for a path.
    pub fn has_weight(&self, path: &ParamPath) -> bool {
        self.anim_weights.contains_key(path)
    }

    /// Remove a weight entry (revert to default 1.0).
    pub fn remove_weight(&mut self, path: &ParamPath) {
        self.anim_weights.remove(path);
    }

    /// Store a quaternion snapshot for smooth mode transitions (drift ↔ fixed).
    pub fn set_prev_mode_quat(&mut self, q: Option<[f64; 4]>) {
        self.prev_mode_quat = q;
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
    /// - `tempo_value`: current tempo (Hz, from session state)
    /// - `palette_key`: current palette name (from session state)
    /// - `dt`: animation time delta (0 when paused)
    /// - `wall_dt`: wall-clock time delta (always positive)
    pub fn tick(
        &mut self,
        preset: &Preset,
        tempo_value: f64,
        palette_key: &str,
        dt: f64,
        wall_dt: f64,
    ) -> (FrameState, Vec<TransitionAction>) {
        // 1. Advance master phase (unbounded — each param's modular arithmetic
        //    handles its own phase wrapping, so fractional ratios work correctly)
        self.master_phase += dt * safe_ratio(tempo_value);

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
        let (quat_angle, quat_axis, quat) = self.resolve_quaternion_values(preset);
        let sampling_resolved = self.resolve_sampling_values(preset);
        let phosphor_resolved = self.resolve_phosphor_values(preset);

        // 6. Build FrameState
        let frame_state = self.build_frame_state(
            preset,
            palette_key,
            &op_resolved,
            &lift_resolved,
            &camera_resolved,
            quat_angle,
            quat_axis,
            quat,
            &sampling_resolved,
            &phosphor_resolved,
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
            k3: self.resolve_toggle_param(&preset.camera.distortion.k3, "camera.distortion.k3"),
            k4: self.resolve_toggle_param(&preset.camera.distortion.k4, "camera.distortion.k4"),
            p1: self.resolve_toggle_param(&preset.camera.distortion.p1, "camera.distortion.p1"),
            p2: self.resolve_toggle_param(&preset.camera.distortion.p2, "camera.distortion.p2"),
            distortion_cx: self.resolve_toggle_param(&preset.camera.distortion.center_x, "camera.distortion.centerX"),
            distortion_cy: self.resolve_toggle_param(&preset.camera.distortion.center_y, "camera.distortion.centerY"),
            squeeze: self.resolve_toggle_param(&preset.camera.distortion.squeeze, "camera.distortion.squeeze"),
        }
    }

    /// Resolve a ToggleParam using two independent weight channels:
    ///
    /// 1. **Toggle weight** (`view_key` path): blends neutral ↔ active (on/off toggle)
    /// 2. **Anim weight** (`view_key.anim` path): blends static ↔ animated (anim toggle)
    ///
    /// These are separate so toggling on/off and toggling anim don't interfere.
    fn resolve_toggle_param(&self, tp: &ToggleParam, view_key: &str) -> f64 {
        let toggle_path = ParamPath::View(view_key.to_string());
        let anim_path = ParamPath::View(format!("{}.anim", view_key));
        let neutral = Self::neutral_view_value(view_key);

        if !tp.on && !self.has_weight(&toggle_path) {
            return neutral;
        }

        let toggle_weight = self.get_weight(&toggle_path);
        let anim_weight = self.get_weight(&anim_path);

        // Resolve active value: uses anim_weight to blend static ↔ animated
        let active_value = resolve_param(&tp.param, self.master_phase, anim_weight);

        // Blend between neutral and active using toggle weight
        if toggle_weight >= 1.0 {
            active_value
        } else if toggle_weight <= 0.0 {
            neutral
        } else {
            (1.0 - toggle_weight) * neutral + toggle_weight * active_value
        }
    }

    /// Canonical neutral/rest value for each view param.
    /// These match the Preset::default() values — the "identity" camera state.
    pub fn neutral_view_value(view_key: &str) -> f64 {
        match view_key {
            "camera.position.orbitX" => 0.0,
            "camera.position.orbitY" => 0.0,
            "camera.position.distance" => 3.5,
            "camera.position.targetX" => 0.0,
            "camera.position.targetY" => 0.0,
            "camera.position.roll" => 0.0,
            "camera.lens.focalLength" => 35.0,
            "camera.lens.sensorSize" => 36.0,
            "camera.lens.orthoZoom" => 2.0,
            "camera.lens.anamorphic" => 1.0,
            "camera.lens.shiftX" => 0.0,
            "camera.lens.shiftY" => 0.0,
            "camera.distortion.k1" => 0.0,
            "camera.distortion.k2" => 0.0,
            "camera.distortion.k3" => 0.0,
            "camera.distortion.k4" => 0.0,
            "camera.distortion.p1" => 0.0,
            "camera.distortion.p2" => 0.0,
            "camera.distortion.centerX" => 0.0,
            "camera.distortion.centerY" => 0.0,
            "camera.distortion.squeeze" => 1.0,
            "sampling.scale" => 0.6,
            "sampling.density" => 27.0,
            "phosphor.persistence" => 0.0,
            "phosphor.beamWidth" => 1.5,
            "phosphor.beamIntensity" => 0.0,
            "phosphor.gain" => 1.0,
            "phosphor.blendMix" => 1.0,
            "phosphor.saturation" => 1.0,
            "phosphor.bloomIntensity" => 0.0,
            "phosphor.bloomRadius" => 2.0,
            "phosphor.bloomThreshold" => 0.3,
            "phosphor.gamma" => 1.0,
            _ => 0.0,
        }
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
    // Phosphor resolution
    // =========================================================================

    fn resolve_phosphor_values(&self, preset: &Preset) -> PhosphorResolved {
        PhosphorResolved {
            persistence: self.resolve_toggle_param(&preset.phosphor.persistence, "phosphor.persistence"),
            beam_width: self.resolve_toggle_param(&preset.phosphor.beam_width, "phosphor.beamWidth"),
            beam_intensity: self.resolve_toggle_param(&preset.phosphor.beam_intensity, "phosphor.beamIntensity"),
            gain: self.resolve_toggle_param(&preset.phosphor.gain, "phosphor.gain"),
            blend_mix: self.resolve_toggle_param(&preset.phosphor.blend_mix, "phosphor.blendMix"),
            saturation: self.resolve_toggle_param(&preset.phosphor.saturation, "phosphor.saturation"),
            bloom_intensity: self.resolve_toggle_param(&preset.phosphor.bloom_intensity, "phosphor.bloomIntensity"),
            bloom_radius: self.resolve_toggle_param(&preset.phosphor.bloom_radius, "phosphor.bloomRadius"),
            bloom_threshold: self.resolve_toggle_param(&preset.phosphor.bloom_threshold, "phosphor.bloomThreshold"),
            gamma: self.resolve_toggle_param(&preset.phosphor.gamma, "phosphor.gamma"),
        }
    }

    // =========================================================================
    // Quaternion resolution
    // =========================================================================

    /// Resolve quaternion values. Returns:
    /// - `angle`: resolved angle param value (for UI display)
    /// - `axis`: resolved axis values (for UI display)
    /// - `quat`: final [w,x,y,z] for rendering
    fn resolve_quaternion_values(&self, preset: &Preset) -> (f64, [f64; 3], [f64; 4]) {
        let q = &preset.quaternion;

        // Quaternion blend weight: drives smooth on/off transitions.
        let quat_blend_path = ParamPath::View("quaternion.blend".to_string());
        let quat_blend = if self.has_weight(&quat_blend_path) {
            self.get_weight(&quat_blend_path)
        } else if q.on {
            1.0
        } else {
            0.0
        };

        // Resolve angle and axis
        let angle_weight = self.get_weight(&ParamPath::View("quaternion.angle".to_string()));
        let resolved_angle = resolve_param(&q.angle, self.master_phase, angle_weight);

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

        if quat_blend <= 1e-12 {
            return (resolved_angle, axis, [1.0, 0.0, 0.0, 0.0]);
        }

        // Build target quaternion from axis-angle, scaled by blend.
        // Uses the wrapped angle (bounded in [min, max]) so that angle*blend
        // is at most TAU — no multi-rotation spinning during blend transitions.
        let effective_angle = resolved_angle * quat_blend;
        let mut final_quat = axis_angle_to_quat(effective_angle, axis);

        // Smooth drift↔fixed mode transition via NLERP
        let mode_blend_path = ParamPath::View("quaternion.mode_blend".to_string());
        if let Some(prev_q) = self.prev_mode_quat {
            if self.has_weight(&mode_blend_path) {
                let t = self.get_weight(&mode_blend_path);
                if t < 1.0 - 1e-12 {
                    final_quat = nlerp(prev_q, final_quat, t);
                }
            }
        }

        (resolved_angle, axis, final_quat)
    }

    // =========================================================================
    // FrameState construction
    // =========================================================================

    fn build_frame_state(
        &self,
        preset: &Preset,
        palette_key: &str,
        op_resolved: &[ResolvedLayer],
        lift_resolved: &[ResolvedLayer],
        camera_resolved: &CameraResolved,
        quat_angle: f64,
        quat_axis: [f64; 3],
        quat: [f64; 4],
        sampling_resolved: &SamplingResolved,
        phosphor_resolved: &PhosphorResolved,
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
        let k3 = self.resolve_view_transition("camera.distortion.k3", camera_resolved.k3);
        let k4 = self.resolve_view_transition("camera.distortion.k4", camera_resolved.k4);
        let p1 = self.resolve_view_transition("camera.distortion.p1", camera_resolved.p1);
        let p2 = self.resolve_view_transition("camera.distortion.p2", camera_resolved.p2);
        let distortion_cx = self.resolve_view_transition("camera.distortion.centerX", camera_resolved.distortion_cx);
        let distortion_cy = self.resolve_view_transition("camera.distortion.centerY", camera_resolved.distortion_cy);
        let squeeze = self.resolve_view_transition("camera.distortion.squeeze", camera_resolved.squeeze);

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
            k3,
            k4,
            p1,
            p2,
            distortion_cx,
            distortion_cy,
            squeeze,
            quat_on: {
                let qp = ParamPath::View("quaternion.blend".to_string());
                if self.has_weight(&qp) {
                    self.get_weight(&qp) > 1e-6
                } else {
                    preset.quaternion.on
                }
            },
            quat,
            quat_angle,
            quat_axis,
            scale: sampling_resolved.scale,
            density: sampling_resolved.density,
            lattice_type: preset.sampling.lattice_type,
            lattice_params: preset.sampling.lattice_params.clone(),
            persistence: self.resolve_view_transition("phosphor.persistence", phosphor_resolved.persistence),
            beam_width: self.resolve_view_transition("phosphor.beamWidth", phosphor_resolved.beam_width),
            beam_intensity: self.resolve_view_transition("phosphor.beamIntensity", phosphor_resolved.beam_intensity),
            gain: self.resolve_view_transition("phosphor.gain", phosphor_resolved.gain),
            blend_mix: self.resolve_view_transition("phosphor.blendMix", phosphor_resolved.blend_mix),
            saturation: self.resolve_view_transition("phosphor.saturation", phosphor_resolved.saturation),
            bloom_intensity: self.resolve_view_transition("phosphor.bloomIntensity", phosphor_resolved.bloom_intensity),
            bloom_radius: self.resolve_view_transition("phosphor.bloomRadius", phosphor_resolved.bloom_radius),
            bloom_threshold: self.resolve_view_transition("phosphor.bloomThreshold", phosphor_resolved.bloom_threshold),
            gamma: self.resolve_view_transition("phosphor.gamma", phosphor_resolved.gamma),
            palette_key: palette_key.to_string(),
            has_active_layers,
            show_caustic: false, // set by app from overlay state
            color_mode: 0,      // set by app from overlay state
            normal_shading: false, // set by app from overlay state
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
            "camera.distortion.k3" => Some(&mut preset.camera.distortion.k3),
            "camera.distortion.k4" => Some(&mut preset.camera.distortion.k4),
            "camera.distortion.p1" => Some(&mut preset.camera.distortion.p1),
            "camera.distortion.p2" => Some(&mut preset.camera.distortion.p2),
            "camera.distortion.centerX" => Some(&mut preset.camera.distortion.center_x),
            "camera.distortion.centerY" => Some(&mut preset.camera.distortion.center_y),
            "camera.distortion.squeeze" => Some(&mut preset.camera.distortion.squeeze),
            "sampling.scale" => Some(&mut preset.sampling.scale),
            "sampling.density" => Some(&mut preset.sampling.density),
            "phosphor.persistence" => Some(&mut preset.phosphor.persistence),
            "phosphor.beamWidth" => Some(&mut preset.phosphor.beam_width),
            "phosphor.beamIntensity" => Some(&mut preset.phosphor.beam_intensity),
            "phosphor.gain" => Some(&mut preset.phosphor.gain),
            "phosphor.blendMix" => Some(&mut preset.phosphor.blend_mix),
            "phosphor.saturation" => Some(&mut preset.phosphor.saturation),
            "phosphor.bloomIntensity" => Some(&mut preset.phosphor.bloom_intensity),
            "phosphor.bloomRadius" => Some(&mut preset.phosphor.bloom_radius),
            "phosphor.bloomThreshold" => Some(&mut preset.phosphor.bloom_threshold),
            "phosphor.gamma" => Some(&mut preset.phosphor.gamma),
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
// Quaternion helpers
// =============================================================================

/// Convert axis-angle to quaternion [w, x, y, z].
fn axis_angle_to_quat(angle: f64, axis: [f64; 3]) -> [f64; 4] {
    let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if len < 1e-12 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let half = angle / 2.0;
    let (s, c) = half.sin_cos();
    let n = s / len;
    [c, axis[0] * n, axis[1] * n, axis[2] * n]
}

/// Normalized linear interpolation between two quaternions.
/// Uses shortest-path (negates if dot < 0). Suitable for short transitions.
fn nlerp(q1: [f64; 4], q2: [f64; 4], t: f64) -> [f64; 4] {
    let dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];
    let sign = if dot < 0.0 { -1.0 } else { 1.0 };
    let w = (1.0 - t) * q1[0] + t * sign * q2[0];
    let x = (1.0 - t) * q1[1] + t * sign * q2[1];
    let y = (1.0 - t) * q1[2] + t * sign * q2[2];
    let z = (1.0 - t) * q1[3] + t * sign * q2[3];
    let len = (w * w + x * x + y * y + z * z).sqrt();
    if len < 1e-12 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [w / len, x / len, y / len, z / len]
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
    k3: f64,
    k4: f64,
    p1: f64,
    p2: f64,
    distortion_cx: f64,
    distortion_cy: f64,
    squeeze: f64,
}

struct SamplingResolved {
    scale: f64,
    density: f64,
}

struct PhosphorResolved {
    persistence: f64,
    beam_width: f64,
    beam_intensity: f64,
    gain: f64,
    blend_mix: f64,
    saturation: f64,
    bloom_intensity: f64,
    bloom_radius: f64,
    bloom_threshold: f64,
    gamma: f64,
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

    const TEST_TEMPO: f64 = 0.3;
    const TEST_PALETTE: &str = "phosphor";

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
        let (frame, _actions) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.25, 0.25);
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

        let (frame, _actions) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.0, 0.25);
        let blend = frame.operations[0].blend;
        assert!(blend < 1.0, "blend should have decreased: {blend}");
        assert!(blend > 0.0, "blend should not be 0 yet: {blend}");

        let (_frame, actions) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.0, 0.5);
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

        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.0, 0.5);
        assert_near(frame.operations[0].blend, 0.5, 1e-10);

        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.0, 0.5);
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
            frame1 = Some(engine1.tick(&preset1, TEST_TEMPO, TEST_PALETTE, dt, dt).0);
            frame2 = Some(engine2.tick(&preset2, TEST_TEMPO, TEST_PALETTE, dt, dt).0);
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
        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 1.0, 1.0);

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
        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.25, 0.25);
        let master_phase = 0.25 * 0.3; // 0.075
        let expected_t = 0.5 - 0.5 * (master_phase * TAU).cos();
        let expected_ax = -1.0 + expected_t * 2.0;
        assert_near(frame.quat_axis[0], expected_ax, 1e-10);
    }

    #[test]
    fn reset_clears_all_state() {
        let preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 1.0, 1.0);
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
        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.5, 0.5);

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
        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.5, 0.5);

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
            engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.1, 0.1);
        }

        assert_eq!(preset, preset_clone);
    }

    #[test]
    fn frame_state_has_correct_resolved_values() {
        let preset = make_test_preset_with_animated_layer();
        let mut engine = AnimationEngine::new();

        let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 0.25, 0.25);

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

    // ================================================================
    // Bug reproduction tests
    // ================================================================

    /// BUG: orbit_x jumps when anim is enabled.
    /// The toggle weight (on/off) and anim weight share the same path.
    /// When anim is enabled, we skip the weight transition for ToggleParams
    /// (to avoid conflicting with toggle weight), so anim switches instantly
    /// from static value to animated value — producing a jump.
    ///
    /// FIX: resolve_toggle_param should use a SEPARATE weight path for
    /// anim blending (".anim" suffix), so both systems work independently.
    #[test]
    fn camera_anim_enable_should_not_jump() {
        let mut preset = Preset::default();
        // orbit_x: on=true, anim=false, value=0.0
        assert!(preset.camera.position.orbit_x.on);
        assert!(!preset.camera.position.orbit_x.param.anim);

        let mut engine = AnimationEngine::new();

        // Run a few frames to get a stable master_phase
        for _ in 0..60 {
            engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 1.0 / 60.0, 1.0 / 60.0);
        }

        // Capture pre-anim orbit_x value
        let (frame_before, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 1.0 / 60.0, 1.0 / 60.0);
        let before = frame_before.orbit_x;

        // Enable anim — simulates what SetParamAnim does
        preset.camera.position.orbit_x.param.anim = true;
        // For ToggleParams, the anim weight should use a separate ".anim" path
        let anim_path = ParamPath::View("camera.position.orbitX.anim".to_string());
        engine.start_weight_transition(anim_path, 0.0, 1.0, 0.3);

        // Next frame: orbit_x should be close to before (weight ≈ 0, still static)
        let (frame_after, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, 1.0 / 60.0, 1.0 / 60.0);
        let after = frame_after.orbit_x;
        let jump = (after - before).abs();
        // The first frame will advance the weight by ~dt/0.3 ≈ 0.055, so a small
        // blend step is expected. The bug was a jump of ~1.95 rad (instant switch).
        assert!(
            jump < 0.5,
            "orbit_x jumped by {jump} when anim was enabled (before={before}, after={after})"
        );
    }

    /// BUG: orbit_y with ratio=1/phi doesn't complete a full sawtooth cycle.
    /// Because master_phase wraps at 1.0, and orbit_y's phase is
    /// (master_phase * 0.618) — when master wraps, the phase jumps from
    /// 0.618 back to 0.0 instead of continuing to 1.0.
    ///
    /// FIX: master_phase should NOT wrap at 1.0. Let it grow unbounded.
    /// Each param's modular arithmetic handles its own phase wrapping.
    #[test]
    fn sawtooth_fractional_ratio_no_discontinuity() {
        let mut preset = Preset::default();
        preset.camera.position.orbit_y.on = true;
        preset.camera.position.orbit_y.param.anim = true;
        preset.camera.position.orbit_y.param.shape = ShapeFunction::Sawtooth;
        // ratio=1/phi ≈ 0.618 — takes ~1.618 master cycles to complete one param cycle
        preset.camera.position.orbit_y.param.ratio = 1.0 / 1.618033988749895;

        let mut engine = AnimationEngine::new();
        let dt = 1.0 / 60.0;
        let mut prev_orbit_y = None;
        let mut max_jump = 0.0_f64;

        // Run for 5 seconds (300 frames) — should cross master_phase=1.0 multiple times
        for _ in 0..300 {
            let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE,dt, dt);
            if let Some(prev) = prev_orbit_y {
                let diff: f64 = frame.orbit_y - prev;
                let jump = diff.abs();
                max_jump = max_jump.max(jump);
            }
            prev_orbit_y = Some(frame.orbit_y);
        }

        // With sawtooth and ratio < 1, a continuous ramp should have small
        // frame-to-frame deltas. The master_phase wrapping bug would cause
        // jumps of ~0.6 * (max-min) ≈ 1.8 radians.
        let range = preset.camera.position.orbit_y.param.max
            - preset.camera.position.orbit_y.param.min;
        assert!(
            max_jump < range * 0.05,
            "orbit_y had discontinuity: max frame-to-frame jump was {max_jump} \
             (range={range}, threshold={})",
            range * 0.05
        );
    }

    /// Verify quaternion rotation is smooth at steady state (blend=1.0).
    /// The sawtooth angle wraps from TAU to 0, but the quaternion
    /// representation makes this visually smooth (TAU and 0 are
    /// the same rotation).
    #[test]
    fn quat_steady_state_no_discontinuity() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.angle.anim = true;
        preset.quaternion.angle.shape = ShapeFunction::Sawtooth;
        preset.quaternion.angle.ratio = 1.0;
        preset.quaternion.mode = QuaternionMode::Fixed;
        preset.quaternion.ax.value = 0.0;
        preset.quaternion.ax.anim = false;
        preset.quaternion.ay.value = 1.0;
        preset.quaternion.ay.anim = false;
        preset.quaternion.az.value = 0.0;
        preset.quaternion.az.anim = false;
        let fast_tempo = 1.0; // fast tempo to hit wrap point quickly

        let mut engine = AnimationEngine::new();
        // blend=1.0 (steady state, no transition)

        let dt = 1.0 / 60.0;
        let mut prev_quat: Option<[f64; 4]> = None;
        let mut max_angular_jump = 0.0_f64;

        // Run for 3 seconds — should cross the angle wrap point multiple times
        for _ in 0..180 {
            let (frame, _) = engine.tick(&preset, fast_tempo, TEST_PALETTE, dt, dt);
            if let Some(prev) = prev_quat {
                // Quaternion angular distance: 2 * acos(|q1 · q2|)
                let dot = (prev[0] * frame.quat[0]
                    + prev[1] * frame.quat[1]
                    + prev[2] * frame.quat[2]
                    + prev[3] * frame.quat[3])
                    .abs()
                    .min(1.0);
                let angular_dist = 2.0 * dot.acos();
                max_angular_jump = max_angular_jump.max(angular_dist);
            }
            prev_quat = Some(frame.quat);
        }

        // At blend=1.0, axis_angle_to_quat is smooth across the sawtooth wrap
        // because sin/cos are periodic (TAU rotation = identity = 0 rotation).
        // Max per-frame step should be proportional to dt * tempo * TAU.
        assert!(
            max_angular_jump < 0.5,
            "quaternion had discontinuity at steady state: max jump was {max_angular_jump} rad"
        );
    }

    /// Verify that the blend transition doesn't cause multi-rotation spinning.
    /// The wrapped angle approach limits per-frame rotation to at most one cycle,
    /// unlike the continuous-angle approach which would cause dozens of spins.
    #[test]
    fn quat_blend_no_multi_spin() {
        let mut preset = Preset::default();
        preset.quaternion.on = true;
        preset.quaternion.angle.anim = true;
        preset.quaternion.angle.shape = ShapeFunction::Sawtooth;
        preset.quaternion.angle.ratio = 1.0;
        preset.quaternion.mode = QuaternionMode::Fixed;
        preset.quaternion.ay.value = 1.0;

        let mut engine = AnimationEngine::new();
        // Run for 10 seconds to accumulate many rotations
        let dt = 1.0 / 60.0;
        for _ in 0..600 {
            engine.tick(&preset, TEST_TEMPO, TEST_PALETTE, dt, dt);
        }

        // Now toggle quat ON with a blend transition
        let blend_path = ParamPath::View("quaternion.blend".to_string());
        engine.start_weight_transition(blend_path, 0.0, 1.0, 0.3);

        let mut prev_quat: Option<[f64; 4]> = None;
        let mut total_angular_travel = 0.0_f64;

        // Track total angular distance over the 0.3s blend transition
        for _ in 0..18 {
            // 18 frames at 60fps = 0.3s
            let (frame, _) = engine.tick(&preset, TEST_TEMPO, TEST_PALETTE,dt, dt);
            if let Some(prev) = prev_quat {
                let dot = (prev[0] * frame.quat[0]
                    + prev[1] * frame.quat[1]
                    + prev[2] * frame.quat[2]
                    + prev[3] * frame.quat[3])
                    .abs()
                    .min(1.0);
                let angular_dist = 2.0 * dot.acos();
                total_angular_travel += angular_dist;
            }
            prev_quat = Some(frame.quat);
        }

        // With the wrapped approach, total rotation during blend should be
        // at most ~TAU (one full turn). The continuous approach would give
        // ~10*TAU (ten turns) which looks like wild spinning.
        assert!(
            total_angular_travel < TAU * 2.0,
            "blend transition caused excessive spinning: total={total_angular_travel:.2} rad \
             ({:.1} rotations)",
            total_angular_travel / TAU
        );
    }
}
