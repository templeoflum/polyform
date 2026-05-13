# Polyform Development Log

## v5 — Original Prototype

The starting point. A single monolithic React + three.js component (~2160 lines) that established the core instrument:

- Three-stage pipeline: operations (R2->R2), lift (R3 displacement), camera
- Transform registries: linear, complex (z^n, mobius, exp, log), polar, wave, non-holomorphic
- Displacement registries: height, wave, full-3D (spherical, conical, helical, ripple)
- Full camera system: orbit, lens, Brown-Conrady distortion, quaternion tumble
- Per-parameter animation with loop/sweep modes and tempo-synced ratios
- Live dimension readout counting structural + temporal DOFs
- Palette system with multiple color schemes
- Direct canvas drag for orbit control

Known issues at this stage:
- No preset save/load
- No per-layer blend (layers toggle instantly)
- Animation loop calls `setState` every frame, causing full React re-renders at 60fps
- No smooth transitions for indirect state changes

## v6 — Preset System + Smooth Transitions

Major additions building on the v5 foundation:

### Preset System
- Designed `preset-schema.ts` with versioned JSON schema (v1)
- Built `preset-io.ts` bridging React state and the portable format
- Save/load via OS file dialogs
- Stable layer UUIDs (`generateLayerId()`) persisting across save/load/reorder
- Graceful forward compatibility: unknown fields ignored, missing fields defaulted, type mismatches warned

### Per-Layer Blend
- Every layer gained a `blend: Param` (0 = identity, 1 = fully active)
- `applyPipeline` mixes each layer's contribution by its blend value
- Toggling a layer animates blend over configurable time (default 0.5s)
- Blend is itself an animatable param with ratio

### Transition Manager
- List-based system: `{ paramRef, fromValue, toValue, startTime, duration, onComplete }`
- Cosine easing for smooth indirect state changes
- Handles layer toggle, preset load, reorder (fade-out, swap, fade-in)
- Direct manipulation (slider drag) sets value immediately, bypassing transitions

### Neutralize / Restore
- Sequential pipeline teardown: distortion -> lens -> position -> quat -> lift -> operations
- Sequential rebuild in forward order
- Each group toggles at a staggered delay (500ms between groups)
- Neutralize snapshot captures which layers were on for accurate restore
- `savedValuesRef` preserves animated parameter values across the cycle
- Bypass mechanism (`neutralizeBypassRef`) allows toggle calls during neutralize state

### Vocabulary Update
Applied the decided renames throughout the UI:
- "transform pipeline" -> "operations"
- "displacement pipeline" -> "lift"
- "input scale" -> "lattice scale"
- "grid density" -> "lattice density"
- "composition" -> "signal flow"

### Recording
- Live video capture via `captureStream` + `MediaRecorder`
- Record flow: neutralize -> start recording at neutral -> restore (captured) -> content -> neutralize (captured) -> stop at neutral -> restore

## v7 — Performance Pass + Offline Renderer

The v6 prototype had two performance problems: 18+ `setState` calls per animation frame causing React re-renders, and complete three.js geometry destruction/recreation every frame. v7 fixes the rendering pipeline and adds a deterministic offline renderer.

### Step 1: Geometry Buffer Reuse

**Problem**: `rebuildGeometry()` disposed all line children and created new `BufferGeometry` + `Float32BufferAttribute` + `Line` objects every frame. At density 27: 56 lines x 2 (sharp+glow) = 112 geometry objects created and destroyed per frame.

**Fix**: Added `geometryPoolRef`. Pre-allocate all geometries and Float32Array buffers once (or when density changes). Each frame writes vertex positions directly into existing buffers with `needsUpdate = true`. Shared materials updated in-place on palette change.

### Step 2: Reduce Pipeline Allocations

**Problem**: `applyPipeline` allocated a `flat = {}` object per active layer per grid point (~4,500 points x layers per frame).

**Fix**: Pre-flatten layer params once per frame at the top of `rebuildGeometry`. Added `applyPipelineFlat` and `applyDispPipelineFlat` variants accepting pre-flattened params. Pre-allocated reusable THREE.Vector3/Matrix4 temps on `sceneRef.current`.

