# Polyform Technical Brief

## What it is

Polyform is a **caustic synthesizer** — an instrument that produces geometric caustic patterns through refractive composition of mathematical maps on a visible 2D substrate. It belongs to the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits) but operates on a fundamentally different signal type: where traditional video synthesizers process raster signals or oscillator voltages, Polyform processes a coordinate manifold — a regular 2D grid that exists all at once and is deformed as a geometric object.

The visual output is the **caustic structure** of the composite map: the fold curves, cusps, and convergence patterns that emerge where the Jacobian of the composition degenerates. These are the same structures that appear as bright curves at the bottom of a swimming pool (envelopes of refracted light rays), but rendered here as deformed grid lines rather than light intensity. The grid is the probe that makes the caustic structure visible.

The instrument's method is **refractive composition**: an ordered stack of refractors — mathematical, displacement, and optical — each bending what flows through it. The camera is not a passive viewer; it is the terminal refractor, co-equal with the function-theoretic stages that precede it.

## What it produces

When you compose R2 -> R2 maps and render a grid through the composition, the visually dominant features are structurally determined by the singularity theory of the composite map:

- **Fold curves**: where the Jacobian determinant approaches zero and grid lines compress together. These are the generic singularities of smooth maps R2 -> R2 (Whitney folds).
- **Cusp points**: where fold curves meet and the map has a higher-order degeneracy. These appear as sharp convergence points in the deformed grid.
- **Caustic envelopes**: the image of the critical set under the map itself — the curves where grid lines pile up most densely. These are the primary visual features of the output.
- **Conformal structure**: when the operations are holomorphic (z^n, Mobius, exp, log, sin, cos), the map preserves angles locally, producing the characteristic circle-preserving geometry of conformal maps. Non-holomorphic operations (conjugate, |z|, reflect) deliberately break this structure.
- **Catastrophe unfoldings**: as parameters vary (especially under animation), the singularity structure of the composite map changes. Watching a cusp split into two folds, or folds merge and annihilate, is literally watching catastrophe unfoldings in the sense of Thom's classification.

The lift stage (displacement refractors) embeds the 2D caustic structure in 3D. The Jacobian-based lifts (`det J -> z`, `log|det J| -> z`) directly visualize the critical set as a 3D surface — the caustic surface of the composition. Height-based lifts (`|T| -> z`, `Re(T) -> z`) and wave lifts add depth that reveals structure invisible in the 2D projection.

The camera and distortion stages (optical refractors) then compose with this 3D embedding, producing parallax-dependent form identity — the same mathematical form appears structurally different from different vantage points, because the observer's position is itself a refractor.

## Architecture

### Refractive stack

The signal path is a linear composition of refractor stages. Each refractor takes input coordinates, applies a parameterized transformation, and passes the result to the next stage. The full stack, expressed as the signal flow formula the instrument displays on-screen:

```
D_{k1,k2}( P_{f,s}( V_{th,ph,r}( Q( ops(L) + lift ) ) ) )
```

Where:
- `L` — the substrate (regular 2D lattice)
- `ops` — function-theoretic refractors (R2 -> R2), composed right-to-left
- `lift` — displacement refractors (R2 -> R3), summed additively
- `Q` — SO(3) quaternion tumble
- `V` — camera position (orbit, distance, target, roll)
- `P` — projection refractor (focal length, sensor, anamorphic, shift)
- `D` — radial distortion refractor (Brown-Conrady k1, k2)

### Three refractor families

**Function-theoretic refractors** (operations): R2 -> R2 maps applied sequentially to every point on the substrate. Composition is non-commutative — reordering changes the caustic structure using the same degrees of freedom. Five categories:

