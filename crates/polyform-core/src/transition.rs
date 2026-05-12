use std::f64::consts::PI;

use crate::animation::PipelineKind;

// =============================================================================
// Types
// =============================================================================

/// What a transition is animating.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransitionTarget {
    LayerBlend {
        kind: PipelineKind,
        layer_id: String,
    },
    ViewParam(String),
    Tempo,
}

/// Action to take when a transition completes.
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionAction {
    DisableLayer {
        kind: PipelineKind,
        layer_id: String,
    },
    /// Restore blend.value in the preset after a fade-in transition completes.
    RestoreBlendValue {
        kind: PipelineKind,
        layer_id: String,
        value: f64,
    },
}

/// A single in-flight transition.
#[derive(Debug, Clone)]
pub struct Transition {
    pub id: u64,
    pub target: TransitionTarget,
    pub from_value: f64,
    pub to_value: f64,
    /// Set on first tick (lazy start).
    pub start_time: Option<f64>,
    pub duration: f64,
    pub on_complete: Option<TransitionAction>,
}

/// Value update produced by a transition tick.
#[derive(Debug, Clone)]
pub struct TransitionUpdate {
    pub target: TransitionTarget,
    pub value: f64,
}

// =============================================================================
// TransitionManager
// =============================================================================

/// Manages in-flight cosine-eased transitions.
///
/// Uses wall-clock time (not anim_clock) so transitions run even when paused.
/// Easing: `0.5 - 0.5 * cos(progress * PI)` — symmetric cosine ease-in-out.
pub struct TransitionManager {
    transitions: Vec<Transition>,
    next_id: u64,
}

impl TransitionManager {
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            next_id: 1,
        }
    }

    /// Start a new transition. Returns a unique ID for cancellation.
    ///
    /// `start_time` sets when the transition begins. Pass `Some(now)` to start
    /// immediately, or `None` for lazy start on first tick.
    pub fn start(
        &mut self,
        target: TransitionTarget,
        from_value: f64,
        to_value: f64,
        duration: f64,
        on_complete: Option<TransitionAction>,
        start_time: Option<f64>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.transitions.push(Transition {
            id,
            target,
            from_value,
            to_value,
            start_time,
            duration,
            on_complete,
        });
        id
    }

    /// Cancel a transition by ID.
    pub fn cancel(&mut self, id: u64) {
        self.transitions.retain(|t| t.id != id);
    }

    /// Cancel all transitions for a given target.
    pub fn cancel_for(&mut self, target: &TransitionTarget) {
        self.transitions.retain(|t| &t.target != target);
    }

    /// Advance all transitions. Returns value updates and completed actions.
    pub fn tick(&mut self, now: f64) -> (Vec<TransitionUpdate>, Vec<TransitionAction>) {
        let mut updates = Vec::new();
        let mut actions = Vec::new();
        let mut completed_indices = Vec::new();

        for (i, t) in self.transitions.iter_mut().enumerate() {
            let start = *t.start_time.get_or_insert(now);
            let elapsed = now - start;
            let progress = if t.duration <= 0.0 {
                1.0
            } else {
                (elapsed / t.duration).min(1.0)
            };
            let eased = 0.5 - 0.5 * (progress * PI).cos();
            let value = t.from_value + (t.to_value - t.from_value) * eased;

            updates.push(TransitionUpdate {
                target: t.target.clone(),
                value,
            });

            if progress >= 1.0 {
                completed_indices.push(i);
                if let Some(action) = &t.on_complete {
                    actions.push(action.clone());
                }
            }
        }

        // Remove completed transitions in reverse order to maintain indices.
        for &i in completed_indices.iter().rev() {
            self.transitions.remove(i);
        }

        (updates, actions)
    }

    /// Number of active (in-flight) transitions.
    pub fn active_count(&self) -> usize {
        self.transitions.len()
    }

    /// Check if there is an active transition for the given target.
    pub fn has_active(&self, target: &TransitionTarget) -> bool {
        self.transitions.iter().any(|t| &t.target == target)
    }
}

impl Default for TransitionManager {
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

    fn assert_near(a: f64, b: f64, eps: f64) {
        assert!(
            (a - b).abs() < eps,
            "expected {b}, got {a} (diff={})",
            (a - b).abs()
        );
    }

    fn blend_target(kind: PipelineKind, id: &str) -> TransitionTarget {
        TransitionTarget::LayerBlend {
            kind,
            layer_id: id.to_string(),
        }
    }

