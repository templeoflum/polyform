// =============================================================================
// Polyform preset schema
//
// A preset is the complete patch state needed to reproduce a configuration:
// pipeline structure, parameter values and ranges, camera, sampling, palette,
// tempo, and display toggles. It does NOT contain workspace state (which UI
// panels are open, MIDI mappings, recent files).
//
// Every animatable parameter has the same shape (Param). Every layer has a
// stable id that survives saves, edits, and reorderings, so morphing systems
// can match layers across presets by identity rather than position.
//
// Forward compatibility: unknown fields are tolerated silently. Missing
// required fields fall back to sensible defaults. Type mismatches are rejected.
// =============================================================================

export const SCHEMA_NAME = "polyform.preset";
export const SCHEMA_VERSION = 1;

// -----------------------------------------------------------------------------
// Core: Param. Every animatable scalar in the system uses this shape.
// -----------------------------------------------------------------------------

export type AnimMode = "loop" | "sweep";

export interface Param {
  /** Current value (and the value last set manually if anim is on). */
  value: number;
  /** Lower bound of the slider/animation range. */
  min: number;
  /** Upper bound of the slider/animation range. */
  max: number;
  /**
   * Whether this param animates automatically on each tick.
   * When animating, value sweeps or loops between min and max.
   */
  anim: boolean;
  /**
   * loop  - wrap min..max periodically (rotations, phases)
   * sweep - cosine ping-pong between min and max (most other things)
   */
  animMode: AnimMode;
  /** Multiplier on global tempo. ratio=1 means one full cycle per tempo unit. */
  ratio: number;
  /**
   * Optional phase offset for FM-style modulation. Value is fraction of one
   * full cycle (0 = no offset, 0.5 = half cycle). The phaseOffset is itself
   * a Param and can be animated.
   */
  phaseOffset?: Param;
}

/**
 * Toggleable param: a Param that can also be enabled or disabled wholesale.
 * Camera DOFs, sampling controls, etc. use this so they can be opted out of.
 */
export interface ToggleParam extends Param {
  on: boolean;
}

// -----------------------------------------------------------------------------
// Layer (operation or lift) parameters
// -----------------------------------------------------------------------------

/**
 * Per-layer mix amount in [0, 1].
 * 0 = layer is identity (transform layers) or contributes nothing (lift layers)
 * 1 = layer is fully active
 *
 * Toggling a layer enabled/disabled animates this between 0 and 1 over the
 * configured blend time. Direct slider edits set the value immediately.
 */
export type Blend = Param;

export interface Layer {
  /** Stable, unique id; persists across saves and reorderings. */
  id: string;
  /** Registry key (e.g. "zPow", "zDisplacement"). */
  type: string;
  /**
   * User-facing on/off. Animation on this is what makes blend smooth.
   * The actual visible mix amount is in `blend.value`.
   */
  enabled: boolean;
  /** Per-layer blend; see Blend doc. */
  blend: Blend;
  /**
   * Per-layer override for blend time, in seconds. null means use the global
   * default from preset.defaults.blendTime.
   */
  blendTimeOverride: number | null;
  /** Layer-specific params keyed by the names defined in the registry. */
  params: { [key: string]: Param };
}

// -----------------------------------------------------------------------------
// Camera
// -----------------------------------------------------------------------------

export type ProjectionMode = "perspective" | "orthographic";

export interface CameraPosition {
  orbitX: ToggleParam;
  orbitY: ToggleParam;
  distance: ToggleParam;
  targetX: ToggleParam;
  targetY: ToggleParam;
  roll: ToggleParam;
}

export interface CameraLens {
  projection: ProjectionMode;
  focalLength: ToggleParam;
  sensorSize: ToggleParam;
  orthoZoom: ToggleParam;
  anamorphic: ToggleParam;
  shiftX: ToggleParam;
  shiftY: ToggleParam;
}

export interface CameraDistortion {
  k1: ToggleParam;
  k2: ToggleParam;
}

export interface Camera {
  position: CameraPosition;
  lens: CameraLens;
  distortion: CameraDistortion;
}

// -----------------------------------------------------------------------------
// Quaternion tumble
// -----------------------------------------------------------------------------

export type QuaternionMode = "fixed" | "drift";

export interface QuaternionAxis {
  value: number;
  anim: boolean;
  ratio: number;
}