| Category | Refractors | Mathematical character |
|----------|-----------|----------------------|
| Linear | translate, scale, rotate, shear, 2x2 matrix | Affine group, GL(2,R) |
| Complex | z^n, 1/z, exp(z), log(z), sin(z), cos(z), Mobius, z+c | Holomorphic / conformal maps on C |
| Polar | spiral, pinch/bulge, swirl, radial sine, angle warp | Polar coordinate nonlinearities |
| Wave | sine warp, ripple | Periodic displacement fields |
| Non-holomorphic | conjugate, reflect, \|z\|, mixed monomials | Orientation-reversing, folding maps |

Each refractor has typed parameters with defaults and ranges. Each has a blend factor (0 = identity bypass, 1 = fully active) that interpolates between the refractor's output and its input: `output = input + (refracted - input) * blend`.

The blend mechanism is what enables smooth transitions. Toggling a refractor on/off animates blend between 0 and 1 over a configurable duration. This means structural changes (adding/removing refractors) produce continuous visual change, not discontinuous jumps.

**Displacement refractors** (lift): applied to the post-operation lattice, producing R3 displacement vectors `[dx, dy, dz]` that are summed additively. Three categories:

| Category | Refractors | What they visualize |
|----------|-----------|-------------------|
| Height | \|T\| -> z, \|T\|-\|z\| -> z, det J -> z, log\|det J\| -> z, Re(T) -> z, Im(T) -> z, arg(T) -> z | Scalar properties of the composite map and its Jacobian |
| Wave | sine wave, sine xy, gaussian bump | Periodic/localized 3D displacement |
| 3D | spherical projection, conical lift, helical twist, noise, ripple xyz | Full-vector R3 displacement |

The Jacobian-based lifts are computed via finite differences: `det J = (df/dx)(dg/dy) - (df/dy)(dg/dx)` with epsilon = 0.01. This computation is deferred — only performed when a displacement refractor that needs it is active (lazy evaluation via `needsJacobian` flag).

**Optical refractors** (camera): the terminal stage of the stack. Unlike function-theoretic refractors which operate in the mathematical domain, optical refractors operate in screen space and viewing geometry. They include:

- **Position**: orbit azimuth/elevation, distance, target offset, roll — spherical coordinates around the origin with Gram-Schmidt stabilized up-vector and pole-degeneracy fallback
- **Projection**: perspective (physical camera model: focal length in mm, sensor size in mm, computed FOV) or orthographic (zoom factor). Anamorphic squeeze and lens shift via projection matrix pre-multiplication
- **Distortion**: Brown-Conrady radial model (k1, k2 coefficients). Applied by projecting world coordinates to NDC, displacing radially by `factor = 1 + k1*r^2 + k2*r^4`, then unprojecting back to world space. This operates on every vertex every frame.

The essay-level claim that optical refractors are co-equal with mathematical refractors is supported by the implementation: every optical parameter uses the same `ToggleParam` interface as mathematical parameters, with the same blend, animation, ratio, and phase offset capabilities.

### Param interface

Every animatable scalar in the system uses a uniform interface:

```
Param {
  value: number       // current value
  min: number         // range minimum
  max: number         // range maximum
  anim: boolean       // animation enabled
  animMode: "loop" | "sweep"
  ratio: number       // tempo multiplier
  phaseOffset?: {     // nested Param for phase modulation
    value, min, max, anim, animMode, ratio
  }
}
```

`ToggleParam` extends this with an `on: boolean` for enable/disable with blend transitions.

Every refractor layer additionally carries:
- `id`: stable UUID (persists across save/load/reorder)
- `type`: registry key
- `blend`: Param (0 = identity, 1 = fully active)
- `blendTimeOverride`: optional per-layer transition duration

### Animation system

Animation is decoupled from React rendering. The system maintains two parallel state representations:

1. **React state** — the source of truth for structural configuration (which refractors are active, parameter ranges, animation flags). Updated via `useState`/`setState`. Drives UI rendering.

2. **Ref-based swept values** (`animValuesRef`) — the source of truth for current animated parameter values. Updated every frame by `sweepAllParams()` without triggering React re-renders. Drives Three.js rendering.

