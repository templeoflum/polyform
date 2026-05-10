# Polyform Development Log

## v5 — Original Prototype

The starting point. A single monolithic React + three.js component (~2160 lines) that established the core instrument:

- Three-stage pipeline: operations (R2→R2), lift (R3 displacement), camera
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
- Sequential pipeline teardown: distortion → lens → position → quat → lift → operations
- Sequential rebuild in forward order
- Each group toggles at a staggered delay (500ms between groups)
- Neutralize snapshot captures which layers were on for accurate restore
- `savedValuesRef` preserves animated parameter values across the cycle
- Bypass mechanism (`neutralizeBypassRef`) allows toggle calls during neutralize state

### Vocabulary Update
Applied the decided renames throughout the UI:
- "transform pipeline" → "operations"
- "displacement pipeline" → "lift"
- "input scale" → "lattice scale"
- "grid density" → "lattice density"
- "composition" → "signal flow"

### Recording
- Live video capture via `captureStream` + `MediaRecorder`
- Record flow: neutralize → start recording at neutral → restore (captured) → content → neutralize (captured) → stop at neutral → restore

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
