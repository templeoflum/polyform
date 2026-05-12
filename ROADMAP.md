# Polyform Roadmap

## Current State

Rust app functional: wgpu rendering, egui UI, full animation engine with parameter motion protocol. 141 tests passing across polyform-core and polyform-app. Moving to integration phase (Spout, MIDI).

---

## Web Prototype (Parked)

These remain valid but are deferred. Some (MIDI, recording quality) are better solved natively.

- [ ] Recording quality (codec artifacts — likely solved by native encoding)
- [ ] Preset morphing (crossfade via layer UUID matching)
- [ ] Additional operation/lift families
- [ ] UI polish (undo/redo, param lock, collapsible panels)

---

## Phase 2 — Rust Port

Target: Native real-time performance instrument with video output routing.

### Phase 2.1: Scaffold + Core Types ✓
- [x] Cargo workspace with `polyform-core` library crate
- [x] All core types: `Param`, `ToggleParam`, `Layer`, `Camera`, `Quaternion`, `Sampling`, `Display`, `Preset`
- [x] Serde JSON round-trip matching existing preset schema exactly
- [x] Unit tests: Rust Preset → JSON matches JS format

### Phase 2.2: Registries + Pipeline Math ✓
- [x] Port 25 operation functions (linear, complex, polar, wave, non-holomorphic, utility)
- [x] Port 15 lift functions (height, wave, full-3D)
- [x] Port `applyPipeline`, `applyDispPipeline`, pre-flattened variants
- [x] Port composition string, DOF counting
- [x] Golden test vectors: generate from JS, verify Rust output within f64 tolerance

### Phase 2.3: Preset System ✓
- [x] Port validation (tolerant: unknown fields ignored, missing → defaults, type mismatch → warn)
- [x] Port import/export with registry reconciliation (drop unknown types, fill missing params)
- [x] Round-trip tests: load JS presets in Rust, re-export, diff

### Phase 2.4: Animation Engine ✓
- [x] Parameter motion protocol: master phase, per-param resolve, weighted contributors
- [x] Shape functions (cosine/sawtooth), FM modulator, phase offset
- [x] Transition manager (cosine easing, per-param targets, time-based)
- [x] Tempo clock (BPM-driven, deterministic mode for offline render)
- [x] Smooth anim on/off via weight transitions (no jumps)
- [x] Slider grab override (pause animation during drag, morph back on release)
- [x] Quaternion: full Param controls for angle + axes, NLERP mode transitions

### Phase 2.5: wgpu Rendering ✓
- [x] Grid geometry generation (density-controlled lattice)
- [x] Pipeline execution (compute shader with vertex upload)
- [x] Camera: orbit, perspective/orthographic, lens params, Brown-Conrady distortion (k1/k2 in shader)
- [x] Quaternion tumble (SO(3)) with SLERP-based blending
- [x] Jacobian computation (finite differences)
- [x] Buffer pool (pre-allocated at MAX_VERTICES, reused per frame)

### Phase 2.6: egui UI ✓
- [x] Window with wgpu surface
- [x] Overlay system (signal flow, active layers, dim count, FPS — fixed positions, keyboard toggles)
- [x] Control panels (tempo, palette, scale/density, operations, lift, quaternion, camera)
- [x] Layer add/remove/reorder (button-based) with blend animation
- [x] Preset save/load via native file dialogs (rfd)

### Phase 2.7: Integration
- [ ] SpoutCapture bridge for Resolume / OBS
- [ ] Live + offline recording (native encoding)
- [ ] MIDI mapping (CC → param, note → layer toggle)

### Preset Compatibility ✓
- [x] Clean break: old-format presets (animMode, phaseOffset, old quaternion) are not supported — start fresh
- [x] New preset format: shape, offset, modulator fields; full Param for quaternion angle/axes
- [x] Tolerant import still silently ignores unknown fields (old test fixtures still parse)
- [x] Round-trip test passes with new format

### Known Issues
- Quaternion drift↔fixed mode switch can produce a small rotational jump (NLERP transition; infrequent operation)

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
