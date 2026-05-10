# Polyform — Coding Session Handoff

## Context in one paragraph

Polyform is a lattice synthesizer — a video-synth-lineage instrument that performs mathematical transformations on a 2D grid in real time. See `polyform-brief.md` for the full project framing, including the philosophical intent (making higher-dimensional thinking accessible through felt intuition). This handoff is purely technical. The current state is a working web prototype (React + three.js) and a fresh preset format (TypeScript schema + IO). Next phase is a native Rust application, but before that, the React component needs to be updated to use the new preset format and new UI vocabulary.

## File inventory

**Web prototype (working):**
- `polydimensional-coupling-v5.jsx` — current React component. Standalone, single file, ~2160 lines. Renders into a three.js scene. Uses the old vocabulary throughout (transforms/displacements/view layers). Uses internal React state, no persistence.

**Preset system (new, not yet integrated):**
- `preset-schema.ts` — TypeScript types, defaults, validators. Schema version 1. Defines `Preset`, `Param`, `ToggleParam`, `Layer`, `Camera`, etc. Exports `validatePreset()` for graceful loading.
- `preset-io.ts` — Bridges between live React state (`AppState`) and the schema. Exports `exportPreset()`, `importPresetJSON()`, `downloadPreset()`, `pickAndLoadPreset()`. Uses a `Registry` interface to reconcile preset references against current build's known transform/lift types.
- `preset-roundtrip-test.ts` — smoke test, all checks pass. Useful as a usage example.

**Documentation:**
- `polyform-brief.md` — the project brief (audience: anyone who needs the full picture)
- `polyform-handoff.md` — this document

## Decisions already made (do not relitigate)

**Naming.** The instrument is **Polyform**. The category/practice is **lattice synthesis**. The 2D grid is the identity-defining anchor.

**Schema vocabulary.** Field names match UI vocabulary. `pipeline.operations` (was "transforms"), `pipeline.lift` (was "displacements"). Use these names in the React component too.

**UI vocabulary mapping** (apply throughout):
- `transform pipeline` → `operations`
- `displacement pipeline` → `lift`
- `view layers` → reorganized into camera sections
- `input scale` → `lattice scale`
- `grid density` → `lattice density`
- `composition` (live formula) → `signal flow`
- `tempo`, `ratio`, `loop`/`sweep`, `palette` — keep as-is
- `DOF` and `dim count` — keep as-is in the readout

**Layer identity.** Every layer carries a stable id (`generateLayerId()`). Persists across save/load/reorder. Used for morphing layer-matching later.

**Per-layer blend.** Every layer has a `blend: Param` and a `blendTimeOverride: number | null`. When a layer is enabled/disabled, blend animates between 0 and 1 over the configured blend time. This is the smooth-transition primitive — it also handles reorder bracketing and preset morphing.

**Reorder via fade.** Reordering a layer is implemented as: animate blend → 0, swap array position, animate blend → 1. Don't try to interpolate composition order itself.

**Tempo lives in the preset.** Don't pull it out as workspace state. Loading a preset crossfades the tempo too.

**Blend time.** Defaults: 0.5s for layer toggle, 1.0s for preset load, 0.3s × 2 for reorder bracketing. Units are seconds, with optional tempo-relative override (deferrable). Default `defaults.blendTime: 1.0` in preset.