### Step 3: Decouple Animation from React

**Problem**: The RAF tick called `setOrbitX`, `setLayers`, etc. 18+ times per frame, forcing full React re-render at 60fps.

**Fix**:
- `animValuesRef` holds swept values outside React
- `sweepAllParams()` extracted from inline sweep logic — reads structural state from `stateRef.current`, writes swept values to `animValuesRef`
- `stateRef.current` assembly merges `animValuesRef` values over base React state for the render path
- Throttled UI sync (~10fps) copies swept values back to React state for slider display
- UI sync uses updater functions that only merge swept params, preserving `on`/`blend` managed by the transition manager

### Step 4: Offline Renderer

Deterministic frame-by-frame rendering that produces correct output regardless of machine speed.

**Architecture**: Bypasses React entirely during rendering. Takes a deep mutable copy of the neutralized state (`offState`), temporarily swaps `stateRef.current` for each frame, renders, composites overlays onto a capture canvas, captures via `captureStream(0)` + `requestFrame()`.

**Render sequence**:
1. Real-time neutralize (user watches)
2. Start capture at flat grid, hold 0.5s
3. Offline restore: blend ops on (sequential), blend disps on (sequential), quat ramp, position/lens/distort groups ramp simultaneously
4. Content: advance clock by 1/fps per frame, sweep all animated params, render
5. Offline neutralize: reverse of restore
6. Hold flat grid 0.5s, stop capture
7. Real-time restore (user watches)

**Key decisions**:
- View param groups (position, lens, distort) ramp all params simultaneously within a group, matching live behavior
- Lattice anim restored after structural rebuild, frozen before final frame with initial values restored for clean looping
- Deep copy includes phaseOffset sub-objects to prevent mutation of live state
- NaN guards on all ratio values (`p.ratio || 1`)
- Try/catch with guaranteed cleanup (recorder stop + live restore always runs)
- Abort mechanism: clicking the render button during offline render sets `offlineActiveRef.current = false`

### Neutralize/Restore Bug Fixes (v7 iteration)

Several rounds of debugging the neutralize/restore system for both live and recording use:

**UI sync overwrites transition results**: `setLayers(av._layers)` was clobbering the transition manager's `onComplete` callback setting `on: false`. Fixed by using updater functions that only merge swept params, preserving `on`/`blend` from React state.

**Stale transition start values**: Transition captured `p.value` from React state, but the visual value was the swept value in `animValuesRef`. Fixed toggle functions to read swept values from `animValuesRef.current`.

**Sweep phase discontinuity on restore**: When `anim` restored to true, sweep computed from `animClock` which had advanced during the transition. Added 2-second resync blend mechanism in `sweepParam` — cosine-eases from the transition endpoint to the natural sweep position.

**Live recording stutter**: `captureStream(60)` + VP9 encoding overhead + UI sync re-renders. Fixed: 30fps capture, VP8 codec preference, 20Mbps bitrate, skip UI sync during recording.

### Recording with Overlays

Both live and offline recorders capture from a composite 2D canvas rather than the raw WebGL canvas. A shared `drawOverlays()` function renders whichever display overlays are active (signal flow, active layers, dim count, FPS) onto the composite after each Three.js render. The capture canvas is DOM-attached (hidden) for cross-browser `captureStream` compatibility.

### Display Layout

Each overlay occupies a fixed canvas position regardless of which others are active:
- Signal flow: bottom-left
- Active layers: top-left
- Dim count: top-right
- FPS: bottom-right (separate toggle from dim count)

## v8 -- Pre-Port Cleanup

Structural prep for the Rust port. No runtime behavior changes.

### Registry extraction

Moved all registries (`TRANSFORMS`, `DISPLACEMENTS`, `PALETTES`, `RATIO_GROUPS`, etc.) and pipeline math (`applyPipeline`, `compositionString`, `countDOFs`, etc.) into `registries.js` -- a pure JS module with no React/Three.js/browser dependencies. v8 imports from it. v7 preserved untouched as the known-good reference. ~850 lines removed from the component file.

