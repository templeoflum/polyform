# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Polyform is a **lattice synthesizer** -- a real-time video synthesis instrument that performs mathematical transformations on a 2D grid. It belongs to the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits). The 2D grid is the identity-defining anchor; all extensions must honor the substrate.

The project's core purpose is making higher-dimensional mathematical thinking *felt and performable*, not just studied. The dimension readout, visible grid, and named mathematical operations are pedagogical features, not implementation details.

## File Inventory

### Rust Crates (active development)

| File | Purpose |
|------|---------|
| `crates/polyform-core/src/types.rs` | Core types: Param, ToggleParam, ShapeFunction, PhaseModulator, Blend |
| `crates/polyform-core/src/layer.rs` | Pipeline layer type |
| `crates/polyform-core/src/preset.rs` | Preset, Camera, Quaternion, Sampling, Display, defaults |
| `crates/polyform-core/src/palette.rs` | Palette struct + static data |
| `crates/polyform-core/src/ops.rs` | 25 operation functions (R2 -> R2) with uniform signature |
| `crates/polyform-core/src/lift.rs` | 15 lift functions (R3 displacement) with uniform signature |
| `crates/polyform-core/src/registry.rs` | OperationDef/LiftDef with fn pointers, labels, categories |
| `crates/polyform-core/src/pipeline.rs` | Pipeline application, pre-flatten, numerical Jacobian |
| `crates/polyform-core/src/animation.rs` | resolve_param(), apply_shape(), ParamPath, PipelineKind |
| `crates/polyform-core/src/transition.rs` | TransitionManager with cosine-eased blends, TransitionTarget/Action |
| `crates/polyform-core/src/engine.rs` | AnimationEngine: tick(), weight system, quaternion resolve, build_frame_state |
| `crates/polyform-core/src/frame_state.rs` | FrameState struct (engine output consumed by renderer) |
| `crates/polyform-core/src/notation.rs` | Composition string, formula symbols, DOF counting |
| `crates/polyform-core/src/preset_io.rs` | Tolerant import/export, registry reconciliation, make_layer |
| `crates/polyform-core/src/tests.rs` | Integration tests (golden vectors, round-trip, schema compliance) |
| `crates/polyform-core/tests/fixtures/golden_vectors.json` | JS-generated expected outputs for cross-language validation |
| `crates/polyform-core/tests/fixtures/js-export.json` | JS preset fixture for deserialization tests |
| `crates/polyform-core/tests/fixtures/reconciliation-test.json` | Fixture for testing registry reconciliation |
| `crates/polyform-app/src/main.rs` | App loop, action dispatch, overlay rendering |
| `crates/polyform-app/src/gpu.rs` | wgpu setup, compute/render pipelines, buffer management |
| `crates/polyform-app/src/camera.rs` | Camera matrix computation (orbit, projection, quaternion) |
| `crates/polyform-app/src/ui.rs` | egui control panels, param/toggle widgets, UiAction enum |
| `crates/polyform-app/src/shaders/compute.wgsl` | Grid generation + pipeline compute shader |
| `crates/polyform-app/src/shaders/render.wgsl` | Vertex/fragment shaders with Brown-Conrady distortion |

### Web Prototype (parked)

| File | Purpose |
|------|---------|
| `web/polydimensional-coupling-v8.jsx` | Last web component (v8 -- extracted registries) |
| `web/main.jsx` | Web entry point |
| `web/registries.js` | Transform, displacement, palette, and ratio-group registries |
| `web/preset-schema.ts` | Preset format types + validators (schema v1) |
| `web/preset-io.ts` | State <-> preset bridge |

### Project Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace config |
| `DEVLOG.md` | Development history (v5 through Phase 2.6) |
| `ROADMAP.md` | Current priorities |
| `README.md` | Public-facing documentation |
| `polyform-brief.md` | Project philosophy |
| `archive/` | Legacy versions (v5, v6, v7) and old docs |
| `tests/generate_golden_vectors.mjs` | Node script to regenerate golden vectors from JS |

## Build & Run

```bash
cargo run -p polyform-app          # Run the native app
cargo test                         # Run all tests (141 across both crates)
cargo test -p polyform-core        # Core library tests only
cargo test -p polyform-app         # App tests only (camera)
```

The web prototype can still be run with `npm install && npx vite` but is parked.

## Architecture: Three-Stage Pipeline

**Stage 1 -- Operations** (R2 -> R2): Ordered composition of 2D maps on the grid. Families: linear, complex (z^n, mobius, exp, log), polar, wave, non-holomorphic. Composition is non-commutative.

**Stage 2 -- Lift** (R3 displacement): Sequential 3D displacement applied to the post-operation lattice. Families: height (magnitude, Jacobian, real/imag), wave (sine, gaussian), full-3D (spherical, conical, helical, ripple).

**Stage 3 -- Camera**: Orbit position, lens (focal length, sensor, anamorphic, shift), radial distortion (Brown-Conrady k1/k2), quaternion tumble. The camera is a performable layer, not a passive viewer.

### Param Shape

