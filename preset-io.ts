// =============================================================================
// Polyform preset I/O -- CANONICAL SPEC
//
// This file is the single source of truth for preset import/export logic. The
// Rust port must reproduce these conversion functions exactly.
//
// Bridges between in-app React state and the on-disk Preset schema.
//
// Two directions:
//   exportPreset(state)  - take live state, produce a Preset ready to serialize
//   importPreset(preset, registry) - take a validated Preset, produce React state
//                                    reconciled against the current registry
//
// The reconciliation step is important: a preset references transform/lift
// types by name (e.g. "zPow"). The registry that defines those names lives in
// the application code. On import, we filter out unknown types (with a warning)
// and merge any registry-defined defaults the preset didn't explicitly set.
// =============================================================================

import {
  Blend,
  Camera,
  Display,
  Layer,
  Param,
  Pipeline,
  Preset,
  Quaternion,
  Sampling,
  ToggleParam,
  defaultBlend,
  emptyPreset,
  generateLayerId,
  validatePreset,
} from "./preset-schema";

// -----------------------------------------------------------------------------
// Registry shape: minimal contract this module needs from the app's transform
// and lift definitions. The actual TRANSFORMS / DISPLACEMENTS objects in the
// app should fit this shape (or be adapted to it).
// -----------------------------------------------------------------------------

export interface ParamDef {
  key: string;
  default: number;
  min: number;
  max: number;
  periodic?: boolean;
}

export interface LayerTypeDef {
  /** Registry key, e.g. "zPow". */
  key: string;
  /** Param definitions in registry order. */
  params: ParamDef[];
}

export interface Registry {
  operations: { [key: string]: LayerTypeDef };
  lift: { [key: string]: LayerTypeDef };
}

// -----------------------------------------------------------------------------
// Live React-state shape (mirrors what's currently in the v5 component, but
// reorganized so this module isn't tightly coupled to React).
//
// The caller is responsible for translating between this AppState and the
// individual useState hooks in the component.
// -----------------------------------------------------------------------------

export interface AppState {
  tempo: Param;
  palette: string;
  pipeline: Pipeline;
  camera: Camera;
  quaternion: Quaternion;
  sampling: Sampling;
  display: Display;
  defaults: { blendTime: number };
}

// =============================================================================
// Export: AppState -> Preset
// =============================================================================

/**
 * Build a Preset from current app state. Metadata (name, timestamps, etc.) is
 * supplied by the caller; everything else is read from state as-is.
 *
 * Only the structural state is serialized - no animation phases, no transient
 * UI flags. The result round-trips cleanly back through importPreset.
 */
export function exportPreset(
  state: AppState,
  metadata: Partial<Preset["metadata"]> = {},
): Preset {
  const base = emptyPreset();
  const now = new Date().toISOString();
  return {
    ...base,
    metadata: {
      ...base.metadata,
      ...metadata,
      modified: now,
      created: metadata.created || base.metadata.created,
    },
    tempo: cloneParam(state.tempo),
    defaults: { blendTime: state.defaults.blendTime },
    palette: state.palette,
    pipeline: cloneLayerStructure(state.pipeline),
    camera: cloneCamera(state.camera),
    quaternion: cloneQuaternion(state.quaternion),
    sampling: cloneSampling(state.sampling),
    display: { ...state.display },
  };
}

export function presetToJSON(preset: Preset, pretty = true): string {
  return JSON.stringify(preset, null, pretty ? 2 : 0);
}

// =============================================================================
// Import: JSON -> Preset -> AppState
// =============================================================================

export type ImportResult =
  | { ok: true; state: AppState; warnings: string[] }
  | { ok: false; error: string };

export function importPresetJSON(json: string, registry: Registry): ImportResult {
  let raw: unknown;
  try {
    raw = JSON.parse(json);
  } catch (e) {
    return { ok: false, error: `Invalid JSON: ${(e as Error).message}` };
  }
  return importPresetValue(raw, registry);
}

export function importPresetValue(raw: unknown, registry: Registry): ImportResult {
  const result = validatePreset(raw);
  if (!result.ok) return { ok: false, error: result.error };
  const { state, registryWarnings } = reconcileWithRegistry(result.preset, registry);
  return {
    ok: true,
    state,
    warnings: [...result.warnings, ...registryWarnings],
  };
}

// =============================================================================
// Registry reconciliation
//
// Every layer in the preset must correspond to a registered type, and its
// params must include all params the registry knows about. This step:
//
//   1. Drops layers whose type isn't in the registry (with warning).
//   2. For surviving layers, fills in any registry-defined params that are
//      missing from the preset (using registry defaults).
//   3. Drops preset params the registry doesn't know about (with warning) -
//      these would be dead state otherwise.
//   4. Clamps preset param values into the registry-defined min/max range
//      ONLY if the preset's own min/max are clearly out of registry bounds.
//      Otherwise the user's range overrides win - they may have intentionally
//      narrowed or shifted the playable range.
// =============================================================================