### Preset system reconciliation

`preset-schema.ts` and `preset-io.ts` are now the canonical spec for the Rust port. Fixed divergences from v7's inlined copy: `Display` interface updated to 5 fields (`showFormula`, `showDimCount`, `showActiveLayers`, `showParamValues`, `showFps`), camera `orbitY` default corrected to 0.1, `cloneToggleParam`/`cloneBlend` fixed to deep-clone `phaseOffset`. Preset key vocabulary decision documented: serialized keys stay descriptive (`pipeline.operations`, `pipeline.lift`), stylized names live in UI only.

### Test coverage

Added vitest with 84 tests covering individual transform/displacement math, pipeline composition, blend interpolation, non-commutativity, pre-flattened equivalence, numerical Jacobian determinants, composition string generation, DOF counting, layer IDs, registry structure, and preset round-trip (including all 5 Display fields and forward compatibility).

### Recording quality -- known limitations (not fixing before port)

Observed during v8 validation at 60fps baseline:

- **Live recording**: FPS drops from 60 to 35-45. Expected -- compositing to the capture canvas and VP8 encoding at 20Mbps in real-time is genuinely expensive in a browser. The `captureStream` + `MediaRecorder` path has inherent overhead that cannot be eliminated in JS.
- **Offline recording**: Output is choppy with poor visual quality despite the FPS counter reading a steady 60. The deterministic clock is working correctly (frames are produced at exact 1/fps intervals), but VP8/WebM encoding produces artifacts. This is a codec limitation, not a timing bug.

Both issues are moot for the Rust port: Phase 2 uses wgpu + native encoding (or image-sequence export) with SpoutCapture for Resolume. The entire capture/encoding pipeline will be replaced. No value in debugging browser MediaRecorder quirks that won't transfer.

---

## Phase 2 — Rust Port

### Phase 2.1: Scaffold + Core Types

Cargo workspace with two crates: `polyform-core` (library, all math and types) and `polyform-app` (binary, wgpu + egui). Core types ported: `Param`, `ToggleParam`, `Layer`, `Preset`, `Camera`, `Quaternion`, `Sampling`, `Display`. Serde round-trip validated against the JS preset format.

### Phase 2.2: Registries + Pipeline Math

Ported all 25 operation functions and 15 lift functions with uniform signatures (`fn(x, y, params) -> (x, y)` for ops, `fn(x, y, z, params) -> (x, y, z)` for lifts). Pipeline application, pre-flatten optimization, and numerical Jacobian all ported. Composition string and DOF counting ported. Golden test vectors generated from JS and validated in Rust within f64 tolerance.

### Phase 2.3: Preset System

Tolerant validation ported: unknown fields silently ignored, missing fields filled from registry defaults, type mismatches warned. Registry reconciliation drops unknown operation/lift types and fills missing params from definitions. Round-trip tests confirm Rust import/export matches JS format.

### Phase 2.4: Animation Engine — Parameter Motion Protocol

Replaced the web prototype's clock-accumulator architecture with a master-phase system. The old approach used an unbounded `animClock`, per-param `LoopState` accumulators, and a separate transition overlay that frequently fought with the sweep system. Every blend/toggle bug in the web version traced back to this fundamental split.

**New architecture**: A single master phase `M` advances by `dt * tempo` each tick (unbounded, wrapping handled per-param). Each animated param computes its value as a pure function: `phi = (M * ratio + offset) mod 1.0`, shaped by cosine or sawtooth, then scaled to `[min, max]`. Optional FM modulation via `PhaseModulator` (ratio, depth, shape). Animation on/off is a smooth weight transition (0.0-1.0) blending between base value and animated value.

**Key properties**:
- Deterministic: same M + config = same output, always
- No accumulators to get out of sync
- Transitions are just weight interpolation — no fighting between sweep and transition systems
- Shape functions: `Cosine` (ping-pong, smooth, default for all params) and `Sawtooth` (linear ramp, wrapping)