export interface Quaternion {
  on: boolean;
  mode: QuaternionMode;
  /** Whether the angle param itself animates. */
  anim: boolean;
  angle: number;
  angleRatio: number;
  axes: {
    ax: QuaternionAxis;
    ay: QuaternionAxis;
    az: QuaternionAxis;
  };
  driftSpeed: number;
}

// -----------------------------------------------------------------------------
// Sampling (lattice extent + density)
// -----------------------------------------------------------------------------

export interface Sampling {
  scale: ToggleParam;
  density: ToggleParam;
}

// -----------------------------------------------------------------------------
// Display toggles
// -----------------------------------------------------------------------------

export interface Display {
  showFormula: boolean;
  showDimReadout: boolean;
}

// -----------------------------------------------------------------------------
// Top-level preset
// -----------------------------------------------------------------------------

export interface PresetMetadata {
  /** User-facing name shown in the preset browser. */
  name: string;
  description: string;
  author: string;
  /** ISO 8601 timestamp. */
  created: string;
  /** ISO 8601 timestamp. */
  modified: string;
  tags: string[];
}

export interface PresetDefaults {
  /**
   * Default blend time in seconds. Used for layer enable/disable transitions,
   * preset loads, and reorder bracketing, when the specific scope doesn't
   * override it.
   */
  blendTime: number;
}

export interface Pipeline {
  /** XY plane operations (formerly "transforms"). Order = composition order. */
  operations: Layer[];
  /** 3D lift (formerly "displacements"). Order = sequential displacement. */
  lift: Layer[];
}

export interface Preset {
  schema: typeof SCHEMA_NAME;
  version: number;
  metadata: PresetMetadata;
  /** Tempo is part of the preset so morphing can crossfade it smoothly. */
  tempo: Param;
  defaults: PresetDefaults;
  palette: string;
  pipeline: Pipeline;
  camera: Camera;
  quaternion: Quaternion;
  sampling: Sampling;
  display: Display;
}

// =============================================================================
// Defaults
// =============================================================================

const PHI = (1 + Math.sqrt(5)) / 2;

export function defaultBlend(): Blend {
  return {
    value: 1.0,
    min: 0,
    max: 1,
    anim: false,
    animMode: "sweep",
    ratio: 1,
  };
}