React state is synced to swept values at ~10fps for UI slider feedback. This architectural split eliminates the performance problem of 60fps `setState` calls while keeping UI sliders responsive.

**Sweep modes**:

- **Loop**: value advances linearly at `speed * span` per second, wrapping at range boundaries. Phase offset shifts the wrap point. Produces periodic motion for parameters like rotation angle.
  ```
  base += dt * speed * span
  base = ((base - min) % span + span) % span + min
  value = ((base + phaseOffset * span) % span) + min
  ```

- **Sweep**: value oscillates via cosine interpolation between min and max. Phase offset shifts the oscillation phase. Produces smooth back-and-forth motion.
  ```
  phase = clock * speed + phaseOffset
  t = 0.5 - 0.5 * cos(phase * 2pi)
  value = min + t * (max - min)
  ```

**Tempo system**: a single global tempo (in cycles/second, displayed as BPM where 1.0 = 60 BPM) with per-parameter ratio multipliers. The ratio determines how many cycles a parameter completes per tempo cycle.

Ratio presets span eight families:
- Integer (1-64)
- Reciprocal (1/2 through 1/64)
- Harmonic (2/3, 3/4, 5/4, etc.)
- Just intonation (16/15 through 2/1)
- Fibonacci (1/8, 2/5, 8/13, etc.)
- Metallic means (phi, silver ratio, bronze, plastic number)
- Irrational (pi, e, sqrt(2), sqrt(3), sqrt(5))
- Powers of 2 (1/128 through 128)

Rational ratios produce closed polyrhythms (the animation eventually repeats). Irrational ratios produce quasi-periodic motion (the animation never exactly repeats). This is the "polydimensional coupling" — many independent oscillatory degrees of freedom coupled through the shared substrate, with ratio relationships determining whether the coupling is periodic or aperiodic.

### Degree-of-freedom accounting

The instrument maintains a live DOF readout (the dimension count) that partitions the active configuration into:

- **Structural DOFs**: the number of parameters across all active refractors, including blend. For each active refractor: `parameter_count + 1` (the +1 is for blend). Plus 2 for the base substrate.
- **Temporal DOFs**: the number of independently animated parameters. Each animated parameter contributes 1. Each animated phase offset contributes 1.
- **Total**: structural + temporal.

The readout also tracks: number of active operators, individual layer details (which params are animated, at what ratio, current blend value). This is computed every ~125ms (throttled at ~8fps) and displayed as an overlay.

The DOF count is not a debug feature. It is the philosophical core of the instrument made visible — the claim that "dimension" means "independent parameter count" rather than "spatial axis count," and that the instrument makes this felt by showing the count rise and fall as you build up a patch.

### Geometry rendering

The substrate is rendered as a set of line segments in Three.js. For a lattice density of N:

- Generate `2 * (N + 1)` lines: N+1 horizontal (constant x, sweep y) and N+1 vertical (sweep x, constant y)
- Each line is sampled at 80 points uniformly spaced across the domain extent
- For each sample point `(x0, y0)`:
  1. Apply the pre-flattened operation pipeline: `(x, y) = applyPipelineFlat(x0, y0, flatLayers)`
  2. Apply the pre-flattened displacement pipeline: `(dx, dy, dz) = applyDispPipelineFlat(x, y, x0, y0, flatDispLayers, jSignedFn)`
  3. Set vertex position to `(x + dx, dz, y + dy)` (note the axis swap: mathematical y -> Three.js z, displacement z -> Three.js y)
  4. If radial distortion is active: project to NDC, apply Brown-Conrady radial displacement, unproject back to world space

**Geometry buffer pool**: pre-allocated `Float32Array` buffers are reused every frame. The pool is sized to the maximum density (51 lines per axis) on first allocation and never reallocated during normal operation. Each line has two Three.js `Line` objects: a sharp primary line (opacity 0.95) and a glow line (opacity 0.18) for bloom effect. Materials are shared across all lines.