**Smooth transitions: two paths.** Direct manipulation (slider drag, typed value) sets value immediately, no smoothing. Indirect changes (preset load, layer toggle, MIDI, reorder) go through a target/value pair where the live value eases toward the target. Each `Param` effectively has `value` (live) and an implicit `target` (where it's headed), with cosine easing over blend time.

**Trajectory.** Phase 1 (web prototype, current). Phase 2 (native Rust app + SpoutCapture bridge). Phase 3 (C++ rewrite, speculative). The preset format is the portable asset across all three.

## Decisions still open

These need a call when you encounter them, but should be small. Defer to the user (the human, not me) on anything load-bearing.

**Tempo-relative blend time.** We agreed on seconds-by-default with optional tempo-relative override. The override mechanism isn't designed yet. Defer until v6 stabilizes; revisit if it actually feels needed in performance.

**Preset library UI.** Save/load currently goes through OS file dialogs (`downloadPreset` / `pickAndLoadPreset`). A preset browser inside the app — bank, favorites, tagging — is on the roadmap but not v6. v6 just needs Save and Load buttons.

**Animation phase on preset load.** When loading a preset whose layers also exist in the current state (matched by id), should the layer's animation phase reset or continue? Current schema doesn't store animation phase, so on a fresh load it resets. For *morphing* between presets later, this matters more — but morphing isn't v6.

**Old-format migration.** v5 has no preset format. Nothing to migrate from. v6 is fresh start. Future versions get migration hooks in `validatePreset` (the version-mismatch warning is already there).

## Next deliverable: v6 of the React component

Take `polydimensional-coupling-v5.jsx` and produce `polydimensional-coupling-v6.jsx` with the following changes. This is roughly the order I'd tackle them, easiest to hardest.

### 1. Vocabulary update (mechanical, ~30 min)

Search-and-replace the UI labels per the mapping above. Field names in state can stay as `layers`/`dispLayers` internally for now if it reduces diff size, but consider renaming to `operations`/`lift` for consistency with the schema. The user-visible strings should all match the new vocabulary.

### 2. Wire in the preset system (~1-2 hours)

- Import `preset-schema.ts` and `preset-io.ts`
- Build a `Registry` object from the existing `TRANSFORMS` and `DISPLACEMENTS` constants. Each entry needs `key`, `params: ParamDef[]`. The mapping is straightforward.
- Add a function `getAppState()` that bundles all the relevant `useState` values into the `AppState` shape from `preset-io.ts`. This will get called when exporting.
- Add a function `applyAppState(state)` that takes a loaded `AppState` and writes it back to all the `useState` setters. This is the harder direction — many small `setX(state.x)` calls.
- Add Save and Load buttons in the UI. Save calls `exportPreset(getAppState())` then `downloadPreset()`. Load calls `pickAndLoadPreset(registry)` then `applyAppState(result.state)`.
- Show import warnings in the UI (a small dismissible banner) — they're useful for debugging missing transforms after a rename.

### 3. Add per-layer blend (~1-2 hours)

- Each layer state object gets a `blend` field (a `Param` object) and `blendTimeOverride: number | null`. Update `makeLayer` and `makeDispLayer` to initialize these.
- The `applyPipeline` function in v5 currently does straight composition. Update it: each layer's contribution is mixed with the prior state by `blend.value`. For an operation: `out = lerp(input, transform(input), blend)`. For a lift: scale the displacement vector by `blend`.
- Add a blend slider to each layer card (`PipelineCard` and `DispPipelineCard`). Should be visible when the card is expanded.
- The existing `blend` utility transform (in TRANSFORMS) keeps its current behavior — it's a different thing. The user-facing label could change from "blend with input" to just "blend (utility)" for clarity, but the function itself is unchanged.

### 4. Implement smooth transitions (~2-3 hours, this is the hard one)

This is the substantive new system. The principle: every state value the animation loop touches has a `target`; the live `value` eases toward `target` over a configurable time.

Approach:

- Add a `target` field alongside every animatable value in state. For params, this means `value` and `target` (where `target` is what the eventual destination is — by default they're equal).
- The animation tick reads `value`, eases it toward `target`, and writes back. Easing function: cosine ease-out is fine, or use an exponential decay with a time constant.
- Direct manipulation (slider drag) sets both `value` and `target` to the new value — instant.
- Indirect manipulation (preset load, layer toggle, MIDI) sets only `target`; the live value catches up over time.
- For layer enable/disable: instead of flipping `enabled`, animate `blend.target` between 0 and 1.

A subtler version: don't store `target` redundantly with `value`. Instead, have a separate "transition manager" that holds a list of active eases. When a transition starts, it pushes an entry: `{ paramRef, fromValue, toValue, startTime, duration }`. The animation loop reads pending transitions and writes the interpolated value. Done transitions get cleared.

The list-based approach is cleaner because it lets you cancel transitions, retarget mid-flight, and inspect what's animating. The redundant-target approach is simpler to start with. Pick based on how much complexity you're willing to take on in one pass; the list approach is what you want long-term.

### 5. Reorder via fade (~30 min, depends on #4)

Once smooth transitions exist, reorder is just: kick off `blend.target = 0`, wait for blend time, swap array position, kick off `blend.target = 1`. Since these are blend times, the two halves can run in series (queued transitions). Total UX: ~1 second per reorder. May feel slow; tune if so.

### 6. Polish

- Verify the dim readout still works correctly (the new blend params count too — they're animatable DOFs)
- Verify the composition string still reads correctly with new vocabulary
- Verify the file still parses cleanly in strict mode (we caught smart-quote bugs earlier; don't reintroduce)

## Known gotchas from prior development

**Smart quotes break the file.** Past versions of the JSX had curly quotes (`""` instead of `""`) that broke the parser. When pasting code into the file, normalize quotes. Run a quick grep before declaring done:

```bash
python3 -c "src=open('file.jsx').read(); [print(f'found {repr(c)}') for c in ['\u2018','\u2019','\u201c','\u201d','\u2014','\u2013'] if c in src]"
```

**Animation loop performance.** The v5 animation loop calls `setLayers`/`setDispLayers` every frame, which causes full React re-renders on every tick. This was already a known perf issue and isn't getting worse, but if you find yourself needing to optimize, the right move is to push the animation state into a ref-based system that bypasses React reconciliation.

**Three.js matrix updates.** When reading `worldGroup.matrixWorld` in the render loop (for the lens distortion path), you need to call `updateMatrixWorld(true)` first. Already handled in v5; just don't break it.

**Pole degeneracy in camera.** The camera up-vector code falls back to world Z when `fwd` is parallel to world Y (looking straight up/down). Already handled in v5; just don't break it.

**Three.js r128.** The version we're using doesn't have `THREE.OrbitControls` available without an extra import, and lacks `THREE.CapsuleGeometry` (r142+). The v5 component avoids both. Don't add them.

**Curly braces in JSX strings.** The previous file had unescaped `{` and `}` characters in strings used as labels. Most worked but some triggered JSX parser quirks. If labels with math notation are used (`r' = r(1 + k1·r²)`), prefer ASCII over Unicode for safety in v6 just like v5.

## Verification checklist for v6

When the React component is updated, before declaring done:

- [ ] `polydimensional-coupling-v6.jsx` parses cleanly (strict TypeScript or babel parser)
- [ ] All UI labels use the new vocabulary (no "transform pipeline", no "displacement pipeline", no "view layers")
- [ ] Save button produces valid JSON that `validatePreset` accepts
- [ ] Load button accepts files produced by Save
- [ ] Loading a preset visibly applies the new state
- [ ] Each layer has a working blend slider
- [ ] Disabling a layer fades its contribution out smoothly (not a jump)
- [ ] Enabling a layer fades it back in
- [ ] Reordering a layer fades out, swaps, fades in
- [ ] Dimension readout still updates correctly
- [ ] Composition string still updates correctly
- [ ] Smart-quote check passes (script above)

## What comes after v6

Phase 2 is the native Rust application. That's a separate session, much larger scope. The work order is roughly:

1. Spike: Rust + wgpu + egui project that opens a control window and an output window and renders something.
2. Verify SpoutCapture sees the output window in Resolume.
3. Define the registry on the Rust side (transform/lift definitions in WGSL shaders).
4. Port the preset format to Rust (serde_json + types).
5. Build the rendering core (compute shader pipeline).
6. Build the egui control UI.
7. Implement smooth transitions (same primitive as v6 React).
8. Implement basic preset save/load.

Don't start any of this until v6 is solid and the user has spent real time with it. The web prototype is where decisions are still being made.

## Open invitation to the next session

If you encounter a design decision that affects the project's identity or feel, ask the user before deciding. The constraints documented here are load-bearing. If you encounter a small implementation choice (variable names, code style, whether to break a function in two), use judgment.

Things specifically worth asking about:
- Anything that touches the dimension readout's prominence or accuracy
- Anything that affects the smoothness of transitions in performance use
- Anything that lets the UI vocabulary drift back toward technical/engineering language
- Anything that suggests removing the 2D grid as the anchor

Things you can decide unilaterally:
- Code organization, refactoring within the file
- Which easing function to use for transitions
- UI spacing, color tweaks, button placement details
- Test coverage decisions
