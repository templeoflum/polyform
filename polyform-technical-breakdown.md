# Polyform: Technical Breakdown

This document describes the complete computational architecture of Polyform, a real-time spatial synthesizer built in Rust (wgpu + egui). It is written for machine consumption — another AI session developing identity and language around this instrument.

---

## 1. What It Is

Polyform generates animated geometric imagery by applying mathematical transformations to a 2D grid (lattice) in real time. The grid is always visible — it is the identity-defining anchor. Every visual is a transformation of this substrate.

The signal flow is:

```
2D lattice -> Operations (R2 -> R2) -> Lift (R3 displacement) -> Camera -> Screen
```

Each stage composes ordered layers of mathematical functions. All parameters animate against a global master phase via a deterministic motion protocol. The result is a performable instrument where mathematical structure is visible and controllable.

---

## 2. The Lattice

The lattice is a regular 2D grid of points in `[-scale, scale]` on both axes, with `density` controlling the number of grid lines (default 27 lines in each direction). Points are connected as line strips — one set runs along x, one along y. This creates the wireframe that makes deformations legible.

**Key numbers:**
- `MAX_VERTICES = 8424` (pre-allocated, not per-frame)
- Vertex count = `density * density * 2` (two orientations)
- Default: `scale = 0.6`, `density = 27`

Grid generation happens on the GPU via compute shader. Each vertex stores `(x0, y0, line_idx)`.

---

## 3. Stage 1: Operations (R2 -> R2)

Operations are composable 2D maps applied sequentially to each grid point. Composition is non-commutative — order matters. There are 25 operation types in 6 categories:

### 3.1 Linear / Affine (5)

| Key | Formula | Params |
|-----|---------|--------|
| `translate` | `(x + tx, y + ty)` | `tx [-1,1]`, `ty [-1,1]` |
| `scale` | `(x*sx, y*sy)` | `sx [0.1,3]`, `sy [0.1,3]` |
| `rotate` | `(x cos θ - y sin θ, x sin θ + y cos θ)` | `theta [0, 2π]` periodic |
| `shear` | `(x + kx*y, y + ky*x)` | `kx [-2,2]`, `ky [-2,2]` |
| `matrix` | `(ax + by, cx + dy)` | `a,b,c,d [-3,3]` — general GL(2) |

### 3.2 Complex / Conformal (8)

These treat `(x, y)` as the complex number `z = x + iy`:

| Key | Formula | Params |
|-----|---------|--------|
| `zPow` | `z^n` (polar: `r^n * e^{inθ}`) | `n [0.1, 5]` |
| `zInv` | `1/z̄` (with ε-guard `d = x²+y²+1e-6`) | none |
| `zExp` | `e^z = e^x(cos y + i sin y)` | none |
| `zLog` | `log z = ln|z| + i arg(z)` (ε-guarded) | none |
| `zSin` | `sin(z) = sin(x)cosh(y) + i cos(x)sinh(y)` | none |
| `zCos` | `cos(z) = cos(x)cosh(y) - i sin(x)sinh(y)` | none |
| `mobius` | `(az+b)/(cz+d)` real coefficients, ε-guarded | `a,b,c,d [-2,2]` |
| `zPlusC` | `z + c` (complex translation) | `cx,cy [-2,2]` |

### 3.3 Polar / Radial (5)

| Key | Formula | Params |
|-----|---------|--------|
| `spiral` | `r * e^{i(θ + k ln r)}` — logarithmic spiral | `k [-5,5]` |
| `pinch` | `r^p * e^{iθ}` — radial power | `p [0.2, 4]` |
| `swirl` | `θ' = θ + k * exp(-r²/σ²)` — gaussian angular twist | `k [-8,8]`, `sigma [0.05, 2]` |
| `radialSine` | `r' = r + amp * sin(freq*r + phase)` | `amp [0, 0.5]`, `freq [0.5, 20]`, `phase [0, 2π]` |
| `angleWarp` | `θ' = θ + amp * sin(k*θ)` | `amp [0, 2]`, `k [1, 12]` |

