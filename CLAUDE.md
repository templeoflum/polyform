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
| `crates/polyform-core/src/preset.rs` | Preset, Camera, Quaternion, Sampling, Phosphor, defaults |
| `crates/polyform-core/src/palette.rs` | Palette struct + static data |
| `crates/polyform-core/src/ops.rs` | 32 operation functions (R2 -> R2) with uniform signature |
| `crates/polyform-core/src/lift.rs` | 18 lift functions (R3 displacement) with uniform signature |
| `crates/polyform-core/src/registry.rs` | OperationDef/LiftDef with fn pointers, labels, categories |
| `crates/polyform-core/src/pipeline.rs` | Pipeline application, pre-flatten, numerical Jacobian |
| `crates/polyform-core/src/animation.rs` | resolve_param(), apply_shape(), ParamPath, PipelineKind |
| `crates/polyform-core/src/transition.rs` | TransitionManager with cosine-eased blends, TransitionTarget/Action |
| `crates/polyform-core/src/engine.rs` | AnimationEngine: tick(), weight system, quaternion resolve, build_frame_state |
| `crates/polyform-core/src/frame_state.rs` | FrameState struct (engine output consumed by renderer) |
| `crates/polyform-core/src/notation.rs` | Composition string, formula symbols, DOF counting |
| `crates/polyform-core/src/morph.rs` | MorphState, blend_presets, cosine-eased preset morphing |
| `crates/polyform-core/src/preset_io.rs` | Tolerant import/export, registry reconciliation, make_layer |
| `crates/polyform-core/src/factory_presets.rs` | 15 built-in full presets as embedded JSON |
| `crates/polyform-core/src/factory_section_presets.rs` | 50 built-in section presets (operations, lift, position, lens, distortion, quaternion, phosphor, sampling) |
| `crates/polyform-core/src/tests.rs` | Integration tests (golden vectors, round-trip, schema compliance) |
| `crates/polyform-core/tests/fixtures/golden_vectors.json` | JS-generated expected outputs for cross-language validation |
| `crates/polyform-core/tests/fixtures/js-export.json` | JS preset fixture for deserialization tests |
| `crates/polyform-core/tests/fixtures/reconciliation-test.json` | Fixture for testing registry reconciliation |
| `crates/polyform-app/src/main.rs` | App loop, action dispatch, overlay rendering |
| `crates/polyform-app/src/gpu.rs` | wgpu setup, compute/render/blit pipelines, offscreen render target, buffer management |
| `crates/polyform-app/src/camera.rs` | Camera matrix computation (orbit, projection, quaternion) |
| `crates/polyform-app/src/ui.rs` | egui control panels, compact widgets, popup system, UiAction enum |
| `crates/polyform-app/src/preset_library.rs` | Preset library directory, section presets, factory preset installation |
| `crates/polyform-app/src/shaders/compute.wgsl` | Grid generation + pipeline compute shader |
| `crates/polyform-app/src/shaders/render.wgsl` | Vertex/fragment shaders with Brown-Conrady distortion |
| `crates/polyform-app/src/shaders/blit.wgsl` | Fullscreen triangle + texture sampling for offscreen-to-surface blit |
| `crates/polyform-app/src/spout.rs` | Spout texture sharing sender (Windows-only, SpoutDX FFI bridge) |

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
| `DEVLOG.md` | Development history (v5 through Phase 2.13) |
| `ROADMAP.md` | Current priorities |
| `README.md` | Public-facing documentation |
| `FEATURES.md` | Feature priorities and shipped status |
| `polyform-brief.md` | Project philosophy |
| `archive/` | Legacy versions (v5, v6, v7) and old docs |
| `tests/generate_golden_vectors.mjs` | Node script to regenerate golden vectors from JS |

## Build & Run

```bash
cargo run -p polyform-app          # Run the native app
cargo test                         # Run all tests (174 across both crates)
cargo test -p polyform-core        # Core library tests only (170)
cargo test -p polyform-app         # App tests only (4, camera)
```

The web prototype can still be run with `npm install && npx vite` but is parked.

## Architecture: Three-Stage Pipeline

**Stage 1 -- Operations** (R2 -> R2): Ordered composition of 2D maps on the grid (32 operations). Families: linear, complex (z^n, mobius, exp, log, joukowski, zTan), polar (kaleidoscope), wave (noise), non-holomorphic, utility (wrap, quantize, clamp). Each operation supports decoupled `cx, cy` center params for movable poles/branch points. Composition is non-commutative.

**Stage 2 -- Lift** (R3 displacement): Sequential 3D displacement applied to the post-operation lattice (18 lifts). Families: height (magnitude, Jacobian, real/imag, saddle, terrace), wave (sine, gaussian), full-3D (spherical, conical, helical, ripple, toroidal).