function reconcileWithRegistry(
  preset: Preset,
  registry: Registry,
): { state: AppState; registryWarnings: string[] } {
  const warnings: string[] = [];

  const reconcileLayers = (layers: Layer[], defs: { [k: string]: LayerTypeDef }, kind: string) => {
    const result: Layer[] = [];
    for (const layer of layers) {
      const def = defs[layer.type];
      if (!def) {
        warnings.push(`Unknown ${kind} type "${layer.type}" - layer dropped.`);
        continue;
      }
      const reconciledParams: { [key: string]: Param } = {};
      for (const pdef of def.params) {
        const existing = layer.params[pdef.key];
        if (existing) {
          reconciledParams[pdef.key] = existing;
        } else {
          reconciledParams[pdef.key] = {
            value: pdef.default,
            min: pdef.min,
            max: pdef.max,
            anim: false,
            animMode: pdef.periodic ? "loop" : "sweep",
            ratio: 1,
          };
        }
      }
      // Detect orphaned params (in preset but not in registry).
      for (const k of Object.keys(layer.params)) {
        if (!def.params.some((p) => p.key === k)) {
          warnings.push(`${kind} "${layer.type}" has unknown param "${k}" - dropped.`);
        }
      }
      result.push({ ...layer, params: reconciledParams });
    }
    return result;
  };

  const pipeline: Pipeline = {
    operations: reconcileLayers(preset.pipeline.operations, registry.operations, "operation"),
    lift: reconcileLayers(preset.pipeline.lift, registry.lift, "lift"),
  };

  const state: AppState = {
    tempo: preset.tempo,
    palette: preset.palette,
    pipeline,
    camera: preset.camera,
    quaternion: preset.quaternion,
    sampling: preset.sampling,
    display: preset.display,
    defaults: preset.defaults,
  };

  return { state, registryWarnings: warnings };
}

// =============================================================================
// Layer construction (used when adding a layer interactively)
// =============================================================================

export function makeLayerFromRegistry(typeKey: string, def: LayerTypeDef): Layer {
  const params: { [key: string]: Param } = {};
  for (const p of def.params) {
    params[p.key] = {
      value: p.default,
      min: p.min,
      max: p.max,
      anim: false,
      animMode: p.periodic ? "loop" : "sweep",
      ratio: 1,
    };
  }
  return {
    id: generateLayerId(),
    type: typeKey,
    enabled: true,
    blend: defaultBlend(),
    blendTimeOverride: null,
    params,
  };
}

// =============================================================================
// Cloning helpers (deep, but flat enough that JSON.parse(JSON.stringify) is fine
// for many cases - the named clones here exist to be explicit about ownership).
// =============================================================================

function cloneParam(p: Param): Param {
  const c = { ...p };
  if (p.phaseOffset) c.phaseOffset = { ...p.phaseOffset };
  return c;
}

function cloneToggleParam(p: ToggleParam): ToggleParam {
  const c = { ...p };
  if (p.phaseOffset) c.phaseOffset = { ...p.phaseOffset };
  return c;
}

function cloneBlend(b: Blend): Blend {
  const c = { ...b };
  if (b.phaseOffset) c.phaseOffset = { ...b.phaseOffset };
  return c;
}

function cloneLayer(l: Layer): Layer {
  const params: { [k: string]: Param } = {};
  for (const k of Object.keys(l.params)) params[k] = cloneParam(l.params[k]);
  return {
    id: l.id,
    type: l.type,
    enabled: l.enabled,
    blend: cloneBlend(l.blend),
    blendTimeOverride: l.blendTimeOverride,
    params,
  };
}

function cloneLayerStructure(p: Pipeline): Pipeline {
  return {
    operations: p.operations.map(cloneLayer),
    lift: p.lift.map(cloneLayer),
  };
}

function cloneCamera(c: Camera): Camera {
  return {
    position: {
      orbitX: cloneToggleParam(c.position.orbitX),
      orbitY: cloneToggleParam(c.position.orbitY),
      distance: cloneToggleParam(c.position.distance),
      targetX: cloneToggleParam(c.position.targetX),
      targetY: cloneToggleParam(c.position.targetY),
      roll: cloneToggleParam(c.position.roll),
    },
    lens: {
      projection: c.lens.projection,
      focalLength: cloneToggleParam(c.lens.focalLength),
      sensorSize: cloneToggleParam(c.lens.sensorSize),
      orthoZoom: cloneToggleParam(c.lens.orthoZoom),
      anamorphic: cloneToggleParam(c.lens.anamorphic),
      shiftX: cloneToggleParam(c.lens.shiftX),
      shiftY: cloneToggleParam(c.lens.shiftY),
    },
    distortion: {
      k1: cloneToggleParam(c.distortion.k1),
      k2: cloneToggleParam(c.distortion.k2),
    },
  };
}

function cloneQuaternion(q: Quaternion): Quaternion {
  return {
    on: q.on,
    mode: q.mode,
    anim: q.anim,
    angle: q.angle,
    angleRatio: q.angleRatio,
    axes: {
      ax: { ...q.axes.ax },
      ay: { ...q.axes.ay },
      az: { ...q.axes.az },
    },
    driftSpeed: q.driftSpeed,
  };
}

function cloneSampling(s: Sampling): Sampling {
  return {
    scale: cloneToggleParam(s.scale),
    density: cloneToggleParam(s.density),
  };
}

// =============================================================================
// Browser file helpers
// =============================================================================

/** Trigger a download of the preset as a .json file. */
export function downloadPreset(preset: Preset, filename?: string): void {
  const json = presetToJSON(preset, true);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  const safeName = (preset.metadata.name || "preset").replace(/[^a-z0-9-_ ]/gi, "_");
  a.download = filename || `${safeName}.polyform.json`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * Open a file picker and return the loaded preset (validated, reconciled).
 * Returns null if the user cancels. Errors propagate as ImportResult.
 */
export function pickAndLoadPreset(registry: Registry): Promise<ImportResult | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      try {
        const text = await file.text();
        resolve(importPresetJSON(text, registry));
      } catch (e) {
        resolve({ ok: false, error: `Failed to read file: ${(e as Error).message}` });
      }
    };
    input.click();
  });
}