### 3.4 Wave / Periodic (2)

| Key | Formula | Params |
|-----|---------|--------|
| `sineWarp` | `(x + amp*sin(freq*y+phase), y + amp*sin(freq*x+phase))` | `amp [0, 0.6]`, `freq [0.5, 20]`, `phase [0, 2π]` |
| `ripple` | Radial: `(r + amp*sin(freq*r+phase))` preserving angle | `amp [0, 0.5]`, `freq [0.5, 20]`, `phase [0, 2π]` |

### 3.5 Non-Holomorphic (4)

| Key | Formula | Params |
|-----|---------|--------|
| `conjugate` | `z̄ = (x, -y)` | none |
| `reflect` | Reflection across line at angle θ: `(x cos 2θ + y sin 2θ, x sin 2θ - y cos 2θ)` | `theta [0, π]` |
| `abs` | `(|x|, |y|)` | none |
| `mixed` | `(x²y³, -x³y²)` — monomial non-holomorphic map | none |

### 3.6 Utility (1)

| Key | Formula | Params |
|-----|---------|--------|
| `blend` | `x0*(1-amount) + x*amount` — lerps toward pre-pipeline position | `amount [0, 1]` |

### 3.7 Pipeline Composition

For a point `(x0, y0)` and layers `[L₁, L₂, ..., Lₙ]`, each with blend `bᵢ ∈ [0,1]`:

```
(x, y) = (x0, y0)
for each layer Lᵢ:
    (ox, oy) = Lᵢ.apply(x, y, params, x0, y0)
    x = x + (ox - x) * bᵢ
    y = y + (oy - y) * bᵢ
```

Each layer lerps between identity and its full effect by its blend value. When blend=0, the layer is identity (no effect). When blend=1, the layer is fully applied. This makes every layer independently controllable and smoothly toggable.

The uniform signature for all operations is: `fn(x, y, params, x0, y0) -> (x', y')` where `x0, y0` are the original pre-pipeline coordinates (only used by the `blend` operation).

---

## 4. Stage 2: Lift (R3 Displacement)

After operations transform the 2D positions, lifts displace the result into 3D. Each lift produces a displacement vector `(dx, dy, dz)` that is added to the current position, scaled by the layer's blend. There are 15 lift types in 4 categories:

### 4.1 Height-style (z displacement only) — 7

| Key | Formula | Params |
|-----|---------|--------|
| `zMagnitude` | `dz = amp * |T|` (magnitude of transformed position) | `amp [-2, 2]` |
| `zDisplacement` | `dz = amp * (|T| - |z₀|)` (change in magnitude) | `amp [-2, 2]` |
| `zLogJac` | `dz = amp * ln|det J|` (log of Jacobian determinant) | `amp [-2, 2]` |
| `zSignedJac` | `dz = amp * det J` (signed Jacobian determinant) | `amp [-2, 2]` |
| `zReal` | `dz = amp * Re(T)` (real part of transformed position) | `amp [-2, 2]` |
| `zImag` | `dz = amp * Im(T)` (imaginary part) | `amp [-2, 2]` |
| `zArg` | `dz = amp * arg(T)` (angle of transformed position) | `amp [-2, 2]` |

### 4.2 Wave / Bumps (z) — 3

| Key | Formula | Params |
|-----|---------|--------|
| `zSineWave` | `dz = amp * sin(freq * |T| + phase)` | `amp [-2,2]`, `freq [0.1,20]`, `phase [0,2π]` |
| `zSineXY` | `dz = amp * sin(fx * Tx) * sin(fy * Ty)` | `amp [-2,2]`, `fx [0.1,15]`, `fy [0.1,15]` |
| `zGaussian` | `dz = amp * exp(-(Tx²+Ty²)/σ²)` | `amp [-2,2]`, `sigma [0.05, 3]` |

### 4.3 Full 3D Displacement — 5

