# Polyform — Feature Priorities

## Shipped

### Lattice Family (Phase 2.9)
Five substrate types: Cartesian, polar, hexagonal, log-polar, bipolar. Per-type parameters. Smooth vertex morphing transitions (cosine-eased, 0.4s). Tempo uncapped to 1,000,000 BPM.

### Decoupled Operation Centers (Phase 2.10)
`cx, cy` shift params for conformal/radial ops (zPow, zInv, zExp, zLog, zSin, zCos, mobius, spiral, pinch, swirl, radialSine, angleWarp, ripple). Movable poles/branch points.

### Full Brown-Conrady Distortion (Phase 2.10)
k1-k4 radial terms, p1-p2 tangential distortion, decoupled distortion center, anamorphic squeeze. All as animatable ToggleParams.

### Caustic Overlay (Phase 2.10)
Visualization of det J = 0 fold/cusp curves on the lattice.

### Phosphor Rendering (Phase 2.12)
Beam quad rendering, HDR persistence buffer, bloom post-process (Gaussian with threshold), color grading (gain, saturation, gamma). All params as animatable ToggleParams.

### Visualization Overlays (Phase 2.12b)
Domain coloring, curl magnitude, conformal defect, normal shading.

### Performance UI Redesign
Compact param widgets (single-line + popup detail), responsive 3-5 column layout, top strip (tempo/preset/palette/viewport/spout/overlays), section-with-toggle pattern.

### Expanded Registries (Phase 2.13)
32 operations (7 new: wrap, quantize, clamp, joukowski, zTan, kaleidoscope, noise). 18 lifts (3 new: zSaddle, zTerrace, toroidal). Fixed mixed op CPU/GPU discrepancy.

### Preset System (Phase 2.11)
PresetFile wrapper with kind discriminator. Preset library at `~/.polyform/presets/` with subdirectories per section. Section presets (save/load operations, lift, camera, phosphor, sampling independently). Inline dropdown preset loading.

### Preset Morphing (Phase 2.11)
Cosine-eased crossfade between presets. Layer UUID matching (matching layers interpolate, unmatched fade in/out). Animation disabled during morph for smooth transitions. Morph duration adjustable via UI (default 3.0s).

### Factory Presets (Phase 2.13)
15 built-in presets across 5 categories: foundations (conformal zoo, mobius dance, spiral galaxy, sine lattice, kaleidoscope), complex compositions (crystal, warp field, pixel drift, event horizon, reef), camera-focused (tunnel, flyover, lens), phosphor character (glow, strobe).

### BPM Timing (Phase 2.13)
Bar-rate convention: Hz = BPM / 240, ratio=1 = one oscillation per bar (4 beats in 4/4). 120 BPM at ratio=1 gives a 2-second cycle. Toggle blend time (0.3s) separated from morph duration.

---

## Current Priority: Integration (Phase 2.14)

**Goal**: External control and capture.

---

## Deferred

These are valid but not blocking performance readiness:

- MIDI mapping (CC -> param, note -> layer toggle)
- Live + offline recording (native encoding)
- VST3/CLAP plugin format
- Hardware control surface design
- Multi-instance sync
