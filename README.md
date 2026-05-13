# Polyform

A lattice synthesizer for real-time video synthesis. Polyform performs mathematical transformations on a 2D grid substrate, producing animated geometric imagery in the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits).

The instrument makes higher-dimensional mathematical thinking felt and performable. The visible grid, named operations, and live dimension readout are pedagogical features -- you develop geometric intuition through play, not study.

## Quick Start

```bash
cargo run -p polyform-app
```

The web prototype (React + three.js) is parked in `web/` and can still be run with `npm install && npx vite` for reference.

## What You See

A 2D lattice, transformed in real time by a composable pipeline of mathematical operations. The grid is always visible -- it's the anchor that makes the mathematics legible. Five lattice types are available: Cartesian, polar, hexagonal, log-polar, and bipolar.

**Operations** (R2 -> R2) warp the grid: z^n, mobius, exp/log, joukowski, spiral, kaleidoscope, shear, wave, noise, wrap, quantize, clamp. 32 operations across linear, complex, polar, wave, non-holomorphic, and utility families. Each operation supports decoupled center params for movable poles/branch points. Composition is non-commutative -- order matters.

**Lift** (R3 displacement) pulls the grid into three dimensions: sine waves, Jacobian height, spherical projection, helical winding, saddle, terrace, toroidal wrap. 18 lift functions across height, wave, and full-3D families.

**Camera** frames the result: orbit, focal length, anamorphic squeeze, full Brown-Conrady lens distortion (k1-k4, p1-p2, decoupled center, squeeze), quaternion tumble. The camera is a performable layer, not a passive viewer.

**Phosphor** adds rendering character: beam quads with configurable width/intensity, HDR persistence for CRT-like afterglow, bloom post-process, gain/saturation/gamma color grading.

Everything animates against a global tempo with polyrhythmic ratios (BPM is bar-rate: ratio=1 means one oscillation per bar at 4/4). The dimension readout shows the live count of structural and temporal degrees of freedom in the active configuration.

**Preset morphing** crossfades between presets with cosine-eased interpolation. Matching layers (by UUID) interpolate all params; unmatched layers fade in/out. Morph duration is adjustable (default 3s). 15 factory presets ship built-in covering every operation family.

## Controls

- **Operations/Lift sections** -- add, remove, reorder, toggle layers. Each layer has a compact slider with anim checkbox. Click `...` for detail (shape, ratio, offset, FM modulator). Blend slider for smooth transitions.
- **Camera sections** -- position, lens, distortion, quaternion tumble. Each parameter is independently animatable with the same compact widget + popup detail pattern.
- **Lattice** -- select substrate type (Cartesian, polar, hex, log-polar, bipolar) with per-type controls.
- **Phosphor** -- persistence, beam width/drive, bloom, gain/saturation/gamma.
- **Top strip** -- tempo (BPM + log slider + presets), preset name/save/load (inline dropdown library), morph duration, palette, viewport size, Spout toggle.
- **Keyboard overlays** -- S (signal flow), A (active layers), D (dim count), F (FPS)

## Display Overlays

Toggle independently via keyboard:
- **Signal flow** (bottom-left) -- live formula showing the composition chain
- **Active layers** (top-left) -- what's on, structural/temporal DOF per layer
- **Dim count** (top-right) -- total structural + temporal dimensions
- **FPS** (bottom-right) -- frame counter

## Visualization Overlays

Toggle via top strip checkboxes:
- **Caustic** -- det J = 0 fold/cusp curves
- **Domain coloring** -- phase/magnitude color mapping of the complex operation
- **Curl** -- curl magnitude of the 2D vector field
- **Conformal defect** -- deviation from angle-preserving mapping
- **Normal shading** -- surface normal visualization for lifted geometry

## Architecture

Cargo workspace with two crates:

- **`polyform-core`** -- library crate containing all math, types, registries, pipeline, animation engine, preset I/O. No rendering dependencies.
- **`polyform-app`** -- binary crate with wgpu rendering, egui UI, and the application loop.

### Pipeline

```
2D lattice -> Operations (R2->R2) -> Lift (R3 displacement) -> Camera -> Phosphor -> Screen
```

### Animation: Parameter Motion Protocol

A single master phase advances continuously, driven by tempo. Each animated param computes its value as a pure function of the master phase, its ratio, offset, and shape function (cosine or sawtooth). No accumulators, no state to get out of sync. Animation on/off is a smooth weight transition blending between base and animated values.