| Key | Formula | Params |
|-----|---------|--------|
| `spherical` | Maps flat grid onto sphere via `φ = atan(|T|/r)` | `r [0.1, 3]` |
| `conical` | `dz = amp * |T|` (same as zMagnitude but in 3d category) | `amp [-2, 2]` |
| `helical` | Twist around axis: `dz = lift*|T|`, xy rotated by `twist*z` | `lift [-2,2]`, `twist [-8,8]` |
| `noise3D` | Pseudo-noise via sin/cos combinations in all 3 axes | `amp [-1,1]`, `freq [0.5,15]`, `seed [0,2π]` |
| `ripple3D` | Radial wave with xy + z displacement | `amp [-1,1]`, `freq [0.5,20]`, `phase [0,2π]` |

### 4.4 Jacobian Computation

Two lifts (`zLogJac`, `zSignedJac`) require the Jacobian determinant of the operations pipeline. This is computed via finite differences:

```
h = 0.001 (GPU) / 1e-5 (CPU)
J = | ∂f/∂x  ∂f/∂y |
    | ∂g/∂x  ∂g/∂y |
det(J) = (∂f/∂x)(∂g/∂y) - (∂f/∂y)(∂g/∂x)
```

Computed by evaluating the full operations pipeline at `(x, y)`, `(x+h, y)`, and `(x, y+h)`. The Jacobian is only computed when at least one active lift requires it (tracked by `any_lift_needs_jacobian` flag).

### 4.5 Lift Pipeline Composition

```
(world_x, world_y, world_z) = (Tx, 0, Ty)   // lattice x→world x, lattice y→world z
for each lift layer Lᵢ:
    (dx, dy, dz) = Lᵢ.apply(Tx, Ty, params, x0, y0, det_j)
    world_x += dx * bᵢ
    world_y += dy * bᵢ    // height axis
    world_z += dz * bᵢ
```

The coordinate mapping places the lattice on the xz plane with lift displacement going into the y axis (height).

---

## 5. Stage 3: Camera

The camera system has four subsystems: orbit position, lens, quaternion tumble, and lens distortion. Each camera parameter is a `ToggleParam` — it has an on/off switch independent of its animation state.

### 5.1 Orbit Position

Spherical coordinates around a target point:

```
eye = target + distance * (sin(orbit_y) * sin(orbit_x), cos(orbit_y), sin(orbit_y) * cos(orbit_x))
target = (target_x, 0, target_y)
```

| Param | Default | Range | Notes |
|-------|---------|-------|-------|
| `orbitX` | 0.0 | [0, 2π] | Azimuth, default ratio 1.0 |
| `orbitY` | 0.1 | [0.1, π-0.1] | Elevation, default ratio 1/φ ≈ 0.618 |
| `distance` | 3.5 | [0.8, 12] | Orbit radius |
| `targetX` | 0.0 | [-2, 2] | Look-at x offset |
| `targetY` | 0.0 | [-2, 2] | Look-at z offset |
| `roll` | 0.0 | [0, 2π] | Camera roll around forward axis |

Pole degeneracy handled: when looking straight up/down, up-vector falls back to world Z.

Roll is applied as axis-angle rotation around the forward vector after Gram-Schmidt orthogonalization.

### 5.2 Lens

| Param | Default | Range | Notes |
|-------|---------|-------|-------|
| `projection` | Perspective | - | Enum: Perspective / Orthographic |
| `focalLength` | 35mm | [8, 200] | FOV = 2 * atan(sensor / (2 * focal)) |
| `sensorSize` | 36mm | [4, 80] | Full-frame equivalent |
| `orthoZoom` | 2.0 | [0.3, 6] | Orthographic half-extent |
| `anamorphic` | 1.0 | [0.3, 3] | Horizontal squeeze factor |
| `shiftX` | 0.0 | [-1, 1] | Lens shift (tilt-shift) |
| `shiftY` | 0.0 | [-1, 1] | Lens shift (tilt-shift) |

Lens correction is applied as a 4x4 matrix: `lens_correction = translate(shiftX, shiftY) * scale(anamorphic, 1)`, multiplied into the projection matrix.

