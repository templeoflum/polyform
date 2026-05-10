import {
  emptyPreset,
  validatePreset,
  generateLayerId,
  defaultBlend,
} from "./preset-schema";
import {
  AppState,
  Registry,
  exportPreset,
  importPresetJSON,
  presetToJSON,
  makeLayerFromRegistry,
} from "./preset-io";

// Mock registry with a couple of operation and lift types.
const registry: Registry = {
  operations: {
    zPow: {
      key: "zPow",
      params: [{ key: "n", default: 2, min: 0.1, max: 5 }],
    },
    rotate: {
      key: "rotate",
      params: [{ key: "theta", default: 0, min: 0, max: Math.PI * 2, periodic: true }],
    },
  },
  lift: {
    zDisplacement: {
      key: "zDisplacement",
      params: [{ key: "amp", default: 1, min: -2, max: 2 }],
    },
  },
};

// Build a non-trivial AppState with two operations and one lift layer.
const fresh = emptyPreset();
const op1 = makeLayerFromRegistry("zPow", registry.operations.zPow);
op1.params.n.value = 3.5;
op1.params.n.anim = true;
op1.params.n.ratio = 1.618;
const op2 = makeLayerFromRegistry("rotate", registry.operations.rotate);
op2.params.theta.value = Math.PI / 4;
const lift1 = makeLayerFromRegistry("zDisplacement", registry.lift.zDisplacement);
lift1.params.amp.value = 0.5;

const state: AppState = {
  tempo: fresh.tempo,
  palette: "cyan",
  pipeline: {
    operations: [op1, op2],
    lift: [lift1],
  },
  camera: fresh.camera,
  quaternion: fresh.quaternion,
  sampling: fresh.sampling,
  display: fresh.display,
  defaults: fresh.defaults,
};

// Round trip: export -> JSON -> import -> compare
const exported = exportPreset(state, { name: "Test Preset", author: "smoke-test" });
const json = presetToJSON(exported);
console.log("=== Exported preset (truncated) ===");
console.log(json.slice(0, 600) + "...\n");

const imported = importPresetJSON(json, registry);
if (!imported.ok) {
  console.error("Import failed:", imported.error);
  process.exit(1);
}

console.log("=== Import warnings ===", imported.warnings);

// Verify the structural data round-trips.
const before = state;
const after = imported.state;

const checks: Array<[string, boolean]> = [
  ["palette preserved", before.palette === after.palette],
  ["op count preserved", before.pipeline.operations.length === after.pipeline.operations.length],
  ["lift count preserved", before.pipeline.lift.length === after.pipeline.lift.length],
  ["op[0] id preserved", before.pipeline.operations[0].id === after.pipeline.operations[0].id],
  ["op[0] type preserved", before.pipeline.operations[0].type === after.pipeline.operations[0].type],
  [
    "op[0] param value preserved",
    before.pipeline.operations[0].params.n.value === after.pipeline.operations[0].params.n.value,
  ],
  [
    "op[0] param anim flag preserved",
    before.pipeline.operations[0].params.n.anim === after.pipeline.operations[0].params.n.anim,
  ],
  [
    "op[0] param ratio preserved",
    Math.abs(
      before.pipeline.operations[0].params.n.ratio - after.pipeline.operations[0].params.n.ratio,
    ) < 1e-9,
  ],
  ["lift[0] amp preserved", before.pipeline.lift[0].params.amp.value === after.pipeline.lift[0].params.amp.value],
  ["camera projection preserved", before.camera.lens.projection === after.camera.lens.projection],
  ["display showFormula preserved", before.display.showFormula === after.display.showFormula],
];

let allPass = true;
for (const [name, pass] of checks) {
  console.log(`  ${pass ? "OK  " : "FAIL"}  ${name}`);
  if (!pass) allPass = false;
}

// Also test forward compatibility: a preset with an unknown layer type should
// drop that layer with a warning, not crash.
const futurePreset = {
  ...JSON.parse(json),
  pipeline: {
    operations: [
      ...JSON.parse(json).pipeline.operations,
      { id: "future01", type: "futureType", enabled: true, blend: defaultBlend(), blendTimeOverride: null, params: {} },
    ],
    lift: JSON.parse(json).pipeline.lift,
  },
};
const futureImport = importPresetJSON(JSON.stringify(futurePreset), registry);
if (!futureImport.ok) {
  console.error("Future import failed:", futureImport.error);
  process.exit(1);
}
const droppedUnknown = futureImport.state.pipeline.operations.length === before.pipeline.operations.length;
const warnedAboutUnknown = futureImport.warnings.some((w) => w.includes("futureType"));
console.log(`  ${droppedUnknown ? "OK  " : "FAIL"}  unknown layer dropped`);
console.log(`  ${warnedAboutUnknown ? "OK  " : "FAIL"}  warning emitted for unknown layer`);
allPass = allPass && droppedUnknown && warnedAboutUnknown;

// Test missing fields: empty object should validate to a usable empty preset.
const empty = validatePreset({});
if (!empty.ok) {
  console.error("Empty preset failed validation:", empty.error);
  process.exit(1);
}
const emptyHasDefaults = empty.preset.tempo.value === 0.3;
console.log(`  ${emptyHasDefaults ? "OK  " : "FAIL"}  empty preset gets default tempo`);
allPass = allPass && emptyHasDefaults;

// Test a totally bogus input (string).
const bogus = validatePreset("not a preset");
const bogusRejected = !bogus.ok;
console.log(`  ${bogusRejected ? "OK  " : "FAIL"}  bogus input rejected`);
allPass = allPass && bogusRejected;

console.log(allPass ? "\nAll round-trip checks passed." : "\nSome checks FAILED.");
process.exit(allPass ? 0 : 1);