**Quaternion system**: Full `Param` controls for angle and all three axes (replacing the old bare-float structure). SLERP-based blending for on/off transitions using angle scaling. NLERP for drift/fixed mode transitions. Slider grab override pauses animation during drag, morphs back on release.

**Preset format clean break**: Old fields (`animMode`, `phaseOffset`, old quaternion `angle`/`angleRatio`/`axes` structure) are no longer supported. New format uses `shape`, `offset`, `modulator` on Param, and full Param objects for quaternion angle/axes. All backwards compatibility code removed.

### Phase 2.5: wgpu Rendering

Grid geometry generated on GPU via compute shader. Pipeline operations and lift applied in the compute pass, results uploaded to vertex buffers. Pre-allocated buffer pool at MAX_VERTICES capacity, reused every frame. Camera system: orbit, perspective/orthographic projection, focal length, sensor size, anamorphic squeeze, shift, Brown-Conrady radial distortion (k1/k2 applied in the vertex shader). Quaternion tumble via `glam::Quat` with SLERP-based blending. Numerical Jacobian via finite differences for height-field lifts.

### Phase 2.6: egui UI

Full control panel matching the web prototype: tempo, palette, lattice scale/density, operation layers, lift layers, quaternion, camera position, camera lens, distortion. Layer add/remove/reorder (button-based) with blend animation. Overlay system at fixed screen positions (signal flow bottom-left, active layers top-left, dim count top-right, FPS bottom-right) with keyboard toggles (S/A/D/F). Preset save/load via native file dialogs (`rfd` crate). Animated slider positions track resolved param values in real time.

### Phase 2.7: Spout Texture Sharing

Windows-only Spout sender via SpoutDX FFI bridge. `SpoutSender` wraps extern "C" calls to `spout_sender_create/open/send_image/close/destroy`. GPU readback via `readback_texture()` copies the rendered texture to a CPU-side buffer (row-aligned to `COPY_BYTES_PER_ROW_ALIGNMENT`, padding stripped). UI toggle in control panel: "Spout: Send" checkbox with optional "Include overlays" for choosing pre- or post-overlay readback.

### Phase 2.8: Viewport Resolution — Camera-Owned Render Target

Decoupled render resolution from window size. Previously the lattice rendered directly to the window surface, so resizing the window changed the camera's perspective arbitrarily.

**Architecture**: `Viewport { width, height }` added to `Preset` (default 1920x1080). The renderer creates an offscreen `wgpu::Texture` at viewport dimensions (`RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC`). The lattice renders to this offscreen texture; camera aspect ratio derives from viewport dimensions. A blit pass samples the offscreen texture via a fullscreen triangle shader and composites it onto the window surface with letterboxing. The blit viewport rect is computed by `letterbox_viewport()` to preserve aspect ratio with black bars.

**Key properties**:
- Resizing the window only changes the preview scale — lattice aspect ratio stays locked
- Viewport is a structural preset property, not animatable
- Spout reads from the offscreen texture at native viewport resolution (pixel-perfect output)
- `ensure_offscreen()` lazily creates/resizes the offscreen texture on dimension change
- UI: DragValue fields for width/height (range 320-7680) + preset buttons (HD 1920x1080, 2K 2560x1440, 4K 3840x2160)

### Phase 2.9: Lattice Family

Five lattice substrate types, each with per-type parameters:

- **Cartesian** — the original rectangular grid (no extra params)
- **Polar** — concentric rings + radial spokes (spoke count)
- **Hexagonal** — three line families at 0°/60°/120° clipped to hexagonal boundary with explicit outline
- **Log-polar** — logarithmic vertex spacing on spokes matching ring distribution (spoke count, inner radius)
- **Bipolar** — exact σ/τ curve formulas with dual foci (focus distance, tau count)

CPU grid generation for each type (compute shader remains lattice-agnostic). Smooth vertex morphing transitions between lattice types (cosine-eased, 0.4s). UI: type selector buttons + per-type controls in the lattice section.