### 5.3 Quaternion Tumble

An additional model-space rotation applied before the view matrix. Uses axis-angle representation converted to a quaternion `[w, x, y, z]`.

**Two modes:**
- **Fixed**: Axis `(ax, ay, az)` resolved from full `Param` objects (individually animatable). Angle from `Param`.
- **Drift**: Axis computed from time-varying sinusoidal functions of a separate `drift_phase` accumulator (deliberately decoupled from master phase): `ax = sin(0.7t) + 0.3*cos(1.3t)`, `ay = cos(0.5t) + 0.3*sin(1.7t)`, `az = sin(0.9t + 1.2)`.

**Blend system**: On/off toggle uses SLERP-based blending — the effective angle is `resolved_angle * quat_blend`, where `quat_blend` transitions 0→1 smoothly. This prevents multi-rotation spinning during blend transitions.

**Drift↔Fixed transitions**: NLERP between the quaternion at mode-switch time and the new mode's target quaternion.

**Quaternion defaults (chosen for aesthetic irrational ratios):**
- `angle`: `[0, 2π]`, anim=true, ratio=1.0
- `ax`: 0.5, ratio=φ ≈ 1.618
- `ay`: 1.0, ratio=1.0
- `az`: 0.3, ratio=1/φ ≈ 0.618

### 5.4 Lens Distortion (Brown-Conrady)

Applied in the vertex shader in NDC space:

```
r² = ndc.x² + ndc.y²
r⁴ = r² * r²
factor = 1 + k1*r² + k2*r⁴
ndc' = ndc * factor
```

Negative k = barrel distortion (fisheye), positive k = pincushion.

| Param | Default | Range |
|-------|---------|-------|
| `k1` | 0.0 | [-1, 1] |
| `k2` | 0.0 | [-1, 1] |

### 5.5 View-Projection Assembly

```
model = quaternion rotation (or identity if quat off)
view = look_at(eye, target, rolled_up)
projection = perspective(fov, aspect) or orthographic(zoom, aspect)
lens_correction = translate(shift) * scale(anamorphic, 1)
VP = lens_correction * projection * view * model
```

Distortion is not in the VP matrix — it's applied per-vertex in the shader.

---

## 6. Animation: Parameter Motion Protocol

All animation is driven by a single deterministic system. There are no per-param accumulators or clock-based sweep systems. Every animated value is a pure function of the master phase plus configuration.

### 6.1 Master Phase

A single scalar `M` grows unbounded:

```
M += dt * tempo
```

Where `dt` is the frame time delta and `tempo` is the global tempo param (default 0.3). `M` does NOT wrap — each param's modular arithmetic handles its own phase computation, ensuring fractional ratios work correctly across wrap boundaries.

### 6.2 Param Resolution

Every animatable scalar in the system uses the `Param` struct:

```rust
Param {
    value: f64,     // base/static value (used when anim=false or weight=0)
    min: f64,
    max: f64,
    anim: bool,     // desired animation state
    ratio: f64,     // frequency relative to master (default 1.0)
    offset: f64,    // phase offset in [0, 1)
    shape: ShapeFunction,  // Cosine (default) or Sawtooth
    modulator: Option<PhaseModulator>,  // optional FM modulation
}
```

The core resolution function:

```
resolve_param(param, master_phase, anim_weight):
    if anim_weight <= 0 or !param.anim: return param.value

    // Carrier phase
    r = safe_ratio(param.ratio)
    phase = (M * r + param.offset) mod 1.0

    // Optional FM modulation
    if modulator and modulator.depth > 0:
        mod_phase = (M * safe_ratio(mod.ratio)) mod 1.0
        phase = (phase + mod.depth * apply_shape(mod.shape, mod_phase)) mod 1.0

    // Shape function: phase [0,1] -> t [0,1]
    t = apply_shape(param.shape, phase)

    // Scale to param range
    anim_value = min + t * (max - min)

    // Weighted blend between base and animated value
    return (1 - anim_weight) * param.value + anim_weight * anim_value
```