**Stage 3 -- Camera**: Orbit position, lens (focal length, sensor, anamorphic, shift), radial distortion (full Brown-Conrady: k1-k4, p1-p2, decoupled center, anamorphic squeeze), quaternion tumble. The camera is a performable layer, not a passive viewer.

**Phosphor System**: Post-processing rendering character -- beam quads with configurable width/intensity, HDR persistence buffer, bloom (Gaussian with threshold), gain/saturation/gamma color grading. All params are ToggleParams (animatable).

### Lattice Types

Five substrate types, selectable per-preset:
- **Cartesian** -- regular rectangular grid (default)
- **Polar** -- concentric rings + radial spokes
- **Hexagonal** -- three line families at 0/60/120 degrees, hexagonal boundary
- **Log-polar** -- logarithmic spacing on radial spokes
- **Bipolar** -- sigma/tau curves with dual foci

Smooth vertex morphing between types (cosine-eased, 0.4s). Lattice type is structural (not animatable).

### Param Shape

Every animatable scalar uses the `Param` struct: `{ value, min, max, anim, shape, ratio, offset, modulator }`. Shape is `ShapeFunction::Cosine` (default, smooth ping-pong) or `ShapeFunction::Sawtooth` (linear ramp). `ToggleParam` extends it with an `on` boolean. Every layer has a `blend: Param` for smooth enable/disable transitions.

Optional `PhaseModulator` per param: `{ ratio, depth, shape }` -- FM synthesis on the animation phase.

### Animation: Parameter Motion Protocol

Single master phase `M` advances by `dt * tempo` each tick. Per-param phase: `phi = (M * ratio + offset) mod 1.0`. Shape function maps phi to [0,1], scaled to [min, max]. Anim on/off is a smooth weight transition (base value <-> animated value). No accumulators, no state to sync.

**BPM timing**: Tempo stored as Hz internally. BPM display uses bar-rate: `Hz = BPM / 240`. This makes ratio=1 mean one oscillation per bar (4 beats in 4/4). At 120 BPM, ratio=1 gives a 2-second cycle.

**Two blend times**: Individual param toggles, layer toggles, and mode switches use `TOGGLE_BLEND_TIME` (0.3s, fixed). Preset morphing uses `blend_time` (default 3.0s, adjustable via UI).

Key engine types:
- `AnimationEngine` -- owns master phase, weight system, transition manager
- `ParamPath` -- identifies any param in the system (layer param, view param, blend, anim weight)
- `FrameState` -- flat struct of all resolved values, produced by `tick()`, consumed by renderer

### Quaternion System

Quaternion uses full `Param` controls for angle and axes (ax, ay, az). Fixed mode: angle + axes resolved via `resolve_param()`. Drift mode: separate accumulator `drift_phase += dt * drift_speed`. On/off: effective angle = resolved_angle * quat_blend (weight transition). Mode transitions: NLERP between stored previous quat and new target.

### Morph System

`MorphState` in `morph.rs` handles cosine-eased crossfade between presets. `blend_presets()` interpolates all Param/ToggleParam fields. Matching layers (by UUID) interpolate; unmatched source layers fade out (blend → 0), unmatched target layers fade in (0 → blend). Animation is disabled (`anim: false`) during the morph for params that differ between source and target — unchanged params preserve their animation state, so section morphs don't disrupt other sections. When the morph completes, the target preset is assigned directly and animation resumes via the weight system. Discrete fields (enabled, lattice type, quaternion mode) snap at the midpoint. Both full and section preset loads use the morph path.

### Preset Library

Directory-based at `~/.polyform/presets/` with subdirectories: `full/`, `operations/`, `lift/`, `position/`, `lens/`, `distortion/`, `quaternion/`, `phosphor/`, `sampling/`. 15 factory full presets + 50 factory section presets installed on startup. Section presets save/load individual sections via header buttons with inline ComboBox dropdowns.

### Rendering Pipeline

Render resolution is a **viewport** property (default 1920x1080), independent of window size. The lattice renders to an offscreen texture at viewport resolution, the camera derives its aspect ratio from the viewport, and the window displays a scaled letterboxed preview via a blit pass. Spout sends the offscreen texture at native viewport resolution -- pixel-perfect, no upscaling.

wgpu compute shader generates the grid and applies operations + lift. Vertex shader applies camera (view-projection matrix). Fragment shader applies full Brown-Conrady distortion (k1-k4, p1-p2, center offset, squeeze). Buffer pool pre-allocated at MAX_VERTICES, reused every frame. Blit shader uses a fullscreen triangle to sample the offscreen texture; letterboxing is achieved by setting the viewport rect on the blit render pass.