Tempo uncapped to 1,000,000 BPM with logarithmic slider, manual entry via DragValue, and grouped BPM presets (slow/musical/fast/raster) accessible via popup.

### Phase 2.10: Compositional Depth

Two major expansions:

**Decoupled operation centers**: `cx, cy` shift params added to conformal/radial operations (zPow, zInv, zExp, zLog, zSin, zCos, mobius, spiral, pinch, swirl, radialSine, angleWarp, ripple). Implementation: translate by `(-cx, -cy)` before applying, translate back after. Standard `Param` objects — animatable, modulatable.

**Extended distortion model**: Full Brown-Conrady implementation:
- k3, k4 higher radial terms (`1 + k1*r^2 + k2*r^4 + k3*r^6 + k4*r^8`)
- Tangential distortion p1, p2 (`dx = 2*p1*x*y + p2*(r^2 + 2*x^2)`)
- Decoupled distortion center (center_x, center_y — separate from NDC origin)
- Anamorphic squeeze (per-axis distortion scaling)
- All new params as `ToggleParam` with animation support

**Caustic overlay**: Visualization of `det J = 0` fold/cusp curves. Threshold `|det J|` below epsilon, render affected vertices with distinct color. Moved from pipeline overlay to the visualization overlay system.

### Phase 2.12: Phosphor — Rendering Character

Complete rendering character overhaul replacing flat-color wireframe with a phosphor display emulation:

**Beam rendering**: Lines replaced with oriented quads (beam width, beam intensity). Quad geometry generated in the vertex shader from line endpoints + camera-facing expansion. Allows variable width and intensity falloff.

**HDR persistence**: Off-screen persistence buffer accumulates previous frames with configurable decay rate. New frame blends with persistence buffer using blend_mix parameter. Enables CRT-like afterglow trails.

**Bloom**: Gaussian blur post-process with configurable intensity, radius, and threshold. Applied to the composited (current + persistence) frame in a separate render pass.

**Color grading**: Gain (overall brightness), saturation (color intensity), gamma (tone curve). Applied as final pass before output.

All phosphor parameters are `ToggleParam` objects with full animation support (value, min, max, anim, shape, ratio, offset, modulator).

### Phase 2.12b: Visualization Overlays

GPU-computed diagnostic overlays rendered alongside the lattice:

- **Domain coloring** — phase + magnitude visualization of the complex mapping (HSL color wheel for argument, brightness for modulus)
- **Curl** — curl magnitude of the 2D vector field (red = positive, blue = negative)
- **Conformal defect** — deviation from angle-preserving mapping (highlights where the transformation breaks conformality)
- **Normal shading** — surface normal visualization for lifted 3D geometry (Lambertian shading against a fixed light direction)

Each overlay is a toggle in the top strip's "Vis:" row. The caustic overlay (from 2.10) was moved into this system. Overlays are computed in the fragment shader using the Jacobian and surface data already available from the pipeline.

### Performance UI Redesign

Complete UI architecture rewrite for dense, performance-oriented layout:

**Compact widgets**: Replaced the old `param_widget`/`toggle_param_widget` (multi-line, always-visible controls) with `compact_param_widget`/`compact_toggle_param_widget` (single-line slider + anim checkbox + ratio badge). Detail controls (shape, ratio presets, min/max, offset, FM modulator) moved to popup windows accessible via `...` button.

**Popup system**: `OpenPopups` state stored in egui's type-map. Toggleable windows for param detail, BPM presets, and ratio presets. Windows close on X button or by clicking the `...` button again.

**Responsive column layout**: 3-5 columns based on window width (min 500px per column). At 5 columns, camera splits into Position+Lens / Distortion+Quaternion.

**Top strip**: Two-row layout:
1. Tempo (BPM drag + log slider + popup), preset name/save/load, palette combo, viewport size + HD/4K buttons, Spout toggle
2. Overlay checkboxes (Signal/Dim/Layers/FPS) + visualization overlays (Caustic/Domain/Curl/Conformal/Normals)

**`section_with_toggle`**: Collapsible section header with group-mute checkbox. Clicking the checkbox toggles all params in the section. Used for Position, Lens, Distortion, Phosphor.