### 6.3 Shape Functions

Two shape functions map normalized phase `[0, 1]` to value `[0, 1]`:

- **Cosine** (default): `t = 0.5 - 0.5 * cos(2π * phase)` — smooth ping-pong, zero derivative at extremes. No discontinuity at wrap point (phase=0 and phase=1 both give t=0).
- **Sawtooth**: `t = phase` — linear ramp from 0 to 1, wrapping discontinuity at reset. Used for continuous rotation.

### 6.4 FM Modulation

Optional per-param `PhaseModulator`:

```rust
PhaseModulator {
    ratio: f64,       // modulator frequency relative to master
    depth: f64,       // modulation amount [0, 1]
    shape: ShapeFunction,
}
```

The modulator shifts the carrier phase: `carrier_phase += depth * shape(M * mod_ratio)`. This produces frequency modulation effects — the animated parameter's oscillation speeds up and slows down rhythmically.

### 6.5 Anim Weights

Every param path has a smooth weight `∈ [0, 1]` controlling the blend between static and animated value. This replaces hard-switching anim on/off:

```
AnimWeight {
    current: f64,
    target: f64,
    rate: f64,  // 1/duration — change per second
}
```

Weights advance linearly toward their target each frame. Default weight is 1.0 (fully animated when anim=true). When anim is toggled off, the weight transitions 1→0 over 0.3s, freezing the param at its current value. When toggled on, 0→1 over 0.3s, morphing from static to animated.

The `ToggleParam` (camera params) uses two independent weight channels:
- **Toggle weight** (`view_key` path): blends between neutral value and active value (for on/off toggle)
- **Anim weight** (`view_key.anim` path): blends between static and animated within the active value

This separation prevents toggle and anim transitions from interfering with each other.

### 6.6 Key Properties

- **Deterministic**: same `M` + same config = same output. Always.
- **No accumulators**: no per-param state that can drift or desync.
- **No sweep-vs-transition conflicts**: the entire old class of bugs is structurally eliminated.
- **Smooth by construction**: all discontinuities are eliminated by weight interpolation.

---

## 7. Structural Transitions

Separate from the animation weight system, a `TransitionManager` handles structural changes that need to animate between two values over time:

- Layer toggle (blend 1→0 or 0→1)
- Preset load value morphing
- Layer reorder (fade out, swap, fade in)

### 7.1 Cosine Easing

```
progress = elapsed / duration
eased = 0.5 - 0.5 * cos(progress * π)
value = from + (to - from) * eased
```

Symmetric ease-in-out. Transitions run on wall-clock time (not anim time), so they continue even when the animation is paused.

### 7.2 Layer Toggle Flow

**Disable**: Start transition from current visual blend to 0.0. On completion, fire `DisableLayer` action to set `layer.enabled = false`.

**Enable**: Set `layer.enabled = true`, `blend.value = 0.0`. Start transition from 0.0 to restored blend value. On completion, fire `RestoreBlendValue` action.

---

## 8. GPU Pipeline

The rendering uses wgpu with a compute-then-render architecture.

### 8.1 Compute Pass

The compute shader (`compute.wgsl`, workgroup_size=64) runs all 25 operations and 15 lifts on the GPU. Each vertex is processed independently:

1. Read grid vertex `(x0, y0)` from input buffer
2. Apply operations pipeline (sequential, per-layer blend)
3. Optionally compute numerical Jacobian (3 pipeline evaluations via finite differences, h=0.001)
4. Apply lift pipeline (additive displacement, per-layer blend)
5. Write `(world_x, world_y, world_z)` to output buffer

**Buffer layout:**
- Input: `array<GridVertex>` where `GridVertex = { x0: f32, y0: f32, line_idx: u32, _pad: u32 }`
- Output: `array<TransformedVertex>` where `TransformedVertex = { vx: f32, vy: f32, vz: f32, _pad: f32 }`
- Uniforms: `PipelineUniforms = { num_ops: u32, num_lifts: u32, any_needs_jacobian: u32, _pad: u32 }`
- Op chain: `array<OpDescriptor>` where each descriptor = `{ type_index: u32, blend: f32, needs_origin: u32, num_params: u32, params: vec4<f32> }`
- Lift chain: `array<LiftDescriptor>` (same structure with `needs_jacobian` instead of `needs_origin`)