### UI Architecture

**Performance UI** -- compact, information-dense layout designed for live use:

- **Top strip**: Tempo (BPM drag + log slider + popup presets), preset name/save/load (inline dropdown library), morph duration, palette selector, viewport size, Spout toggle, overlay checkboxes (signal/dim/layers/FPS + visualization: caustic/domain/curl/conformal/normals)
- **Responsive columns** (3-5 depending on window width): Operations, Lift, Lattice+Phosphor, Camera (splits into Position+Lens / Distortion+Quaternion at 5 cols)
- **`compact_param_widget`**: Single-line slider with anim checkbox + ratio badge + `...` popup button for detail (shape, ratio picker, min/max, offset, FM modulator)
- **`compact_toggle_param_widget`**: Same as above with prepended on/off checkbox
- **`section_with_toggle`**: Collapsible section with group-mute checkbox on the header line
- **Popup system**: `OpenPopups` stored in egui type-map, toggleable windows for param detail and BPM/ratio presets
- **`UiAction` enum**: All mutations flow as actions dispatched by `main.rs`
- **`ParamGrabStart`/`ParamGrabEnd`**: Pause animation during slider drag
- **`frame_state_view_value`**: Maps view keys to FrameState fields for swept value display

### Visualization Overlays

In addition to the standard display overlays (signal flow, dim count, active layers, FPS), the app supports GPU-computed visualization overlays:
- **Caustic** -- `det J = 0` fold/cusp curves
- **Domain coloring** -- phase/magnitude color mapping of the complex operation
- **Curl** -- curl magnitude of the 2D vector field
- **Conformal defect** -- deviation from conformal (angle-preserving) mapping
- **Normal shading** -- surface normal visualization for lifted geometry

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
- **Tempo is ephemeral session state.** Not saved in presets. Controlled by the performer, not the patch.
- **Dimension readout is always visible and accurate.** Counts structural + temporal DOFs. This is the philosophical core made visible.
- **Layer identity.** Every layer has a stable UUID. Persists across save/load/reorder.

## Known Gotchas

- **NaN guard on ratios.** `safe_ratio()` in animation.rs handles zero/infinity. Always go through it.
- **Camera pole degeneracy.** Up-vector falls back to world Z when looking straight up/down.
- **Quaternion drift/fixed mode switch.** Can produce a small rotational jump via NLERP transition. Known issue, infrequent operation.
- **Joukowski origin fade.** joukowski(z + a/z) uses a smooth fade below r=0.3 to prevent singularity spikes at origin. Both Rust and WGSL must match.
- **BPM is bar-rate.** `Hz = BPM / 240`, not `BPM / 60`. Ratio=1 = one oscillation per bar (4 beats). Don't use `/60` anywhere.
- **Two blend times.** `TOGGLE_BLEND_TIME` (0.3s) for individual toggles, `self.blend_time` (configurable) for preset morphs. Don't use `self.blend_time` for toggles.
- **Morph disables animation.** `lerp_param` sets `anim: false` during morphs. The animation engine must not resolve animated values from blended presets mid-morph.
- **No backwards compatibility.** Old-format presets (animMode, phaseOffset, old quaternion structure) will not load. No old presets to worry about -- start fresh.
- **Two copies of test fixtures.** The active fixtures are at `crates/polyform-core/tests/fixtures/`. The root-level `tests/fixtures/` directory is empty (stale copies deleted).

## Display Overlays

Each overlay occupies a fixed canvas position regardless of which others are active:
- **Signal flow**: bottom-left (keyboard: S)
- **Active layers**: top-left (keyboard: A)
- **Dim count**: top-right (keyboard: D)
- **FPS**: bottom-right (keyboard: F)

## Project Trajectory

**Phase 1** (parked): Web prototype (React + three.js) for design validation.
**Phase 2** (active): Native Rust app (wgpu + egui). Phases 2.1-2.14 complete (core engine, rendering, UI, Spout, viewport, lattice family, compositional depth, phosphor, visualization overlays, Performance UI, preset system with morphing, expanded registries, factory presets, BPM timing, split camera section presets). Next: MIDI mapping, recording.
**Phase 3** (speculative): C++ rewrite with native Spout SDK integration.

The **preset format is the portable asset** across all three phases.

## When to Ask the User

Ask before deciding anything that touches:
- Dimension readout prominence or accuracy
- Smoothness of transitions in performance use
- UI vocabulary drifting back toward technical/engineering language
- Removing the 2D grid as the anchor

Use your own judgment for: code organization, easing functions, UI spacing/color, test coverage.
