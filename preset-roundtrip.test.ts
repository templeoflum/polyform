import { describe, it, expect } from "vitest";
import {
  emptyPreset,
  validatePreset,
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

function buildTestState(): AppState {
  const fresh = emptyPreset();
  const op1 = makeLayerFromRegistry("zPow", registry.operations.zPow);
  op1.params.n.value = 3.5;
  op1.params.n.anim = true;
  op1.params.n.ratio = 1.618;
  const op2 = makeLayerFromRegistry("rotate", registry.operations.rotate);
  op2.params.theta.value = Math.PI / 4;
  const lift1 = makeLayerFromRegistry("zDisplacement", registry.lift.zDisplacement);
  lift1.params.amp.value = 0.5;

  return {
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
}

describe("preset round-trip", () => {
  const state = buildTestState();
  const exported = exportPreset(state, { name: "Test Preset", author: "smoke-test" });
  const json = presetToJSON(exported);

  it("import succeeds", () => {
    const imported = importPresetJSON(json, registry);
    expect(imported.ok).toBe(true);
  });

  it("palette preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.palette).toBe(state.palette);
  });

  it("operation count preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations.length).toBe(state.pipeline.operations.length);
  });

  it("lift count preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.lift.length).toBe(state.pipeline.lift.length);
  });

  it("op[0] id preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations[0].id).toBe(state.pipeline.operations[0].id);
  });

  it("op[0] type preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations[0].type).toBe(state.pipeline.operations[0].type);
  });

  it("op[0] param value preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations[0].params.n.value).toBe(state.pipeline.operations[0].params.n.value);
  });

  it("op[0] param anim flag preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations[0].params.n.anim).toBe(state.pipeline.operations[0].params.n.anim);
  });

  it("op[0] param ratio preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations[0].params.n.ratio).toBeCloseTo(state.pipeline.operations[0].params.n.ratio, 9);
  });

  it("lift[0] amp preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.lift[0].params.amp.value).toBe(state.pipeline.lift[0].params.amp.value);
  });

  it("camera projection preserved", () => {
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.camera.lens.projection).toBe(state.camera.lens.projection);
  });
});

describe("Display fields round-trip", () => {
  it("all 5 display fields preserved", () => {
    const state = buildTestState();
    state.display = {
      showFormula: false,
      showDimCount: true,
      showActiveLayers: false,
      showParamValues: true,
      showFps: true,
    };
    const exported = exportPreset(state, { name: "Display Test" });
    const json = presetToJSON(exported);
    const imported = importPresetJSON(json, registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.display.showFormula).toBe(false);
    expect(imported.state.display.showDimCount).toBe(true);
    expect(imported.state.display.showActiveLayers).toBe(false);
    expect(imported.state.display.showParamValues).toBe(true);
    expect(imported.state.display.showFps).toBe(true);
  });

  it("default display has all 5 fields", () => {
    const preset = emptyPreset();
    expect(preset.display.showFormula).toBe(true);
    expect(preset.display.showDimCount).toBe(true);
    expect(preset.display.showActiveLayers).toBe(true);
    expect(preset.display.showParamValues).toBe(true);
    expect(preset.display.showFps).toBe(false);
  });
});

describe("forward compatibility", () => {
  it("unknown layer type is dropped with warning", () => {
    const state = buildTestState();
    const exported = exportPreset(state);
    const raw = JSON.parse(presetToJSON(exported));
    raw.pipeline.operations.push({
      id: "future01",
      type: "futureType",
      enabled: true,
      blend: defaultBlend(),
      blendTimeOverride: null,
      params: {},
    });
    const imported = importPresetJSON(JSON.stringify(raw), registry);
    if (!imported.ok) throw new Error("import failed");
    expect(imported.state.pipeline.operations.length).toBe(state.pipeline.operations.length);
    expect(imported.warnings.some((w) => w.includes("futureType"))).toBe(true);
  });
});

describe("validation edge cases", () => {
  it("empty object validates to usable preset with default tempo", () => {
    const result = validatePreset({});
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.preset.tempo.value).toBe(0.3);
    }
  });

  it("bogus input (string) is rejected", () => {
    const result = validatePreset("not a preset");
    expect(result.ok).toBe(false);
  });
});