Operations and lifts are dispatched via `switch` statement on `type_index`. Max 4 params per layer packed into a `vec4<f32>`.

### 8.2 Render Pass

The render shader (`render.wgsl`) reads from the compute output:

- **Vertex shader**: Reads `TransformedVertex`, applies view-projection matrix, then applies Brown-Conrady distortion in NDC space.
- **Fragment shader**: Returns a flat color from uniforms (the palette line color).

Render uniforms: `{ view_projection: mat4x4<f32>, line_color: vec4<f32>, k1: f32, k2: f32, _pad: vec2<f32> }`

Lines are drawn as line-strip primitives. The grid has two line sets (horizontal and vertical), each rendered as separate draw calls indexed by `line_idx`.

### 8.3 Buffer Management

All buffers are pre-allocated at `MAX_VERTICES = 8424` capacity. No per-frame allocation. Buffer contents are updated each frame by uploading resolved param values from the CPU-side `FrameState` into the GPU descriptor arrays. The grid input buffer is only regenerated when density or scale changes.

---

## 9. FrameState

The engine's `tick()` produces a `FrameState` — a complete snapshot of all resolved values for one frame. This is the contract between the animation engine and the renderer.

```rust
FrameState {
    // Pipeline (resolved layer states with blend + params as f64 values)
    operations: Vec<FlatLayerState>,
    lifts: Vec<FlatLayerState>,
    any_lift_needs_jacobian: bool,

    // Camera (all resolved to final f64 values)
    orbit_x, orbit_y, distance, target_x, target_y, roll,
    projection: ProjectionMode,
    focal_length, sensor_size, ortho_zoom,
    anamorphic, shift_x, shift_y,
    k1, k2,

    // Quaternion
    quat_on: bool,
    quat: [f64; 4],          // final [w,x,y,z] for rendering
    quat_angle: f64,          // resolved angle (for UI display)
    quat_axis: [f64; 3],      // resolved axis (for UI display)

    // Sampling
    scale: f64,
    density: f64,

    // Display
    palette_key: String,
    has_active_layers: bool,
}
```

The `FlatLayerState` contains `type_key: String`, `blend: f64`, and `params: HashMap<String, f64>` — all resolved from Param objects via `resolve_param()`.

---

## 10. Engine Tick Flow

Each frame, `AnimationEngine::tick(preset, dt, wall_dt)` executes:

1. **Advance master phase**: `M += dt * safe_ratio(tempo.value)` — unbounded, monotonically increasing
2. **Advance wall clock + weights**: `wall_clock += wall_dt`, advance all AnimWeights toward their targets linearly
3. **Process structural transitions**: tick the TransitionManager, produce updates and completed actions
4. **Advance drift phase**: `drift_phase += dt * drift_speed` (for quaternion drift mode)
5. **Resolve all params**:
   - Operations: for each layer, resolve blend and all params via `resolve_param(param, M, weight)`
   - Lifts: same as operations
   - Camera: resolve each ToggleParam via two-channel weight system (toggle weight + anim weight)
   - Quaternion: resolve angle + axis params, build axis-angle quaternion, apply SLERP blend
   - Sampling: resolve scale and density
6. **Build FrameState**: assemble all resolved values into the output struct

The engine does NOT mutate the preset during tick — it takes an immutable reference. Structural mutations (layer enable/disable from transition completion) are returned as actions and applied separately.

**Timing modes:**
- Live: `dt = wall_delta`, `wall_dt = wall_delta`
- Offline: `dt = 1/fps`, `wall_dt = 1/fps`
- Paused: `dt = 0`, `wall_dt = wall_delta` (transitions still run)

---

## 11. Palette System

