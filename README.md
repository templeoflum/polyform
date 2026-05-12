# Polyform

A lattice synthesizer for real-time video synthesis. Polyform performs mathematical transformations on a 2D grid substrate, producing animated geometric imagery in the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits).

The instrument makes higher-dimensional mathematical thinking felt and performable. The visible grid, named operations, and live dimension readout are pedagogical features -- you develop geometric intuition through play, not study.

## Quick Start

```bash
cargo run -p polyform-app
```

The web prototype (React + three.js) is parked in `web/` and can still be run with `npm install && npx vite` for reference.

## What You See

A regular 2D grid, transformed in real time by a composable pipeline of mathematical operations. The grid is always visible -- it's the anchor that makes the mathematics legible.

**Operations** (R2 -> R2) warp the grid: z^n, mobius, exp/log, spiral, shear, wave. Composition is non-commutative -- order matters.

**Lift** (R3 displacement) pulls the grid into three dimensions: sine waves, Jacobian height, spherical projection, helical winding.

**Camera** frames the result: orbit, focal length, anamorphic squeeze, lens distortion, quaternion tumble. The camera is a performable layer, not a passive viewer.

Everything animates against a global tempo with polyrhythmic ratios. The dimension readout shows the live count of structural and temporal degrees of freedom in the active configuration.

## Controls

- **Drag** the canvas to orbit the camera
- **Operations/Lift sections** -- add, remove, reorder, toggle layers. Each layer has per-parameter animation controls (shape, ratio, offset, FM modulator) and a blend slider for smooth transitions.
- **Camera sections** -- position, lens, distortion, quaternion tumble. Each parameter is independently animatable.
- **Keyboard overlays** -- S (signal flow), A (active layers), D (dim count), F (FPS)
- **Presets** -- save/load complete instrument state as JSON via native file dialogs.

## Display Overlays

Toggle independently via keyboard:
- **Signal flow** (bottom-left) -- live formula showing the composition chain
- **Active layers** (top-left) -- what's on, structural/temporal DOF per layer
- **Dim count** (top-right) -- total structural + temporal dimensions
- **FPS** (bottom-right) -- frame counter

## Architecture

Cargo workspace with two crates:

- **`polyform-core`** -- library crate containing all math, types, registries, pipeline, animation engine, preset I/O. No rendering dependencies.
- **`polyform-app`** -- binary crate with wgpu rendering, egui UI, and the application loop.

### Pipeline

```
2D lattice -> Operations (R2->R2) -> Lift (R3 displacement) -> Camera -> Screen
```

### Animation: Parameter Motion Protocol

A single master phase advances continuously, driven by tempo. Each animated param computes its value as a pure function of the master phase, its ratio, offset, and shape function (cosine or sawtooth). No accumulators, no state to get out of sync. Animation on/off is a smooth weight transition blending between base and animated values.

Optional FM modulation per param: a secondary oscillator shifts the carrier phase by a configurable depth and ratio.

### Param Model

Every animatable scalar: `{ value, min, max, anim, shape, ratio, offset, modulator }`. Toggle params add `on`. Layers add `blend`. Shape functions: `cosine` (smooth ping-pong, default) and `sawtooth` (linear ramp).

### Key Systems

- **Compute shader pipeline** -- grid generation and pipeline math on GPU via wgpu compute pass
- **Buffer pool** -- pre-allocated at MAX_VERTICES, reused every frame, no per-frame allocation
- **Weight-based transitions** -- anim on/off, layer toggle, mode switch all use smooth weight interpolation
- **Quaternion tumble** -- full Param controls for angle + axes, SLERP blending, NLERP mode transitions
- **Tolerant preset I/O** -- unknown fields ignored, missing fields defaulted, type mismatches warned

## Preset Format

JSON schema v1. Contains complete instrument state: pipeline structure, all parameter values/ranges/animation settings, camera, palette, tempo, display toggles. Layer IDs are stable UUIDs for future morphing support. See `crates/polyform-core/src/preset.rs` for types.

## Project Trajectory

| Phase | Stack | Status |
|-------|-------|--------|
| 1 -- Web prototype | React + three.js | Parked (design validation complete) |
| 2 -- Native app | Rust + wgpu + egui | **Active** (2.1-2.6 complete, 2.7 next) |
| 3 -- Production | C++ + Spout SDK | Speculative |

The preset format is the portable asset across all phases. Next: Spout bridge for Resolume, MIDI mapping.

## File Structure

```
crates/
  polyform-core/src/       Core library (no rendering deps)
    types.rs               Param, ToggleParam, ShapeFunction, PhaseModulator
    preset.rs              Preset, Camera, Quaternion, Sampling, Display
    layer.rs               Pipeline layer type
    ops.rs                 25 operation functions (R2 -> R2)
    lift.rs                15 lift functions (R3 displacement)
    registry.rs            OperationDef/LiftDef with fn pointers, labels, categories
    pipeline.rs            Pipeline application, pre-flatten, numerical Jacobian
    animation.rs           resolve_param(), apply_shape(), ParamPath
    engine.rs              AnimationEngine: tick(), weight system, quaternion resolve
    transition.rs          TransitionManager with cosine-eased blends
    frame_state.rs         FrameState (engine output consumed by renderer)
    notation.rs            Composition string, formula symbols, DOF counting
    preset_io.rs           Tolerant import/export, registry reconciliation
    palette.rs             Palette struct + static data
  polyform-core/tests/     Test fixtures (golden vectors, reconciliation)
  polyform-app/src/        Application binary
    main.rs                App loop, action handlers, overlay rendering
    gpu.rs                 wgpu setup, compute/render pipelines, buffer management
    camera.rs              Camera matrix computation (orbit, projection, distortion)
    ui.rs                  egui control panels, param widgets
    shaders/
      compute.wgsl         Grid generation + pipeline compute shader
      render.wgsl          Vertex/fragment shaders with lens distortion
web/                       Parked web prototype (React + three.js)
archive/                   Legacy versions (v5, v6, v7)
```

## License

Not yet determined.