export function defaultPhaseOffset(): Param {
  return { value: 0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
}

export function defaultParam(value: number, min: number, max: number, periodic = false): Param {
  return {
    value,
    min,
    max,
    anim: false,
    animMode: periodic ? "loop" : "sweep",
    ratio: 1,
  };
}

export function defaultToggleParam(
  on: boolean,
  value: number,
  min: number,
  max: number,
  periodic = false,
): ToggleParam {
  return { ...defaultParam(value, min, max, periodic), on };
}

export function defaultMetadata(): PresetMetadata {
  const now = new Date().toISOString();
  return {
    name: "Untitled",
    description: "",
    author: "",
    created: now,
    modified: now,
    tags: [],
  };
}

export function defaultCamera(): Camera {
  return {
    position: {
      orbitX: defaultToggleParam(true, 0, 0, Math.PI * 2, true),
      orbitY: { ...defaultToggleParam(true, 0.6, 0.1, Math.PI - 0.1, true), ratio: 1 / PHI },
      distance: defaultToggleParam(true, 3.5, 0.8, 12),
      targetX: defaultToggleParam(false, 0, -2, 2),
      targetY: defaultToggleParam(false, 0, -2, 2),
      roll: defaultToggleParam(false, 0, 0, Math.PI * 2, true),
    },
    lens: {
      projection: "perspective",
      focalLength: defaultToggleParam(true, 35, 8, 200),
      sensorSize: defaultToggleParam(true, 36, 4, 80),
      orthoZoom: defaultToggleParam(true, 2, 0.3, 6),
      anamorphic: defaultToggleParam(false, 1, 0.3, 3),
      shiftX: defaultToggleParam(false, 0, -1, 1),
      shiftY: defaultToggleParam(false, 0, -1, 1),
    },
    distortion: {
      k1: defaultToggleParam(false, 0, -1, 1),
      k2: defaultToggleParam(false, 0, -1, 1),
    },
  };
}

export function defaultQuaternion(): Quaternion {
  return {
    on: false,
    mode: "fixed",
    anim: true,
    angle: 0,
    angleRatio: 1,
    axes: {
      ax: { value: 0.5, anim: false, ratio: PHI },
      ay: { value: 1.0, anim: false, ratio: 1 },
      az: { value: 0.3, anim: false, ratio: 1 / PHI },
    },
    driftSpeed: 0.1,
  };
}

export function defaultSampling(): Sampling {
  return {
    scale: defaultToggleParam(true, 0.6, 0.1, 1.5),
    density: defaultToggleParam(true, 27, 7, 51),
  };
}

export function defaultDisplay(): Display {
  return { showFormula: true, showDimReadout: true };
}

export function defaultDefaults(): PresetDefaults {
  return { blendTime: 1.0 };
}

export function defaultTempo(): Param {
  return defaultParam(0.3, 0, 2);
}

export function emptyPreset(): Preset {
  return {
    schema: SCHEMA_NAME,
    version: SCHEMA_VERSION,
    metadata: defaultMetadata(),
    tempo: defaultTempo(),
    defaults: defaultDefaults(),
    palette: "acid",
    pipeline: { operations: [], lift: [] },
    camera: defaultCamera(),
    quaternion: defaultQuaternion(),
    sampling: defaultSampling(),
    display: defaultDisplay(),
  };
}

// =============================================================================
// Validation
//
// validatePreset returns either { ok: true, preset } with all defaults filled
// in, or { ok: false, error } with a human-readable error. It tolerates extra
// fields and missing fields, but rejects type mismatches on required scalars.
//
// The returned preset is always a fully-populated Preset object - it's safe
// to use directly without further null-checks.
// =============================================================================

export type ValidationResult =
  | { ok: true; preset: Preset; warnings: string[] }
  | { ok: false; error: string };

interface ValidationContext {
  warnings: string[];
}

function isObject(x: unknown): x is Record<string, unknown> {
  return typeof x === "object" && x !== null && !Array.isArray(x);
}

function asNumber(x: unknown, fallback: number, ctx: ValidationContext, path: string): number {
  if (typeof x === "number" && Number.isFinite(x)) return x;
  if (x !== undefined) ctx.warnings.push(`${path}: expected number, using default ${fallback}`);
  return fallback;
}

function asString(x: unknown, fallback: string): string {
  return typeof x === "string" ? x : fallback;
}

function asBool(x: unknown, fallback: boolean): boolean {
  return typeof x === "boolean" ? x : fallback;
}

function asAnimMode(x: unknown, fallback: AnimMode): AnimMode {
  return x === "loop" || x === "sweep" ? x : fallback;
}

function asProjectionMode(x: unknown, fallback: ProjectionMode): ProjectionMode {
  return x === "perspective" || x === "orthographic" ? x : fallback;
}

function asQuaternionMode(x: unknown, fallback: QuaternionMode): QuaternionMode {
  return x === "fixed" || x === "drift" ? x : fallback;
}

function readParam(input: unknown, fallback: Param, ctx: ValidationContext, path: string): Param {
  if (!isObject(input)) {
    if (input !== undefined) ctx.warnings.push(`${path}: not an object, using default`);
    return { ...fallback };
  }
  const result: Param = {
    value: asNumber(input.value, fallback.value, ctx, `${path}.value`),
    min: asNumber(input.min, fallback.min, ctx, `${path}.min`),
    max: asNumber(input.max, fallback.max, ctx, `${path}.max`),
    anim: asBool(input.anim, fallback.anim),
    animMode: asAnimMode(input.animMode, fallback.animMode),
    ratio: asNumber(input.ratio, fallback.ratio, ctx, `${path}.ratio`),
  };
  if (input.phaseOffset !== undefined) {
    result.phaseOffset = readParam(input.phaseOffset, defaultPhaseOffset(), ctx, `${path}.phaseOffset`);
  }
  return result;
}

function readToggleParam(
  input: unknown,
  fallback: ToggleParam,
  ctx: ValidationContext,
  path: string,
): ToggleParam {
  const base = readParam(input, fallback, ctx, path);
  const on = isObject(input) ? asBool(input.on, fallback.on) : fallback.on;
  return { ...base, on };
}

function readLayer(input: unknown, ctx: ValidationContext, path: string): Layer | null {
  if (!isObject(input)) {
    ctx.warnings.push(`${path}: not an object, dropping layer`);
    return null;
  }
  const type = asString(input.type, "");
  if (!type) {
    ctx.warnings.push(`${path}: missing layer type, dropping layer`);
    return null;
  }
  const id = asString(input.id, "") || generateLayerId();
  const params: { [key: string]: Param } = {};
  if (isObject(input.params)) {
    for (const k of Object.keys(input.params)) {
      // We don't know the registry-defined defaults here. Use a permissive default
      // and let the consumer reconcile with the registry on load.
      params[k] = readParam(
        input.params[k],
        defaultParam(0, -1, 1),
        ctx,
        `${path}.params.${k}`,
      );
    }
  }
  return {
    id,
    type,
    enabled: asBool(input.enabled, true),
    blend: readParam(input.blend, defaultBlend(), ctx, `${path}.blend`),
    blendTimeOverride:
      typeof input.blendTimeOverride === "number" ? input.blendTimeOverride : null,
    params,
  };
}

function readLayerArray(input: unknown, ctx: ValidationContext, path: string): Layer[] {
  if (!Array.isArray(input)) return [];
  return input
    .map((entry, i) => readLayer(entry, ctx, `${path}[${i}]`))
    .filter((l): l is Layer => l !== null);
}

function readCamera(input: unknown, ctx: ValidationContext): Camera {
  const fallback = defaultCamera();
  if (!isObject(input)) return fallback;
  const pos = isObject(input.position) ? input.position : {};
  const lens = isObject(input.lens) ? input.lens : {};
  const dist = isObject(input.distortion) ? input.distortion : {};
  return {
    position: {
      orbitX: readToggleParam(pos.orbitX, fallback.position.orbitX, ctx, "camera.position.orbitX"),
      orbitY: readToggleParam(pos.orbitY, fallback.position.orbitY, ctx, "camera.position.orbitY"),
      distance: readToggleParam(pos.distance, fallback.position.distance, ctx, "camera.position.distance"),
      targetX: readToggleParam(pos.targetX, fallback.position.targetX, ctx, "camera.position.targetX"),
      targetY: readToggleParam(pos.targetY, fallback.position.targetY, ctx, "camera.position.targetY"),
      roll: readToggleParam(pos.roll, fallback.position.roll, ctx, "camera.position.roll"),
    },
    lens: {
      projection: asProjectionMode(lens.projection, fallback.lens.projection),
      focalLength: readToggleParam(lens.focalLength, fallback.lens.focalLength, ctx, "camera.lens.focalLength"),
      sensorSize: readToggleParam(lens.sensorSize, fallback.lens.sensorSize, ctx, "camera.lens.sensorSize"),
      orthoZoom: readToggleParam(lens.orthoZoom, fallback.lens.orthoZoom, ctx, "camera.lens.orthoZoom"),
      anamorphic: readToggleParam(lens.anamorphic, fallback.lens.anamorphic, ctx, "camera.lens.anamorphic"),
      shiftX: readToggleParam(lens.shiftX, fallback.lens.shiftX, ctx, "camera.lens.shiftX"),
      shiftY: readToggleParam(lens.shiftY, fallback.lens.shiftY, ctx, "camera.lens.shiftY"),
    },
    distortion: {
      k1: readToggleParam(dist.k1, fallback.distortion.k1, ctx, "camera.distortion.k1"),
      k2: readToggleParam(dist.k2, fallback.distortion.k2, ctx, "camera.distortion.k2"),
    },
  };
}

function readQuaternion(input: unknown, ctx: ValidationContext): Quaternion {
  const fallback = defaultQuaternion();
  if (!isObject(input)) return fallback;
  const axes = isObject(input.axes) ? input.axes : {};
  const readAxis = (raw: unknown, fb: QuaternionAxis, path: string): QuaternionAxis => {
    if (!isObject(raw)) return { ...fb };
    return {
      value: asNumber(raw.value, fb.value, ctx, `${path}.value`),
      anim: asBool(raw.anim, fb.anim),
      ratio: asNumber(raw.ratio, fb.ratio, ctx, `${path}.ratio`),
    };
  };
  return {
    on: asBool(input.on, fallback.on),
    mode: asQuaternionMode(input.mode, fallback.mode),
    anim: asBool(input.anim, fallback.anim),
    angle: asNumber(input.angle, fallback.angle, ctx, "quaternion.angle"),
    angleRatio: asNumber(input.angleRatio, fallback.angleRatio, ctx, "quaternion.angleRatio"),
    axes: {
      ax: readAxis(axes.ax, fallback.axes.ax, "quaternion.axes.ax"),
      ay: readAxis(axes.ay, fallback.axes.ay, "quaternion.axes.ay"),
      az: readAxis(axes.az, fallback.axes.az, "quaternion.axes.az"),
    },
    driftSpeed: asNumber(input.driftSpeed, fallback.driftSpeed, ctx, "quaternion.driftSpeed"),
  };
}

function readSampling(input: unknown, ctx: ValidationContext): Sampling {
  const fallback = defaultSampling();
  if (!isObject(input)) return fallback;
  return {
    scale: readToggleParam(input.scale, fallback.scale, ctx, "sampling.scale"),
    density: readToggleParam(input.density, fallback.density, ctx, "sampling.density"),
  };
}

function readDisplay(input: unknown): Display {
  const fb = defaultDisplay();
  if (!isObject(input)) return fb;
  return {
    showFormula: asBool(input.showFormula, fb.showFormula),
    showDimReadout: asBool(input.showDimReadout, fb.showDimReadout),
  };
}

function readMetadata(input: unknown): PresetMetadata {
  const fb = defaultMetadata();
  if (!isObject(input)) return fb;
  return {
    name: asString(input.name, fb.name),
    description: asString(input.description, fb.description),
    author: asString(input.author, fb.author),
    created: asString(input.created, fb.created),
    modified: asString(input.modified, fb.modified),
    tags: Array.isArray(input.tags) ? input.tags.filter((t): t is string => typeof t === "string") : [],
  };
}

function readDefaults(input: unknown, ctx: ValidationContext): PresetDefaults {
  const fb = defaultDefaults();
  if (!isObject(input)) return fb;
  return {
    blendTime: asNumber(input.blendTime, fb.blendTime, ctx, "defaults.blendTime"),
  };
}

/**
 * Read an unknown JSON-parsed value and return a fully-populated, validated Preset.
 * Tolerates missing fields (uses defaults), warns on type mismatches, and rejects
 * only at the top level if the input isn't an object at all.
 *
 * Future schema versions are also accepted - a warning is added but the file is
 * still loaded, on the principle that newer presets in older readers should mostly
 * still work.
 */
export function validatePreset(raw: unknown): ValidationResult {
  if (!isObject(raw)) {
    return { ok: false, error: "Preset is not a JSON object." };
  }
  const ctx: ValidationContext = { warnings: [] };

  const schema = asString(raw.schema, "");
  if (schema && schema !== SCHEMA_NAME) {
    ctx.warnings.push(`Schema name "${schema}" is unrecognized; expected "${SCHEMA_NAME}".`);
  }

  const version = asNumber(raw.version, SCHEMA_VERSION, ctx, "version");
  if (version > SCHEMA_VERSION) {
    ctx.warnings.push(
      `Preset is from a newer schema version (${version} > ${SCHEMA_VERSION}). Some fields may be ignored.`,
    );
  } else if (version < SCHEMA_VERSION) {
    // Future migration hook: run upgrade(version, raw) here.
    ctx.warnings.push(
      `Preset is from an older schema version (${version} < ${SCHEMA_VERSION}). Loaded with current defaults.`,
    );
  }

  const pipeline: Pipeline = isObject(raw.pipeline)
    ? {
        operations: readLayerArray(raw.pipeline.operations, ctx, "pipeline.operations"),
        lift: readLayerArray(raw.pipeline.lift, ctx, "pipeline.lift"),
      }
    : { operations: [], lift: [] };

  const preset: Preset = {
    schema: SCHEMA_NAME,
    version: SCHEMA_VERSION,
    metadata: readMetadata(raw.metadata),
    tempo: readParam(raw.tempo, defaultTempo(), ctx, "tempo"),
    defaults: readDefaults(raw.defaults, ctx),
    palette: asString(raw.palette, "acid"),
    pipeline,
    camera: readCamera(raw.camera, ctx),
    quaternion: readQuaternion(raw.quaternion, ctx),
    sampling: readSampling(raw.sampling, ctx),
    display: readDisplay(raw.display),
  };

  return { ok: true, preset, warnings: ctx.warnings };
}

// =============================================================================
// IDs
// =============================================================================

/** Eight-character hex-ish id. Collision odds: ~1 in 2.8 trillion. */
export function generateLayerId(): string {
  return Math.random().toString(36).slice(2, 10).padEnd(8, "0");
}