Four-color scheme: `{ bg, grid, warp, glow }`. All hex strings.

| Key | Reference | Character |
|-----|-----------|-----------|
| `phosphor` | Tektronix 465, P1 phosphor | Canonical oscilloscope green |
| `p31` | Tektronix 2465, P31 phosphor | Blue-green CRT / radar |
| `amber` | IBM 5151, Wyse WY-50 | Warm amber monitor |
| `white` | Fairlight CMI, P4 phosphor | Cool white vector display |
| `rutt-etra` | Rutt-Etra video synthesizer | Electric blue scan lines |

Backward-compatible aliases: `acid` → `phosphor`, `cyan` → `p31`, `blush`/`void` → `white`.

---

## 12. Notation System

### 12.1 Signal Flow (Composition String)

A live mathematical formula showing the active composition chain. Built right-to-left for operations (math convention), additive for lifts:

- Empty: `L` (bare lattice)
- One op: `R(L)` (rotate applied to lattice)
- Two ops: `S . R(L)` (scale composed with rotate)
- With lift: `S . R(L) + |T| -> z`
- With quaternion: `Q(S . R(L))`
- With camera: `V_{th,ph}(Q(S . R(L)))` (view with azimuth, elevation)
- With projection: `P_{f,s}(V_{th,ph}(...))`
- With distortion: `D_{k1,k2}(P_{f}(...))`

Each operation and lift has a formula symbol (e.g., `z^n`, `(az+b)/(cz+d)`, `e^z`, `ln|J|`, `S²`).

### 12.2 DOF Counting

Structural DOF = number of controllable parameters. Temporal DOF = number of independently animated parameters. Base manifold contributes 2 structural (the 2D lattice).

For each active layer: structural += (number of params + 1 for blend). For each animated param: temporal += 1. For each param with active FM modulator (depth > 0): temporal += 1 additional.

---

## 13. Preset Format

JSON schema v1. Complete instrument state serialized as:

```json
{
    "schema": "polyform.preset",
    "version": 1,
    "metadata": { "name", "description", "author", "created", "modified", "tags" },
    "tempo": Param,
    "defaults": { "blendTime": 1.0 },
    "palette": "phosphor",
    "pipeline": {
        "operations": [Layer, ...],
        "lift": [Layer, ...]
    },
    "camera": {
        "position": { "orbitX": ToggleParam, "orbitY": ToggleParam, ... },
        "lens": { "projection": "perspective", "focalLength": ToggleParam, ... },
        "distortion": { "k1": ToggleParam, "k2": ToggleParam }
    },
    "quaternion": {
        "on": bool,
        "mode": "fixed"|"drift",
        "angle": Param,
        "ax": Param, "ay": Param, "az": Param,
        "driftSpeed": 0.1
    },
    "sampling": { "scale": ToggleParam, "density": ToggleParam },
    "display": { "showFormula": bool, ... }
}
```

Where `Param = { value, min, max, anim, shape, ratio, offset, modulator? }` and `ToggleParam = { on, ...Param }` (flattened via serde).

Layer = `{ id: UUID, type: registry_key, enabled: bool, blend: Param, params: { key: Param } }`.

All JSON keys are camelCase (enforced by `serde(rename_all = "camelCase")`). Unknown fields are silently ignored on import. Missing fields use registry defaults. The preset format is the portable asset across all platform phases.

Tolerant import: unknown operation/lift types are dropped with warnings. Missing params are filled from registry definitions. Extra params are silently ignored.

---

## 14. Crate Architecture

Two-crate Cargo workspace:

