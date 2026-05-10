# Polyform Roadmap

## Current State

v8 web prototype complete: registries extracted, preset schema canonical, 84 tests passing. Source restructured (`src/`, `archive/`). Web prototype is design-validated — moving to Rust port.

---

## Web Prototype (Parked)

These remain valid but are deferred until the Rust app is functional. Some (MIDI, recording quality) are better solved natively.

- [ ] Recording quality (codec artifacts — likely solved by native encoding)
- [ ] Preset morphing (crossfade via layer UUID matching)
- [ ] MIDI / OSC input
- [ ] Additional operation/lift families
- [ ] UI polish (undo/redo, param lock, collapsible panels)

---

## Phase 2 — Rust Port

Target: Native real-time performance instrument with video output routing.

### Phase 2.1: Scaffold + Core Types
- [ ] Cargo workspace with `polyform-core` library crate
- [ ] All core types: `Param`, `ToggleParam`, `Layer`, `Camera`, `Quaternion`, `Sampling`, `Display`, `Preset`
- [ ] Serde JSON round-trip matching existing preset schema exactly
- [ ] Unit tests: Rust Preset → JSON matches JS format

### Phase 2.2: Registries + Pipeline Math
- [ ] Port 30 operation functions (linear, complex, polar, wave, non-holomorphic)
- [ ] Port 15 lift functions (height, wave, full-3D)
- [ ] Port `applyPipeline`, `applyDispPipeline`, pre-flattened variants
- [ ] Port composition string, DOF counting
- [ ] Golden test vectors: generate from JS, verify Rust output within f64 tolerance

### Phase 2.3: Preset System
- [ ] Port validation (tolerant: unknown fields ignored, missing → defaults, type mismatch → warn)
- [ ] Port import/export with registry reconciliation (drop unknown types, fill missing params)
- [ ] Round-trip tests: load JS presets in Rust, re-export, diff

### Phase 2.4: Animation Engine
- [ ] Param sweep (loop/sweep modes, ratio-scaled, phaseOffset)
- [ ] Transition manager (cosine easing, per-param targets, time-based)
- [ ] Tempo clock (BPM-driven, deterministic mode for offline render)

### Phase 2.5: wgpu Rendering
- [ ] Grid geometry generation (density-controlled lattice)
- [ ] Pipeline execution (compute shader or CPU + vertex upload — evaluate both)
- [ ] Camera: orbit, perspective/orthographic, lens params, Brown-Conrady distortion
- [ ] Quaternion tumble (SO(3))
- [ ] Jacobian computation (finite differences)
- [ ] Buffer pool (pre-allocated, reused per frame)

### Phase 2.6: egui UI
- [ ] Window with wgpu surface
- [ ] Overlay system (signal flow, active layers, dim count, FPS — fixed positions)
- [ ] Control panels matching web UI (tempo, palette, scale/density, operations, lift, quat, camera)
- [ ] Layer add/remove/reorder with blend animation
- [ ] Preset save/load via native file dialogs

### Phase 2.7: Integration
- [ ] SpoutCapture bridge for Resolume / OBS
- [ ] Live + offline recording (native encoding)
- [ ] MIDI mapping (CC → param, note → layer toggle)

### Cross-Cutting Constraints
- **Preset JSON is the bridge.** Web presets must load in Rust and vice versa.
- **2D grid anchor, per-layer blend, smooth transitions, dim readout** — identity constraints, not negotiable.
- **Stable layer UUIDs** — preset morphing depends on ID matching across presets.

---

## Phase 3 — Production (Speculative)

- [ ] C++ rewrite with native Spout SDK
- [ ] VST3/CLAP plugin format for DAW hosting
- [ ] Hardware control surface design
- [ ] Multi-instance sync for multi-screen installations

---

## Non-Goals

These are explicitly out of scope:

- Particle systems or untethered generative visuals (grid is the anchor)
- Audio-reactive mode as primary feature (may add as optional input)
- Node-based / visual programming interface (the pipeline is the interface)
- 3D model import or mesh manipulation (this is a lattice instrument)
