# Polyform Roadmap

## Current State

v7 web prototype is functional: geometry buffer reuse, animation decoupled from React, transition manager, preset system, live + offline recording with overlay capture.

---

## Near-Term (Web Prototype)

### Recording Quality
- [ ] Investigate offline render choppiness (likely VP8/WebM encoding artifacts at current bitrate)
- [ ] Try alternative codecs (H.264 via MediaRecorder where supported)
- [ ] Experiment with higher bitrate or variable bitrate encoding
- [ ] Consider frame-to-image-sequence export as lossless alternative

### Preset System
- [ ] Preset morphing (crossfade between two presets using layer UUIDs for alignment)
- [ ] Preset library browser (thumbnails, categories)
- [ ] Preset versioning (migration from v1 to future schema versions)

### Input
- [ ] MIDI mapping (CC to any param, note triggers for layer toggles)
- [ ] OSC input for network control
- [ ] Keyboard shortcuts for performance (layer toggles, neutralize, record)

### Pipeline Extensions
- [ ] Additional operation families (fractal, noise-based)
- [ ] Additional lift families (terrain, attractor-based)
- [ ] Layer groups / sub-compositions

### UI
- [ ] Collapsible panel sections
- [ ] Param lock (exclude from preset load / neutralize)
- [ ] Undo/redo for structural changes

---

## Phase 2 — Native App (Rust + wgpu + egui)

Target: Real-time performance instrument with video output routing.

### Core
- [ ] Port pipeline math to Rust (operations, lift, camera)
- [ ] wgpu compute shader for grid evaluation (parallel per-point)
- [ ] egui UI matching current web controls
- [ ] Preset format compatibility (same JSON schema, Rust serde)

### Video Output
- [ ] SpoutCapture bridge for Resolume / VDMX / OBS
- [ ] NDI output as alternative
- [ ] Hardware-accelerated encoding for recording

### Performance
- [ ] GPU-side geometry (vertex shader displacement)
- [ ] Multi-resolution grid (adaptive density near camera)
- [ ] 120fps+ target for VJ use

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