Optional FM modulation per param: a secondary oscillator shifts the carrier phase by a configurable depth and ratio.

### Param Model

Every animatable scalar: `{ value, min, max, anim, shape, ratio, offset, modulator }`. Toggle params add `on`. Layers add `blend`. Shape functions: `cosine` (smooth ping-pong, default) and `sawtooth` (linear ramp).

### Key Systems

- **Compute shader pipeline** -- grid generation and pipeline math on GPU via wgpu compute pass
- **Buffer pool** -- pre-allocated at MAX_VERTICES, reused every frame, no per-frame allocation
- **Weight-based transitions** -- anim on/off, layer toggle, mode switch all use smooth weight interpolation (0.3s)
- **Preset morphing** -- cosine-eased crossfade between presets, animation disabled during morph, configurable duration
- **Quaternion tumble** -- full Param controls for angle + axes, SLERP blending, NLERP mode transitions
- **Phosphor rendering** -- beam quads, HDR persistence buffer, Gaussian bloom, color grading
- **Tolerant preset I/O** -- unknown fields ignored, missing fields defaulted, type mismatches warned
- **Factory presets** -- 15 built-in presets covering every operation family, installed to library on startup
- **Spout texture sharing** -- Windows-only, sends offscreen texture at native viewport resolution

## Preset Format

JSON. Contains the instrument patch: pipeline structure (operations + lift layers), all parameter values/ranges/animation settings, camera, quaternion, sampling, phosphor. Layer IDs are stable UUIDs for morphing support. Ephemeral session state (tempo, palette, viewport, display toggles) is not saved in presets. Section presets (operations, lift, camera, phosphor, sampling) can be saved/loaded independently. See `crates/polyform-core/src/preset.rs` for types.

## Project Trajectory

| Phase | Stack | Status |
|-------|-------|--------|
| 1 -- Web prototype | React + three.js | Parked (design validation complete) |
| 2 -- Native app | Rust + wgpu + egui | **Active** (2.1-2.13 complete: preset system, morphing, expanded registries) |
| 3 -- Production | C++ + Spout SDK | Speculative |

The preset format is the portable asset across all phases. Next: MIDI mapping, recording. C++ rewrite speculative.

## File Structure

```
crates/
  polyform-core/src/       Core library (no rendering deps)
    types.rs               Param, ToggleParam, ShapeFunction, PhaseModulator
    preset.rs              Preset, Camera, Quaternion, Sampling, Phosphor
    layer.rs               Pipeline layer type
    ops.rs                 32 operation functions (R2 -> R2)
    lift.rs                18 lift functions (R3 displacement)
    registry.rs            OperationDef/LiftDef with fn pointers, labels, categories
    pipeline.rs            Pipeline application, pre-flatten, numerical Jacobian
    animation.rs           resolve_param(), apply_shape(), ParamPath
    engine.rs              AnimationEngine: tick(), weight system, quaternion resolve
    transition.rs          TransitionManager with cosine-eased blends
    frame_state.rs         FrameState (engine output consumed by renderer)
    notation.rs            Composition string, formula symbols, DOF counting
    morph.rs               MorphState, blend_presets, cosine-eased preset morphing
    preset_io.rs           Tolerant import/export, registry reconciliation
    factory_presets.rs     15 built-in presets as embedded JSON
    palette.rs             Palette struct + static data
  polyform-core/tests/     Test fixtures (golden vectors, reconciliation)
  polyform-app/src/        Application binary
    main.rs                App loop, action handlers, overlay rendering
    gpu.rs                 wgpu setup, compute/render/blit pipelines, buffer management
    camera.rs              Camera matrix computation (orbit, projection, distortion)
    ui.rs                  egui Performance UI, compact widgets, popup system
    preset_library.rs      Preset library directory management, section presets
    shaders/
      compute.wgsl         Grid generation + pipeline compute shader
      render.wgsl          Vertex/fragment shaders with Brown-Conrady distortion
      blit.wgsl            Fullscreen triangle + texture sampling for offscreen blit
    spout.rs               Spout texture sharing (Windows-only, SpoutDX FFI)
web/                       Parked web prototype (React + three.js)
archive/                   Legacy versions (v5, v6, v7)
```

## License

Not yet determined.
