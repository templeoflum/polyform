use std::f64::consts::PI;

use crate::layer::Layer;
use crate::preset::Preset;
use crate::types::{Param, ToggleParam};

// =============================================================================
// MorphState
// =============================================================================

/// Active morph between two presets.
#[derive(Debug, Clone)]
pub struct MorphState {
    pub source: Preset,
    pub target: Preset,
    pub duration: f64,
    pub elapsed: f64,
}

impl MorphState {
    pub fn new(source: Preset, target: Preset, duration: f64) -> Self {
        Self {
            source,
            target,
            duration: duration.max(0.001),
            elapsed: 0.0,
        }
    }

    /// Raw progress 0..1.
    pub fn progress(&self) -> f64 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Cosine-eased progress (smooth start/end).
    pub fn eased_progress(&self) -> f64 {
        let t = self.progress();
        0.5 * (1.0 - (t * PI).cos())
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Advance morph time. Returns true if morph just completed.
    pub fn advance(&mut self, dt: f64) -> bool {
        let was_complete = self.is_complete();
        self.elapsed += dt;
        !was_complete && self.is_complete()
    }

    /// Produce a blended preset at the current morph progress.
    pub fn blended_preset(&self) -> Preset {
        let t = self.eased_progress();
        blend_presets(&self.source, &self.target, t)
    }
}

// =============================================================================
// Preset blending
// =============================================================================

/// Blend two presets at parameter `t` (0.0 = source, 1.0 = target).
///
/// - Matching layers (same UUID): interpolate all Param fields
/// - Unmatched source layers: fade blend → 0.0
/// - Unmatched target layers: fade blend 0.0 → target blend
/// - Camera/quaternion/sampling/phosphor: interpolate all fields
pub fn blend_presets(source: &Preset, target: &Preset, t: f64) -> Preset {
    let mut result = target.clone();

    // Pipeline: operations
    result.pipeline.operations = blend_layers(
        &source.pipeline.operations,
        &target.pipeline.operations,
        t,
    );

    // Pipeline: lift
    result.pipeline.lift = blend_layers(
        &source.pipeline.lift,
        &target.pipeline.lift,
        t,
    );

    // Camera
    result.camera.position.orbit_x = blend_toggle_param(
        &source.camera.position.orbit_x, &target.camera.position.orbit_x, t,
    );
    result.camera.position.orbit_y = blend_toggle_param(
        &source.camera.position.orbit_y, &target.camera.position.orbit_y, t,
    );
    result.camera.position.distance = blend_toggle_param(
        &source.camera.position.distance, &target.camera.position.distance, t,
    );
    result.camera.position.target_x = blend_toggle_param(
        &source.camera.position.target_x, &target.camera.position.target_x, t,
    );
    result.camera.position.target_y = blend_toggle_param(
        &source.camera.position.target_y, &target.camera.position.target_y, t,
    );
    result.camera.position.roll = blend_toggle_param(
        &source.camera.position.roll, &target.camera.position.roll, t,
    );

    result.camera.lens.focal_length = blend_toggle_param(
        &source.camera.lens.focal_length, &target.camera.lens.focal_length, t,
    );
    result.camera.lens.sensor_size = blend_toggle_param(
        &source.camera.lens.sensor_size, &target.camera.lens.sensor_size, t,
    );
    result.camera.lens.ortho_zoom = blend_toggle_param(
        &source.camera.lens.ortho_zoom, &target.camera.lens.ortho_zoom, t,
    );
    result.camera.lens.anamorphic = blend_toggle_param(
        &source.camera.lens.anamorphic, &target.camera.lens.anamorphic, t,
    );
    result.camera.lens.shift_x = blend_toggle_param(
        &source.camera.lens.shift_x, &target.camera.lens.shift_x, t,
    );
    result.camera.lens.shift_y = blend_toggle_param(
        &source.camera.lens.shift_y, &target.camera.lens.shift_y, t,
    );

    result.camera.distortion.k1 = blend_toggle_param(
        &source.camera.distortion.k1, &target.camera.distortion.k1, t,
    );
    result.camera.distortion.k2 = blend_toggle_param(
        &source.camera.distortion.k2, &target.camera.distortion.k2, t,
    );
    result.camera.distortion.k3 = blend_toggle_param(
        &source.camera.distortion.k3, &target.camera.distortion.k3, t,
    );
    result.camera.distortion.k4 = blend_toggle_param(
        &source.camera.distortion.k4, &target.camera.distortion.k4, t,
    );
    result.camera.distortion.p1 = blend_toggle_param(
        &source.camera.distortion.p1, &target.camera.distortion.p1, t,
    );
    result.camera.distortion.p2 = blend_toggle_param(
        &source.camera.distortion.p2, &target.camera.distortion.p2, t,
    );
    result.camera.distortion.center_x = blend_toggle_param(
        &source.camera.distortion.center_x, &target.camera.distortion.center_x, t,
    );
    result.camera.distortion.center_y = blend_toggle_param(
        &source.camera.distortion.center_y, &target.camera.distortion.center_y, t,
    );
    result.camera.distortion.squeeze = blend_toggle_param(
        &source.camera.distortion.squeeze, &target.camera.distortion.squeeze, t,
    );

    // Quaternion: interpolate angle + axis params
    result.quaternion.angle = lerp_param(&source.quaternion.angle, &target.quaternion.angle, t);
    result.quaternion.ax = lerp_param(&source.quaternion.ax, &target.quaternion.ax, t);
    result.quaternion.ay = lerp_param(&source.quaternion.ay, &target.quaternion.ay, t);
    result.quaternion.az = lerp_param(&source.quaternion.az, &target.quaternion.az, t);
    result.quaternion.drift_speed = lerp(source.quaternion.drift_speed, target.quaternion.drift_speed, t);
    // on/mode: snap at midpoint
    result.quaternion.on = if t < 0.5 { source.quaternion.on } else { target.quaternion.on };
    result.quaternion.mode = if t < 0.5 { source.quaternion.mode } else { target.quaternion.mode };

    // Sampling
    result.sampling.scale = blend_toggle_param(&source.sampling.scale, &target.sampling.scale, t);
    result.sampling.density = blend_toggle_param(&source.sampling.density, &target.sampling.density, t);
    // Lattice type: snap at midpoint
    result.sampling.lattice_type = if t < 0.5 {
        source.sampling.lattice_type
    } else {
        target.sampling.lattice_type
    };

    // Phosphor
    result.phosphor.persistence = blend_toggle_param(
        &source.phosphor.persistence, &target.phosphor.persistence, t,
    );
    result.phosphor.beam_width = blend_toggle_param(
        &source.phosphor.beam_width, &target.phosphor.beam_width, t,
    );
    result.phosphor.beam_intensity = blend_toggle_param(
        &source.phosphor.beam_intensity, &target.phosphor.beam_intensity, t,
    );
    result.phosphor.gain = blend_toggle_param(
        &source.phosphor.gain, &target.phosphor.gain, t,
    );
    result.phosphor.blend_mix = blend_toggle_param(
        &source.phosphor.blend_mix, &target.phosphor.blend_mix, t,
    );
    result.phosphor.saturation = blend_toggle_param(
        &source.phosphor.saturation, &target.phosphor.saturation, t,
    );
    result.phosphor.bloom_intensity = blend_toggle_param(
        &source.phosphor.bloom_intensity, &target.phosphor.bloom_intensity, t,
    );
    result.phosphor.bloom_radius = blend_toggle_param(
        &source.phosphor.bloom_radius, &target.phosphor.bloom_radius, t,
    );
    result.phosphor.bloom_threshold = blend_toggle_param(
        &source.phosphor.bloom_threshold, &target.phosphor.bloom_threshold, t,
    );
    result.phosphor.gamma = blend_toggle_param(
        &source.phosphor.gamma, &target.phosphor.gamma, t,
    );

    // Metadata: take target
    result.metadata = target.metadata.clone();

    result
}

// =============================================================================
// Layer blending
// =============================================================================

fn blend_layers(source: &[Layer], target: &[Layer], t: f64) -> Vec<Layer> {
    let mut result = Vec::new();

    // Track which target layers have been matched
    let mut matched_targets: Vec<bool> = vec![false; target.len()];

    // For each source layer, find a matching target layer by UUID
    for src in source {
        if let Some(tgt_idx) = target.iter().position(|tl| tl.id == src.id) {
            // Matching layers: interpolate
            matched_targets[tgt_idx] = true;
            result.push(blend_layer(src, &target[tgt_idx], t));
        } else {
            // Source-only layer: fade out (blend → 0)
            let mut faded = src.clone();
            faded.blend.value = lerp(src.blend.value, 0.0, t);
            // Keep enabled but with fading blend
            result.push(faded);
        }
    }

    // Unmatched target layers: fade in (blend 0 → target blend)
    for (i, tgt) in target.iter().enumerate() {
        if !matched_targets[i] {
            let mut faded = tgt.clone();
            faded.blend.value = lerp(0.0, tgt.blend.value, t);
            result.push(faded);
        }
    }

    result
}

fn blend_layer(source: &Layer, target: &Layer, t: f64) -> Layer {
    let mut result = target.clone();

    // Blend the blend param
    result.blend = lerp_param(&source.blend, &target.blend, t);

    // Enabled: snap at midpoint
    result.enabled = if t < 0.5 { source.enabled } else { target.enabled };

    // Params: interpolate matching keys, use target defaults for new ones
    for (key, tgt_param) in &target.params {
        if let Some(src_param) = source.params.get(key) {
            result.params.insert(key.clone(), lerp_param(src_param, tgt_param, t));
        }
        // If key only in target, result already has target value (from clone)
    }

    result
}

// =============================================================================
// Interpolation helpers
// =============================================================================

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn lerp_param(a: &Param, b: &Param, t: f64) -> Param {
    Param {
        value: lerp(a.value, b.value, t),
        min: lerp(a.min, b.min, t),
        max: lerp(a.max, b.max, t),
        // Animation stays OFF during the morph. The morph interpolates only
        // static values; animation resumes naturally when the morph completes
        // and the target preset (with its anim flags) is assigned.
        anim: false,
        ratio: lerp(a.ratio, b.ratio, t),
        offset: lerp(a.offset, b.offset, t),
        shape: if t < 0.5 { a.shape } else { b.shape },
        modulator: if t < 0.5 { a.modulator.clone() } else { b.modulator.clone() },
    }
}

fn blend_toggle_param(a: &ToggleParam, b: &ToggleParam, t: f64) -> ToggleParam {
    ToggleParam {
        on: if t < 0.5 { a.on } else { b.on },
        param: lerp_param(&a.param, &b.param, t),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn morph_state_progress() {
        let source = Preset::default();
        let target = Preset::default();
        let mut morph = MorphState::new(source, target, 2.0);
        assert_eq!(morph.progress(), 0.0);
        assert!(!morph.is_complete());

        morph.advance(1.0);
        assert!((morph.progress() - 0.5).abs() < 1e-10);

        morph.advance(1.0);
        assert!(morph.is_complete());
        assert!((morph.progress() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn morph_eased_endpoints() {
        let source = Preset::default();
        let target = Preset::default();

        let morph_start = MorphState::new(source.clone(), target.clone(), 1.0);
        assert!((morph_start.eased_progress() - 0.0).abs() < 1e-10);

        let mut morph_end = MorphState::new(source, target, 1.0);
        morph_end.advance(1.0);
        assert!((morph_end.eased_progress() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn blend_at_zero_returns_source() {
        let mut source = Preset::default();
        source.camera.position.orbit_x.param.value = 1.0;
        let mut target = Preset::default();
        target.camera.position.orbit_x.param.value = 2.0;

        let blended = blend_presets(&source, &target, 0.0);
        assert!((blended.camera.position.orbit_x.param.value - 1.0).abs() < 1e-10);
    }

    #[test]
    fn blend_at_one_returns_target() {
        let mut source = Preset::default();
        source.camera.position.orbit_x.param.value = 1.0;
        let mut target = Preset::default();
        target.camera.position.orbit_x.param.value = 2.0;

        let blended = blend_presets(&source, &target, 1.0);
        assert!((blended.camera.position.orbit_x.param.value - 2.0).abs() < 1e-10);
    }

    #[test]
    fn blend_at_half_is_midpoint() {
        let mut source = Preset::default();
        source.camera.position.distance.param.value = 2.0;
        let mut target = Preset::default();
        target.camera.position.distance.param.value = 6.0;

        let blended = blend_presets(&source, &target, 0.5);
        assert!((blended.camera.position.distance.param.value - 4.0).abs() < 1e-10);
    }

    #[test]
    fn matching_layers_interpolated() {
        let source_layer = Layer {
            id: "shared-id".to_string(),
            type_key: "rotate".to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params: {
                let mut m = std::collections::HashMap::new();
                m.insert("theta".to_string(), Param::new(0.0, 0.0, 6.28, false));
                m
            },
        };
        let target_layer = Layer {
            id: "shared-id".to_string(),
            type_key: "rotate".to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params: {
                let mut m = std::collections::HashMap::new();
                m.insert("theta".to_string(), Param::new(3.14, 0.0, 6.28, false));
                m
            },
        };

        let result = blend_layers(&[source_layer], &[target_layer], 0.5);
        assert_eq!(result.len(), 1);
        assert!((result[0].params["theta"].value - 1.57).abs() < 1e-10);
    }

    #[test]
    fn unmatched_source_fades_out() {
        let source_layer = Layer {
            id: "source-only".to_string(),
            type_key: "rotate".to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params: std::collections::HashMap::new(),
        };

        let result = blend_layers(&[source_layer], &[], 0.5);
        assert_eq!(result.len(), 1);
        assert!((result[0].blend.value - 0.5).abs() < 1e-10);
    }

    #[test]
    fn unmatched_target_fades_in() {
        let target_layer = Layer {
            id: "target-only".to_string(),
            type_key: "rotate".to_string(),
            enabled: true,
            blend: Param::new(1.0, 0.0, 1.0, false),
            blend_time_override: None,
            params: std::collections::HashMap::new(),
        };

        let result = blend_layers(&[], &[target_layer], 0.5);
        assert_eq!(result.len(), 1);
        assert!((result[0].blend.value - 0.5).abs() < 1e-10);
    }
}