Every animatable scalar uses the `Param` struct: `{ value, min, max, anim, shape, ratio, offset, modulator }`. Shape is `ShapeFunction::Cosine` (default, smooth ping-pong) or `ShapeFunction::Sawtooth` (linear ramp). `ToggleParam` extends it with an `on` boolean. Every layer has a `blend: Param` for smooth enable/disable transitions.

Optional `PhaseModulator` per param: `{ ratio, depth, shape }` -- FM synthesis on the animation phase.

### Animation: Parameter Motion Protocol

Single master phase `M` advances by `dt * tempo` each tick. Per-param phase: `phi = (M * ratio + offset) mod 1.0`. Shape function maps phi to [0,1], scaled to [min, max]. Anim on/off is a smooth weight transition (base value <-> animated value). No accumulators, no state to sync.

Key engine types:
- `AnimationEngine` -- owns master phase, weight system, transition manager
- `ParamPath` -- identifies any param in the system (layer param, view param, blend, anim weight)
- `FrameState` -- flat struct of all resolved values, produced by `tick()`, consumed by renderer

### Quaternion System

Quaternion uses full `Param` controls for angle and axes (ax, ay, az). Fixed mode: angle + axes resolved via `resolve_param()`. Drift mode: separate accumulator `drift_phase += dt * drift_speed`. On/off: effective angle = resolved_angle * quat_blend (weight transition). Mode transitions: NLERP between stored previous quat and new target.

### Rendering Pipeline

wgpu compute shader generates the grid and applies operations + lift. Vertex shader applies camera (view-projection matrix). Fragment shader applies Brown-Conrady distortion (k1/k2). Buffer pool pre-allocated at MAX_VERTICES, reused every frame.

### UI Architecture

egui control panels emit `UiAction` variants. `main.rs` dispatches actions to modify preset and engine state. `param_widget` and `toggle_param_widget` show animated slider positions via `frame_state_view_value` lookups. Slider grab emits `ParamGrabStart`/`ParamGrabEnd` to pause animation during direct manipulation.

## Vocabulary (Load-Bearing)

These renames are decided and must be applied in all new code and UI strings:

| Old term | New term |
|---|---|
| transform pipeline | **operations** |
| displacement pipeline | **lift** |
| view layers | camera sections |
| input scale | **lattice scale** |
| grid density | **lattice density** |
| composition (formula) | **signal flow** |

`tempo`, `ratio`, `loop`/`sweep`, `palette`, `DOF`, `dim count` stay as-is.

## Design Constraints (Do Not Relitigate)

- **2D grid is the anchor.** Everything is tethered to a visible, regular 2D grid. No untethered fluid sims or pure generative work.
- **Per-layer blend.** Each layer has a `blend` param (0=identity, 1=fully active). Toggling animates blend over a configurable time.
- **Smooth transitions: two paths.** Direct manipulation (slider drag) sets value immediately. Indirect changes (preset load, layer toggle, MIDI) animate toward a target with cosine easing.
- **Reorder via fade.** Animate blend->0, swap position, animate blend->1. Don't interpolate composition order.
- **Tempo lives in the preset.** Crossfading presets also crossfades tempo.
- **Dimension readout is always visible and accurate.** Counts structural + temporal DOFs. This is the philosophical core made visible.
- **Layer identity.** Every layer has a stable UUID. Persists across save/load/reorder.

## Known Gotchas

- **NaN guard on ratios.** `safe_ratio()` in animation.rs handles zero/infinity. Always go through it.
- **Camera pole degeneracy.** Up-vector falls back to world Z when looking straight up/down.
- **Quaternion drift/fixed mode switch.** Can produce a small rotational jump via NLERP transition. Known issue, infrequent operation.
- **Preset format is v2 (parameter motion protocol).** Old-format presets with `animMode`, `phaseOffset`, or the old quaternion `angle`/`angleRatio`/`axes` structure will not load correctly. No backwards compatibility code exists -- start fresh.
- **Two copies of test fixtures.** The active fixtures are at `crates/polyform-core/tests/fixtures/`. The root-level `tests/fixtures/` directory is empty (stale copies deleted).

## Display Overlays

Each overlay occupies a fixed canvas position regardless of which others are active:
- **Signal flow**: bottom-left (keyboard: S)
- **Active layers**: top-left (keyboard: A)
- **Dim count**: top-right (keyboard: D)
- **FPS**: bottom-right (keyboard: F)

## Project Trajectory

**Phase 1** (parked): Web prototype (React + three.js) for design validation.
**Phase 2** (active): Native Rust app (wgpu + egui). Phases 2.1-2.6 complete, 2.7 (Spout/MIDI) next.
**Phase 3** (speculative): C++ rewrite with native Spout SDK integration.

The **preset format is the portable asset** across all three phases.

## When to Ask the User

Ask before deciding anything that touches:
- Dimension readout prominence or accuracy
- Smoothness of transitions in performance use
- UI vocabulary drifting back toward technical/engineering language
- Removing the 2D grid as the anchor

Use your own judgment for: code organization, easing functions, UI spacing/color, test coverage.