**Layer rows**: Collapsible layer headers with enable checkbox, expand arrow, type label, and inline blend slider. Expanded content shows type combo, full blend widget, per-param widgets, and move/remove buttons.

### Phase 2.13: Expanded Registries + Preset System

Three-part expansion: new operations/lifts, preset infrastructure, and timing corrections.

**7 new operations** (25 -> 32 total):
- **Utility**: `wrap` (modular tiling), `quantize` (grid snapping), `clamp` (radial containment)
- **Complex**: `joukowski` (z + a/z, conformal airfoil shapes with smooth origin fade), `zTan` (complex tangent with magnitude clamping)
- **Polar**: `kaleidoscope` (N-fold angular symmetry with mirror)
- **Wave**: `noise` (trig-based 2D displacement noise)

Each operation added in Rust (ops.rs), WGSL (compute.wgsl), registry (registry.rs), and GPU dispatch (gpu.rs) with unit tests.

**3 new lifts** (15 -> 18 total):
- **Height**: `zSaddle` (hyperbolic paraboloid: dz = amp * x * y), `zTerrace` (quantized staircase surface)
- **Full-3D**: `toroidal` (wrap grid onto torus surface)

**Fixed `mixed` op discrepancy**: WGSL was computing z^2 (redundant with zPow n=2) instead of the Rust mixed-monomial map (x^2*y^3, -x^3*y^2).

**Preset system**:
- `PresetFile` wrapper type with `kind` discriminator (Full, Operations, Lift, Camera, Phosphor, Sampling)
- `PresetKind` enum with `dir_name()` for library subdirectories
- Preset library at `~/.polyform/presets/` with subdirectories per section
- `install_factory_presets()` writes 15 built-in presets to library on startup
- Section presets: save/load individual sections via header buttons with inline ComboBox dropdowns
- Replaced blocking file dialogs with non-blocking egui ComboBox dropdowns for preset loading

**15 factory presets** across five categories:
- Foundations: conformal zoo, mobius dance, spiral galaxy, sine lattice, kaleidoscope
- Complex compositions: crystal, warp field, pixel drift, event horizon, reef
- Camera-focused: tunnel, flyover, lens
- Phosphor character: glow, strobe

**Preset morphing** (`morph.rs`):
- `MorphState`: source, target, duration, elapsed time with cosine easing
- `blend_presets()`: interpolates all params between source and target presets
- Matching layers (by UUID) interpolate all params; unmatched source layers fade out, unmatched target layers fade in
- Animation disabled during morph (`anim: false` in lerp_param) -- prevents rapid cycling from animation engine resolving mid-transition params
- Camera, quaternion, sampling, phosphor all interpolated with appropriate handling (continuous params lerp, discrete values snap at midpoint)
- Morph duration adjustable via UI (default 3.0s), displayed in top strip

**BPM timing fix**: Changed BPM-to-Hz conversion from `/60` to `/240`. This makes ratio=1 mean "one oscillation per bar (4 beats in 4/4)" -- standard musical LFO convention. At 120 BPM, ratio=1 gives a 2-second cycle (one bar). At 30 BPM, ratio=1 gives an 8-second cycle. Previously 30 BPM felt as fast as a 2-second oscillation.

**Toggle/morph time separation**: Individual param toggles, layer toggles, and mode switches now use a fixed 0.3s blend time (`TOGGLE_BLEND_TIME`), separate from the morph duration (`blend_time`, default 3.0s). Previously both used the same value, causing phosphor toggles to take 3 seconds to respond after the morph duration was increased.

**Joukowski singularity handling**: Replaced hard clamp (`max(r2, 0.01)`) with smooth fade -- joukowski effect linearly fades to identity below r=0.3, preventing the origin spike/line artifact in the crystal preset.

**Camera orbit safety**: Factory preset orbitY ranges changed from `min: 0` to `min: 0.15` to avoid the camera pole degeneracy (up-vector flip when looking straight down).

169 tests passing (165 core + 4 app).
