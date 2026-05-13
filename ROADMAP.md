# Polyform Roadmap

## Current State

Rust app fully functional through Phase 2.13: wgpu rendering, egui Performance UI, full animation engine with parameter motion protocol. 169 tests passing. 32 operations, 18 lifts. Preset system with library, section presets, and cosine-eased morphing between presets. 15 factory presets. BPM timing corrected to bar-rate convention. Spout texture sharing (Windows). Viewport resolution decoupled from window size. Five lattice types with smooth morphing. Full Brown-Conrady distortion (k1-k4, p1-p2, decoupled center, squeeze). Decoupled operation centers. Phosphor rendering system (beam quads, HDR persistence, bloom, color grading). Visualization overlays (domain coloring, curl, conformal defect, normal shading, caustic). Performance UI redesign complete (compact widgets, popup system, responsive columns).

---

## Web Prototype (Parked)

These remain valid but are deferred. Some (MIDI, recording quality) are better solved natively.

- [ ] Recording quality (codec artifacts — likely solved by native encoding)
- [ ] UI polish (undo/redo, param lock)

---

## Phase 2 — Rust Port

Target: Native real-time performance instrument with video output routing.

### Phase 2.1: Scaffold + Core Types ✓
- [x] Cargo workspace with `polyform-core` library crate
- [x] All core types: `Param`, `ToggleParam`, `Layer`, `Camera`, `Quaternion`, `Sampling`, `Preset`
- [x] Serde JSON round-trip matching existing preset schema exactly
- [x] Unit tests: Rust Preset → JSON matches JS format

### Phase 2.2: Registries + Pipeline Math ✓
- [x] Port 32 operation functions (linear, complex, polar, wave, non-holomorphic, utility)
- [x] Port 18 lift functions (height, wave, full-3D)
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

### Phase 2.7: Spout Texture Sharing ✓
- [x] SpoutDX FFI bridge (Windows-only extern "C" wrapper)
- [x] GPU readback: texture-to-buffer copy with row alignment
- [x] UI toggle: enable/disable Spout, include/exclude overlays

### Phase 2.8: Viewport Resolution ✓
- [x] `Viewport { width, height }` (default 1920x1080)
- [x] Offscreen render target (RENDER_ATTACHMENT + TEXTURE_BINDING + COPY_SRC)
- [x] Camera aspect ratio derived from viewport, not window
- [x] Blit pass: fullscreen triangle shader, letterboxed viewport rect
- [x] Spout reads from offscreen texture at native viewport resolution
- [x] UI: DragValue width/height + preset buttons (HD/4K)

### Phase 2.9: Lattice Family ✓
- [x] Lattice type selector: Cartesian, polar, hexagonal, log-polar, bipolar
- [x] Per-type parameters (spoke count, inner radius, focus distance, tau count)
- [x] CPU grid generation for each lattice type (compute shader is lattice-agnostic)
- [x] Hex grid: 3 line families at 0°/60°/120° clipped to hexagonal boundary with explicit outline
- [x] Log-polar: logarithmic vertex spacing on spokes matching ring distribution
- [x] Bipolar: exact σ/τ curve formulas with dual foci
- [x] Smooth vertex morphing transitions between lattice types (cosine-eased, 0.4s)
- [x] Tempo uncapped to 1,000,000 BPM with logarithmic slider, manual entry, grouped hot cues

### Phase 2.10: Compositional Depth ✓
- [x] Decoupled operation centers — `cx, cy` shift params for conformal/radial ops (movable poles/branch points)
- [x] Decoupled distortion center — separate Brown-Conrady center from NDC origin
- [x] k3, k4 higher radial distortion terms
- [x] Tangential distortion p1, p2 (complete Brown-Conrady model)
- [x] Anamorphic squeeze (per-axis distortion scaling)
- [x] Caustic curve overlay — visualize det J = 0 (fold/cusp curves)

### Phase 2.12: Phosphor — Rendering Character ✓
- [x] Beam quad rendering (configurable width, intensity)
- [x] HDR persistence buffer (configurable decay, blend mix)
- [x] Bloom post-process (Gaussian blur with threshold)
- [x] Color grading (gain, saturation, gamma)
- [x] All params as ToggleParam with full animation support

### Phase 2.12b: Visualization Overlays ✓
- [x] Domain coloring (phase/magnitude color mapping)
- [x] Curl magnitude visualization
- [x] Conformal defect overlay
- [x] Normal shading for lifted geometry
- [x] Caustic overlay moved to visualization system

### Performance UI Redesign ✓
- [x] Compact param/toggle widgets (single-line + popup for detail)
- [x] Popup system for ratio presets, BPM presets, param detail
- [x] Responsive 3-5 column layout
- [x] Top strip: tempo + preset + palette + viewport + spout + overlays
- [x] Section-with-toggle (collapsible sections with group mute)
- [x] Visualization overlay toggles in top strip

### Phase 2.11: Preset System Redesign ✓
- [x] PresetFile wrapper type (full + section presets with `kind` discriminator)
- [x] Preset library directory (`~/.polyform/presets/` with subdirectories per section)
- [x] Section presets (save/load individual sections: operations, lift, camera, phosphor, sampling)
- [x] Inline dropdown preset loading (replaced blocking file dialogs)
- [x] Crossfade morphing between presets via layer UUID matching
- [x] Animation disabled during morph for smooth transitions
- [x] UI: morph duration control in top strip (default 3.0s, adjustable 0.1-30s)

### Phase 2.13: Expanded Registries + Timing ✓
- [x] 7 new operations: wrap, quantize, clamp, joukowski, zTan, kaleidoscope, noise (25 → 32)
- [x] 3 new lifts: zSaddle, zTerrace, toroidal (15 → 18)
- [x] Fixed mixed op CPU/GPU discrepancy
- [x] 15 factory presets across 5 categories
- [x] BPM timing corrected: bar-rate convention (Hz = BPM / 240, ratio=1 = one oscillation per bar)
- [x] Toggle blend time (0.3s) separated from morph duration
- [x] Joukowski smooth origin fade (no singularity spike)
- [x] Camera orbitY safety (min 0.15 to avoid pole degeneracy)

### Phase 2.14: Integration (Next)
- [ ] MIDI mapping (CC → param, note → layer toggle)
- [ ] Live + offline recording (native encoding)

### Preset Compatibility ✓
- [x] Clean break: old-format presets (animMode, phaseOffset, old quaternion) are not supported — start fresh
- [x] New preset format: shape, offset, modulator fields; full Param for quaternion angle/axes
- [x] Tolerant import still silently ignores unknown fields (old test fixtures still parse)
- [x] Round-trip test passes with new format

### Known Issues
- Quaternion drift/fixed mode switch can produce a small rotational jump (NLERP transition; infrequent operation)
- Spout requires the SpoutDX native library; build skips Spout on non-Windows platforms
- Offscreen readback for Spout uses a raw pointer split-borrow (`readback_offscreen`) — safe in practice since `readback_texture` does not touch `offscreen_texture`, but uses `unsafe`

### Cross-Cutting Constraints
- **Preset JSON is the portable asset.** Must work across all phases.
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
