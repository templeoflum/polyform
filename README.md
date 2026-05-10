# Polyform

A lattice synthesizer for real-time video synthesis. Polyform performs mathematical transformations on a 2D grid substrate, producing animated geometric imagery in the lineage of video synthesis (Rutt-Etra, LZX Industries, Sleepy Circuits).

The instrument makes higher-dimensional mathematical thinking felt and performable. The visible grid, named operations, and live dimension readout are pedagogical features — you develop geometric intuition through play, not study.

## Quick Start

```bash
npm install
npx vite
```

Open `http://localhost:5173` in a modern browser.

## What You See

A regular 2D grid, transformed in real time by a composable pipeline of mathematical operations. The grid is always visible — it's the anchor that makes the mathematics legible.

**Operations** (R2 → R2) warp the grid: z^n, mobius, exp/log, spiral, shear, wave. Composition is non-commutative — order matters.

**Lift** (R3 displacement) pulls the grid into three dimensions: sine waves, Jacobian height, spherical projection, helical winding.

**Camera** frames the result: orbit, focal length, anamorphic squeeze, lens distortion, quaternion tumble. The camera is a performable layer, not a passive viewer.

Everything animates against a global tempo with polyrhythmic ratios. The dimension readout shows the live count of structural and temporal degrees of freedom in the active configuration.

## Controls

- **Drag** the canvas to orbit the camera
- **Operations/Lift sections** — add, remove, reorder, toggle layers. Each layer has per-parameter animation controls (loop/sweep mode, ratio, phase offset) and a blend slider for smooth transitions.
- **Camera sections** — position, lens, distortion, quaternion tumble. Each parameter is independently animatable.
- **Neutralize/Restore** — sequentially peels all active layers back to the flat grid, then rebuilds them. Used as performance gesture and as bookends for video recording.
- **Record** — live video capture with neutralize/restore as intro/outro.
- **Offline Render** — deterministic frame-by-frame rendering for perfect output regardless of machine speed.
- **Presets** — save/load complete instrument state as JSON.

## Display Overlays

Toggle independently:
- **Signal flow** (bottom-left) — live formula showing the composition chain
- **Active layers** (top-left) — what's on, structural/temporal DOF per layer, animated param values
- **Dim count** (top-right) — total structural + temporal dimensions, system info
- **FPS** (bottom-right) — frame counter

All overlays are captured in both live and offline recordings when toggled on.

## Architecture

Single-file React + three.js component (`polydimensional-coupling-v7.jsx`, ~4900 lines). Vite dev server for HMR.

### Pipeline

```
2D lattice → Operations (R2→R2) → Lift (R3 displacement) → Camera → Screen
```

### Key Systems

- **Geometry buffer pool** — pre-allocated Float32Arrays reused every frame, no GC pressure
- **Animation decoupled from React** — swept values live in refs, React state synced at 10fps for UI
- **Transition manager** — cosine-eased blends for smooth toggle/preset transitions
- **Neutralize/Restore** — sequential pipeline teardown/rebuild with per-group timing

### Param Model

Every animatable scalar: `{ value, min, max, anim, animMode, ratio, phaseOffset }`. Toggle params add `on`. Layers add `blend`. Animation modes: `loop` (wrapping) and `sweep` (ping-pong cosine).

## Recording

**Live recording** captures the composite canvas (WebGL + overlays) via `captureStream(0)` with manual frame requests. VP8 encoding at 20Mbps.

**Offline rendering** drives the animation deterministically: advances the clock by `1/fps` per frame, renders, composites overlays, captures. The video starts and ends with a 0.5s hold on the flat grid, with neutralize/restore transitions as bookends for clean looping.

## Preset Format

JSON schema v1. Contains complete instrument state: pipeline structure, all parameter values/ranges/animation settings, camera, palette, tempo, display toggles. Layer IDs are stable UUIDs for future morphing support. See `preset-schema.ts` for types.

## Project Trajectory

| Phase | Stack | Status |
|-------|-------|--------|
| 1 — Web prototype | React + three.js | **Active** |
| 2 — Native app | Rust + wgpu + egui | Next |
| 3 — Production | C++ + Spout SDK | Speculative |

The preset format is the portable asset across all phases.

## File Inventory

| File | Purpose |
|------|---------|
| `polydimensional-coupling-v7.jsx` | Current component (v7 — perf + offline renderer) |
| `polydimensional-coupling-v6.jsx` | Previous version (preset system, blend, transitions) |
| `polydimensional-coupling-v5.jsx` | Original prototype |
| `main.jsx` | Entry point |
| `vite.config.js` | Vite config |
| `preset-schema.ts` | Preset format types + validators |
| `preset-io.ts` | State ↔ preset bridge |
| `polyform-brief.md` | Project philosophy |
| `DEVLOG.md` | Development history |
| `ROADMAP.md` | Current priorities |

## License

Not yet determined.
