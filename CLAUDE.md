# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Polyform is a **lattice synthesizer** — a real-time video synthesis instrument that performs mathematical transformations on a 2D grid. It belongs to the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits). The 2D grid is the identity-defining anchor; all extensions must honor the substrate.

The project's core purpose is making higher-dimensional mathematical thinking *felt and performable*, not just studied. The dimension readout, visible grid, and named mathematical operations are pedagogical features, not implementation details.

## File Inventory

| File | Purpose |
|------|---------|
| `web/polydimensional-coupling-v8.jsx` | Current component (v8 — extracted registries, ~4500 lines) |
| `web/main.jsx` | Entry point |
| `web/registries.js` | Transform, displacement, palette, and ratio-group registries |
| `web/preset-schema.ts` | Preset format types + validators (schema v1) |
| `web/preset-io.ts` | State <-> preset bridge |
| `web/registries.test.js` | Registry tests (vitest) |
| `web/preset-roundtrip.test.ts` | Preset roundtrip tests (vitest) |
| `src/lib.rs` | Rust crate root |
| `src/types.rs` | Core types: Param, ToggleParam, AnimMode, etc. |
| `src/layer.rs` | Pipeline layer type |
| `src/preset.rs` | Top-level Preset, Camera, Quaternion, Sampling, Display |
| `src/palette.rs` | Palette struct + static data |
| `src/registry.rs` | Registry types + operation/lift metadata |
| `Cargo.toml` | Rust crate config |
| `vite.config.js` | Vite config |
| `polyform-brief.md` | Project philosophy |
| `DEVLOG.md` | Development history |
| `ROADMAP.md` | Current priorities |
| `README.md` | Public-facing documentation |
| `archive/` | Legacy versions (v5, v6, v7) and old docs |

## Build & Run

```bash
npm install
npx vite
```

Open `http://localhost:5173` in a modern browser.

**Smart-quote check** (run before declaring any JSX file done):
```bash
python3 -c "src=open('file.jsx').read(); [print(f'found {repr(c)}') for c in ['\u2018','\u2019','\u201c','\u201d','\u2014','\u2013'] if c in src]"
```

## Architecture: Three-Stage Pipeline

**Stage 1 — Operations** (R2 -> R2): Ordered composition of 2D maps on the grid. Families: linear, complex (z^n, mobius, exp, log), polar, wave, non-holomorphic. Composition is non-commutative.

**Stage 2 — Lift** (R3 displacement): Sequential 3D displacement applied to the post-operation lattice. Families: height (magnitude, Jacobian, real/imag), wave (sine, gaussian), full-3D (spherical, conical, helical, ripple).

**Stage 3 — Camera**: Orbit position, lens (focal length, sensor, anamorphic, shift), radial distortion (Brown-Conrady k1/k2), quaternion tumble. The camera is a performable layer, not a passive viewer.

### Param Shape

Every animatable scalar uses the `Param` interface: `{ value, min, max, anim, animMode ("loop"|"sweep"), ratio, phaseOffset }`. `ToggleParam` extends it with an `on` boolean. Every layer has a `blend: Param` for smooth enable/disable transitions.

### v7 Component Internal Organization

- **Registries** (~lines 8-476): `TRANSFORMS`, `DISPLACEMENTS`, `PALETTES`, `RATIO_GROUPS`
- **Pipeline functions** (~530-700): `makeLayer`, `makeDispLayer`, `applyPipeline`, `applyPipelineFlat`, `applyDispPipeline`, `applyDispPipelineFlat`, `compositionString`
- **Three.js rendering** (~700-1400): Geometry buffer pool, camera, Jacobian computation, lens distortion
- **React component** (PolyformViz, ~1400+): State management, animation loop, UI
- **Key refs**: `animValuesRef` (swept values), `geometryPoolRef` (buffer pool), `stateRef` (merged state for render), `captureCanvasRef` (composite recording canvas)

### Key Systems (v7)

- **Geometry buffer pool** — pre-allocated Float32Arrays reused every frame, no GC pressure. Recreated only when density changes.
- **Animation decoupled from React** — `sweepAllParams()` writes to `animValuesRef`, React state synced at ~10fps for UI sliders.
- **Transition manager** — cosine-eased blends for toggle/preset transitions. Infrequent setState calls (structural events only).
- **Neutralize/Restore** — sequential pipeline teardown/rebuild with per-group timing. Bypass mechanism for recording.
- **Live recording** — composite canvas (WebGL + overlays) via `captureStream(0)` + manual `requestFrame()`. VP8 at 20Mbps.
- **Offline rendering** — deterministic clock (`1/fps` per frame), same composite canvas, rAF-based pacing. Includes neutralize/restore as bookends with 0.5s flat grid hold.

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
- **Layer identity.** Every layer has a stable UUID (`generateLayerId()`). Persists across save/load/reorder.

## Known Gotchas

- **Smart quotes break JSX.** Curly quotes instead of straight quotes will break the parser. Always verify with the script above.
- **Three.js r184.** No `OrbitControls` import, no `CapsuleGeometry`. Don't add either. Canvas drag for orbit is custom.
- **`updateMatrixWorld(true)`** must be called before reading `matrixWorld` in the render loop (lens distortion path).
- **Camera pole degeneracy.** Up-vector falls back to world Z when looking straight up/down.
- **NaN guard on ratios.** Always use `(p.ratio || 1)` — ratio can be undefined or 0 from older presets.
- **Deep copy phaseOffset.** When cloning param objects (e.g., for offline render), must also spread `phaseOffset` sub-object to prevent mutation of live state.
- **DOM-attached capture canvas.** The composite canvas for recording must be in the DOM (`document.body.appendChild`) for cross-browser `captureStream` compatibility. Hidden via fixed positioning off-screen.
- **Offline render codec choppiness.** VP8/WebM encoding can produce slightly choppy output even with correct frame timing. Known issue — likely needs codec change or image-sequence export.

## Display Overlays

Each overlay occupies a fixed canvas position regardless of which others are active:
- **Signal flow**: bottom-left
- **Active layers**: top-left
- **Dim count**: top-right
- **FPS**: bottom-right (separate toggle from dim count)

All overlays are captured in both live and offline recordings via the shared `drawOverlays()` function.

## Project Trajectory

**Phase 1** (current): Web prototype (React + three.js) for design validation.
**Phase 2** (next): Native Rust app (wgpu + egui) with SpoutCapture bridge for Resolume.
**Phase 3** (speculative): C++ rewrite with native Spout SDK integration.

The **preset format is the portable asset** across all three phases.

## When to Ask the User

Ask before deciding anything that touches:
- Dimension readout prominence or accuracy
- Smoothness of transitions in performance use
- UI vocabulary drifting back toward technical/engineering language
- Removing the 2D grid as the anchor

Use your own judgment for: code organization, easing functions, UI spacing/color, test coverage.