    #[test]
    fn transition_0_to_1_midpoint() {
        let mut tm = TransitionManager::new();
        tm.start(
            blend_target(PipelineKind::Operations, "layer1"),
            0.0,
            1.0,
            1.0,
            None,
            None,
        );

        // First tick at t=0 sets start_time
        let (updates, actions) = tm.tick(0.0);
        assert_eq!(updates.len(), 1);
        assert_near(updates[0].value, 0.0, 1e-10);
        assert!(actions.is_empty());
        assert_eq!(tm.active_count(), 1);

        // Tick at t=0.5 (midpoint): cosine easing at 50% -> 0.5
        let (updates, actions) = tm.tick(0.5);
        assert_eq!(updates.len(), 1);
        assert_near(updates[0].value, 0.5, 1e-10);
        assert!(actions.is_empty());

        // Tick at t=1.0 (complete): value=1.0
        let (updates, actions) = tm.tick(1.0);
        assert_eq!(updates.len(), 1);
        assert_near(updates[0].value, 1.0, 1e-10);
        assert!(actions.is_empty());
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn cancel_removes_transition() {
        let mut tm = TransitionManager::new();
        let id = tm.start(
            blend_target(PipelineKind::Operations, "layer1"),
            0.0,
            1.0,
            1.0,
            None,
            None,
        );
        assert_eq!(tm.active_count(), 1);
        tm.cancel(id);
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn cancel_for_removes_all_matching() {
        let mut tm = TransitionManager::new();
        let target = blend_target(PipelineKind::Operations, "layer1");
        tm.start(target.clone(), 0.0, 1.0, 1.0, None, None);
        tm.start(target.clone(), 0.5, 0.0, 2.0, None, None);
        tm.start(
            blend_target(PipelineKind::Lift, "layer2"),
            0.0,
            1.0,
            1.0,
            None,
            None,
        );
        assert_eq!(tm.active_count(), 3);
        tm.cancel_for(&target);
        assert_eq!(tm.active_count(), 1);
    }

    #[test]
    fn on_complete_action_returned() {
        let mut tm = TransitionManager::new();
        let action = TransitionAction::DisableLayer {
            kind: PipelineKind::Operations,
            layer_id: "layer1".to_string(),
        };
        tm.start(
            blend_target(PipelineKind::Operations, "layer1"),
            1.0,
            0.0,
            0.5,
            Some(action.clone()),
            None,
        );

        // Not complete yet
        let (_, actions) = tm.tick(0.0);
        assert!(actions.is_empty());

        // Complete at t=0.5
        let (updates, actions) = tm.tick(0.5);
        assert_eq!(updates.len(), 1);
        assert_near(updates[0].value, 0.0, 1e-10);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], action);
        assert_eq!(tm.active_count(), 0);
    }

    #[test]
    fn multiple_concurrent_transitions() {
        let mut tm = TransitionManager::new();
        tm.start(
            blend_target(PipelineKind::Operations, "a"),
            0.0,
            1.0,
            1.0,
            None,
            None,
        );
        tm.start(
            blend_target(PipelineKind::Lift, "b"),
            1.0,
            0.0,
            2.0,
            None,
            None,
        );

        // Tick at t=0
        let (updates, _) = tm.tick(0.0);
        assert_eq!(updates.len(), 2);

        // Tick at t=1.0: first completes, second at midpoint
        let (updates, _) = tm.tick(1.0);
        assert_eq!(updates.len(), 2);
        // First (0->1 over 1s): complete
        assert_near(updates[0].value, 1.0, 1e-10);
        // Second (1->0 over 2s): at 50% progress, eased = 0.5, value = 1 + (0-1)*0.5 = 0.5
        assert_near(updates[1].value, 0.5, 1e-10);

        // First completed, only second remains
        assert_eq!(tm.active_count(), 1);
    }

    #[test]
    fn zero_duration_completes_immediately() {
        let mut tm = TransitionManager::new();
        tm.start(
            blend_target(PipelineKind::Operations, "fast"),
            0.0,
            1.0,
            0.0,
            None,
            None,
        );

        let (updates, _) = tm.tick(0.0);
        assert_eq!(updates.len(), 1);
        assert_near(updates[0].value, 1.0, 1e-10);
        assert_eq!(tm.active_count(), 0);
    }
}
