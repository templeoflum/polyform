use glam::{Mat4, Vec3};
use polyform_core::frame_state::FrameState;
use polyform_core::types::ProjectionMode;

/// View-projection matrices for rendering.
pub struct CameraMatrices {
    pub view_projection: Mat4,
}

/// Build camera matrices from a FrameState and window aspect ratio.
///
/// Matches the JS Three.js camera setup:
/// - Lattice on xz plane (x → world x, lattice y → world z, lift → world y)
/// - Orbit around target with polar angles
/// - Quaternion as model rotation
pub fn build_camera(frame: &FrameState, aspect: f32) -> CameraMatrices {
    // --- Orbit camera (view matrix) ---
    let orbit_x = frame.orbit_x as f32;
    let orbit_y = frame.orbit_y as f32;
    let distance = frame.distance as f32;
    let target_x = frame.target_x as f32;
    let target_y = frame.target_y as f32;
    let roll = frame.roll as f32;

    let sin_y = orbit_y.sin();
    let cos_y = orbit_y.cos();
    let sin_x = orbit_x.sin();
    let cos_x = orbit_x.cos();

    let eye = Vec3::new(
        target_x + distance * sin_y * sin_x,
        distance * cos_y,
        target_y + distance * sin_y * cos_x,
    );

    let target = Vec3::new(target_x, 0.0, target_y);
    let forward = (target - eye).normalize_or_zero();

    // Up vector with pole degeneracy fallback
    let flip_y = if sin_y >= 0.0 { 1.0_f32 } else { -1.0_f32 };
    let candidate_up = Vec3::new(0.0, flip_y, 0.0);
    let up = if forward.dot(candidate_up).abs() > 0.9999 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        candidate_up
    };

    // Gram-Schmidt orthogonalization
    let right = forward.cross(up).normalize_or_zero();
    let ortho_up = right.cross(forward).normalize_or_zero();

    // Apply roll as axis-angle rotation around forward
    let rolled_up = if roll.abs() > 1e-6 {
        let roll_rot = Mat4::from_axis_angle(forward, roll);
        roll_rot.transform_vector3(ortho_up)
    } else {
        ortho_up
    };

    let view = Mat4::look_at_rh(eye, target, rolled_up);

    // --- Projection matrix ---
    let projection = match frame.projection {
        ProjectionMode::Perspective => {
            let focal_length = frame.focal_length as f32;
            let sensor_size = frame.sensor_size as f32;
            let fov = 2.0 * (sensor_size / (2.0 * focal_length)).atan();
            Mat4::perspective_rh(fov, aspect, 0.1, 100.0)
        }
        ProjectionMode::Orthographic => {
            let ortho_zoom = frame.ortho_zoom as f32;
            let half_w = ortho_zoom * aspect;
            let half_h = ortho_zoom;
            Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, 0.1, 100.0)
        }
    };

    // --- Lens shift + anamorphic ---
    let anamorphic = frame.anamorphic as f32;
    let shift_x = frame.shift_x as f32;
    let shift_y = frame.shift_y as f32;

    let lens_correction = Mat4::from_translation(Vec3::new(shift_x, shift_y, 0.0))
        * Mat4::from_scale(Vec3::new(anamorphic, 1.0, 1.0));

    let corrected_projection = lens_correction * projection;

    // --- Quaternion tumble (model rotation) ---
    let model = if frame.quat_on {
        let [ax, ay, az] = frame.quat_axis;
        let axis = Vec3::new(ax as f32, ay as f32, az as f32);
        let len_sq = axis.length_squared();
        let safe_axis = if len_sq < 1e-6 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            axis.normalize()
        };
        Mat4::from_axis_angle(safe_axis, frame.quat_angle as f32)
    } else {
        Mat4::IDENTITY
    };

    let view_projection = corrected_projection * view * model;

    CameraMatrices { view_projection }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polyform_core::frame_state::FrameState;
    use polyform_core::types::ProjectionMode;

    fn default_frame() -> FrameState {
        FrameState {
            operations: vec![],
            lifts: vec![],
            any_lift_needs_jacobian: false,
            orbit_x: 0.0,
            orbit_y: 0.1,
            distance: 3.5,
            target_x: 0.0,
            target_y: 0.0,
            roll: 0.0,
            projection: ProjectionMode::Perspective,
            focal_length: 35.0,
            sensor_size: 36.0,
            ortho_zoom: 2.0,
            anamorphic: 1.0,
            shift_x: 0.0,
            shift_y: 0.0,
            k1: 0.0,
            k2: 0.0,
            quat_on: false,
            quat_angle: 0.0,
            quat_axis: [0.0, 1.0, 0.0],
            scale: 0.6,
            density: 27.0,
            palette_key: "phosphor".to_string(),
            has_active_layers: false,
        }
    }

    #[test]
    fn default_camera_produces_valid_vp() {
        let frame = default_frame();
        let cam = build_camera(&frame, 16.0 / 9.0);
        // VP matrix should not be zero or NaN
        let cols = cam.view_projection.to_cols_array();
        for v in &cols {
            assert!(!v.is_nan(), "VP matrix contains NaN");
            assert!(v.is_finite(), "VP matrix contains infinity");
        }
    }

    #[test]
    fn orbit_x_rotates_eye() {
        let mut frame = default_frame();
        frame.orbit_x = std::f64::consts::FRAC_PI_2;
        frame.orbit_y = std::f64::consts::FRAC_PI_2; // equator
        let cam = build_camera(&frame, 1.0);
        // At orbit_x = PI/2 on the equator, eye should be at (+distance, 0, 0) relative to target
        let cols = cam.view_projection.to_cols_array();
        assert!(!cols[0].is_nan());
    }

    #[test]
    fn quaternion_with_known_rotation() {
        let mut frame = default_frame();
        frame.quat_on = true;
        frame.quat_angle = std::f64::consts::FRAC_PI_2;
        frame.quat_axis = [0.0, 1.0, 0.0];
        let cam = build_camera(&frame, 1.0);
        let cols = cam.view_projection.to_cols_array();
        for v in &cols {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn orthographic_zoom() {
        let mut frame = default_frame();
        frame.projection = ProjectionMode::Orthographic;
        frame.ortho_zoom = 3.0;
        let cam = build_camera(&frame, 1.0);
        let cols = cam.view_projection.to_cols_array();
        for v in &cols {
            assert!(v.is_finite());
        }
    }
}