### `polyform-core` (library, no rendering dependencies)
- `types.rs` — Param, ToggleParam, ShapeFunction, PhaseModulator, ProjectionMode, QuaternionMode
- `preset.rs` — Preset, Camera, Quaternion, Sampling, Display (serde-based)
- `layer.rs` — Layer struct (id, type_key, enabled, blend, params)
- `ops.rs` — 25 operation functions with uniform `fn(x, y, params, x0, y0) -> (x, y)` signature
- `lift.rs` — 15 lift functions with uniform `fn(x, y, params, x0, y0, j_signed) -> (dx, dy, dz)` signature
- `registry.rs` — OperationDef/LiftDef with fn pointers, param metadata, categories
- `pipeline.rs` — Pipeline composition, pre-flatten optimization, numerical Jacobian
- `animation.rs` — `resolve_param()`, `apply_shape()`, ParamPath, PipelineKind
- `engine.rs` — AnimationEngine: tick(), weight system, quaternion resolution, transitions
- `transition.rs` — TransitionManager with cosine-eased blends
- `frame_state.rs` — FrameState struct (engine output consumed by renderer)
- `notation.rs` — Composition string, formula symbols, DOF counting
- `preset_io.rs` — Tolerant import/export, registry reconciliation
- `palette.rs` — Palette definitions (5 palettes)

### `polyform-app` (binary, wgpu + egui)
- `main.rs` — App loop, action dispatch, overlay rendering
- `gpu.rs` — wgpu setup, compute/render pipelines, buffer management
- `camera.rs` — Camera matrix computation (orbit, projection, quaternion, lens correction)
- `ui.rs` — egui control panels, param widgets, layer management
- `shaders/compute.wgsl` — Grid generation + all operations + lifts on GPU
- `shaders/render.wgsl` — Vertex transform + Brown-Conrady distortion

---

## 15. Mathematical Properties

### Non-commutativity
Operation composition is non-commutative: `scale(rotate(L)) ≠ rotate(scale(L))`. This is visible and intentional — it's a core pedagogical feature.

### Holomorphic vs Non-holomorphic
The complex operations (zPow, zExp, zLog, zSin, zCos, mobius) are conformal (angle-preserving). The non-holomorphic operations (conjugate, reflect, abs, mixed) break conformality. Both families coexist in the pipeline.

### Jacobian as Musical Feature
The Jacobian determinant measures local area distortion from the operations pipeline. When used as a lift (zLogJac, zSignedJac), it creates height maps that reflect the mathematical intensity of the transformation — regions of high compression or expansion become visible as terrain. This is computed via finite differences, not analytically.

### Phase Relationships
The golden ratio (φ = 1.618...) and its reciprocal (1/φ = 0.618...) appear as default ratios for quaternion axes. These produce maximally irrational frequency relationships — the axes never exactly repeat relative to each other, creating perpetual visual novelty. Polyrhythmic ratios throughout the system (arbitrary ratios between any two animated params) produce similar effects.

### Dimensional Counting
The dimension readout counts the total degrees of freedom in the active configuration: structural (controllable parameters) + temporal (independently animated parameters). This makes the mathematical complexity of the current state visible as a single number.

---

## 16. Capabilities Summary

What this instrument can produce:

- **2D conformal maps visualized in real time**: complex power, exponential, logarithmic, Mobius transforms on a visible grid
- **Non-linear deformations**: spirals, swirls, pinches, wave warps, all composable in arbitrary order
- **3D displacement from 2D**: height fields from magnitude, Jacobian, real/imaginary parts; wave patterns; spherical/helical/conical projections
- **Animated everything**: every scalar parameter can oscillate against the master phase with independent frequency ratios, FM modulation, and smooth shape functions
- **Performable camera**: orbit, focal length, anamorphic squeeze, lens distortion, quaternion tumble — all animatable, all smoothly toggable
- **Smooth transitions by construction**: layer toggle, anim toggle, mode switch, preset load — all weight-interpolated, no jumps
- **Live mathematical notation**: composition chain and DOF count displayed in real time
- **Deterministic rendering**: identical master phase + config = identical output, suitable for offline frame-by-frame export
- **GPU-accelerated**: all pipeline math runs on compute shader, ~8400 vertices at 60fps

The instrument makes higher-dimensional mathematical structure felt and performable. The 2D grid, named operations, live dimension readout, and signal flow notation are not implementation details — they are the interface through which mathematical thinking becomes spatial and temporal experience.