**Pipeline pre-flattening**: before the per-point inner loop, layer parameters are extracted into flat objects (`{ key: value }`) and inactive layers are filtered out. This avoids per-point object property lookups and conditional checks inside the hot loop.

**Jacobian computation**: finite-difference approximation using epsilon = 0.01. Three pipeline evaluations per point that needs it (center, +dx, +dy). Deferred — only computed when a displacement refractor with `needsJacobian: true` is active. The Jacobian determinant is the signed area expansion factor of the composite map at that point.

### Transition system

A transition manager handles smooth state changes for indirect events (toggle, preset load, reorder). Transitions are cosine-eased interpolations:

```
eased = 0.5 - 0.5 * cos(progress * pi)
value = from + (to - from) * eased
```

Three categories of transitions:

1. **Refractor toggle**: blend animates 0 -> 1 (enable) or 1 -> 0 (disable, then set `on: false` on completion). Default 0.5s duration, overridable per-layer.
2. **View parameter toggle**: value animates between current visual value and a neutral value (identity position, default focal length, etc.). Animation state is saved on toggle-off and restored on toggle-on, with a 2-second resync blend to rejoin the natural sweep.
3. **Reorder**: blend -> 0, swap position in array, blend -> 1. This avoids interpolating composition order (which is mathematically undefined) by using blend as a smooth structural transition.

### Neutralize / restore

A full-stack teardown and rebuild sequence used as bookends for recording:

**Neutralize** (peel away in reverse pipeline order):
1. Distortion off (all params simultaneously, 500ms)
2. Lens off (all params simultaneously, 500ms)
3. Camera position off (all params simultaneously, 500ms)
4. Quaternion off (angle -> 0, 500ms)
5. Displacement refractors off (reverse order, 500ms each)
6. Operation refractors off (reverse order, 500ms each)

**Restore** (rebuild in forward pipeline order):
1. Operation refractors on (forward order, blend 0 -> 1)
2. Displacement refractors on (forward order, blend 0 -> 1)
3. Quaternion on (angle 0 -> saved)
4. Camera position on (all params, neutral -> saved)
5. Lens on (all params, neutral -> saved)
6. Distortion on (all params, neutral -> saved)

A snapshot of which refractors/params were active is stored before neutralize and used to drive restore. Bypass flags allow the neutralize system to call toggle functions that would otherwise be blocked during the neutralize state.

### Recording

**Live recording**: captures the composite canvas (WebGL render + 2D overlay text) via `captureStream(0)` with manual `requestFrame()` calls. VP8 or VP9 codec at 20Mbps. The composite canvas is DOM-attached (positioned off-screen) for cross-browser `captureStream` compatibility.

**Offline rendering**: deterministic frame-by-frame rendering with a synthetic clock (`dt = 1/fps` per frame). Completely bypasses React state — operates on a deep-copied `offState` object that is mutated directly. The offline render has four phases:

1. **Hold**: 0.5s of flat grid (clean loop-in point)
2. **Restore**: rebuild from flat grid to full state (each refractor group fades in sequentially)
3. **Content**: main duration at target framerate, sweeping all params with deterministic clock
4. **Neutralize**: reverse teardown back to flat grid (clean loop-out point)

Frame pacing uses `requestAnimationFrame` for vsync accuracy with `setTimeout` pre-waits for long frame periods. VP9 at 40Mbps for offline output.

### Preset system

A preset is a JSON document (schema v1) containing the complete refractive stack configuration:

```
{
  schema: "polyform.preset",
  version: 1,
  metadata: { name, description, author, created, modified, tags },
  tempo: Param,
  palette: string,
  pipeline: {
    operations: [{ id, type, enabled, blend, params }],
    lift: [{ id, type, enabled, blend, params }]
  },
  camera: {
    position: { orbitX, orbitY, distance, targetX, targetY, roll },
    lens: { projection, focalLength, sensorSize, orthoZoom, anamorphic, shiftX, shiftY },
    distortion: { k1, k2 }
  },
  quaternion: { on, mode, anim, angle, angleRatio, axes, driftSpeed },
  sampling: { scale, density },
  display: { showFormula, showDimCount, showActiveLayers, showParamValues, showFps }
}
```

Validation is graceful: unknown fields are ignored (forward compatibility), missing fields fall back to defaults, type mismatches warn but don't fail. Layer IDs are stable UUIDs that persist across save/load. The schema has a version migration hook placeholder for future format changes.

On import, each layer type is validated against the current registry. Unknown types are dropped with a warning. Missing params are filled from registry defaults. Extra params are stripped with a warning.

The preset format is the portable asset across all project phases (web -> Rust -> C++). The renderer is replaceable; the preset format is not.

### Display overlays

Four overlay regions, each at a fixed canvas position:

- **Bottom-left**: signal flow formula — the refractive stack expressed in mathematical notation (e.g., `D_{k1,k2}(P_{f,s}(V_{th,ph}(z^n . R(L) + |T|-|z|)))`)
- **Top-left**: active layers — per-refractor DOF breakdown with blend values, animation details, and ratios
- **Top-right**: dimension count — structural/temporal/total DOF partition with system info (palette, projection, FOV, scale, density, quaternion state)
- **Bottom-right**: FPS readout and recording status

All overlays are rendered via a shared `drawOverlays()` function onto a 2D canvas that composites with the WebGL render. Both live and offline recording capture these overlays.

## Implementation

### Current stack

- **React 19** — UI rendering, state management
- **Three.js r184** — 3D line rendering, camera, projection matrices
- **Vite 8** — dev server, HMR, build

### File structure

| File | Lines | Purpose |
|------|-------|---------|
| `polydimensional-coupling-v7.jsx` | ~5000 | Monolithic component: registries, pipeline, rendering, UI |
| `preset-schema.ts` | ~636 | Preset JSON schema types and validation |
| `preset-io.ts` | ~380 | State <-> preset serialization bridge |
| `main.jsx` | 5 | Entry point |

The v7 component is organized internally as:
- Lines 8-476: refractor registries (TRANSFORMS, DISPLACEMENTS, PALETTES, RATIO_GROUPS)
- Lines 530-700: pipeline functions (apply, pre-flatten, composition string, DOF counting)
- Lines 1358-1470: transition manager, preset bridge functions
- Lines 1475-3700: React component (state, animation loop, Three.js setup, geometry rebuild, layer manipulation)
- Lines 3700-4200: main JSX return (UI layout)
- Lines 4200-5000: sub-components (PipelineCard, ViewLayerCard, form elements)

### Performance characteristics

- **Hot path**: `rebuildGeometry()` runs every frame. For density 27 (default), this evaluates the full refractive stack at 56 lines x 81 samples = 4,536 points per frame. At density 51: 104 lines x 81 = 8,424 points.
- **Zero allocation in hot path**: geometry buffers are pre-allocated Float32Arrays. Pipeline layers are pre-flattened once per frame. No object creation inside the per-point loop.
- **React decoupled**: animation runs in refs at 60fps. React state synced at ~10fps. Structural events (toggle, reorder) trigger `setState` directly.
- **Jacobian cost**: when active, triples the pipeline evaluation count for affected points (center + two finite-difference samples).

## Trajectory

**Phase 1** (current): Web prototype. React + Three.js. Design validation surface. CPU-bound pipeline evaluation.

**Phase 2** (planned): Rust native app. wgpu compute shaders for parallel grid evaluation. egui control surface. SpoutCapture bridge for Resolume output. Same preset format.

**Phase 3** (speculative): C++ rewrite with native Spout SDK. VST3/CLAP plugin potential. Hardware control surface. Multi-instance sync.
