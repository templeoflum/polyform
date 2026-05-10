import { describe, it, expect } from "vitest";
import {
  TRANSFORMS, DISPLACEMENTS, CATEGORIES, DISP_CATEGORIES,
  PALETTES, RATIO_GROUPS, BPM_PRESETS,
  FORMULA_SYM, REGISTRY,
  PHI,
  generateLayerId, makeLayer, makeDispLayer,
  applyPipeline, applyPipelineFlat,
  applyDispPipeline, applyDispPipelineFlat,
  preFlattenLayers, preFlattenDispLayers,
  compositionString, countDOFs,
} from "./registries.js";

const EPSILON = 1e-9;

function near(a, b, eps = EPSILON) {
  return Math.abs(a - b) < eps;
}

// ============================================================
// Individual transform (refractor) tests
// ============================================================

describe("TRANSFORMS - individual operations", () => {
  it("rotate(1, 0, {theta: pi/2}) -> (0, 1)", () => {
    const [x, y] = TRANSFORMS.rotate.fn(1, 0, { theta: Math.PI / 2 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(1, 6);
  });

  it("scale(1, 1, {sx: 2, sy: 3}) -> (2, 3)", () => {
    const [x, y] = TRANSFORMS.scale.fn(1, 1, { sx: 2, sy: 3 });
    expect(x).toBeCloseTo(2, 6);
    expect(y).toBeCloseTo(3, 6);
  });

  it("zPow(0, 1, {n: 2}) -> (-1, 0)  (i^2 = -1)", () => {
    const [x, y] = TRANSFORMS.zPow.fn(0, 1, { n: 2 });
    expect(x).toBeCloseTo(-1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("zExp(0, 0) -> (1, 0)", () => {
    const [x, y] = TRANSFORMS.zExp.fn(0, 0);
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("conjugate(1, 2) -> (1, -2)", () => {
    const [x, y] = TRANSFORMS.conjugate.fn(1, 2);
    expect(x).toBe(1);
    expect(y).toBe(-2);
  });

  it("translate(1, 2, {tx: 0.5, ty: -0.3}) -> (1.5, 1.7)", () => {
    const [x, y] = TRANSFORMS.translate.fn(1, 2, { tx: 0.5, ty: -0.3 });
    expect(x).toBeCloseTo(1.5, 6);
    expect(y).toBeCloseTo(1.7, 6);
  });

  it("shear(1, 0, {kx: 1, ky: 0}) -> (1, 0)", () => {
    const [x, y] = TRANSFORMS.shear.fn(1, 0, { kx: 1, ky: 0 });
    expect(x).toBe(1);
    expect(y).toBe(0);
  });

  it("shear(0, 1, {kx: 1, ky: 0}) -> (1, 1)", () => {
    const [x, y] = TRANSFORMS.shear.fn(0, 1, { kx: 1, ky: 0 });
    expect(x).toBe(1);
    expect(y).toBe(1);
  });

  it("matrix identity [[1,0],[0,1]] -> same point", () => {
    const [x, y] = TRANSFORMS.matrix.fn(3, 7, { a: 1, b: 0, c: 0, d: 1 });
    expect(x).toBe(3);
    expect(y).toBe(7);
  });

  it("zInv(1, 0) -> approximately (1, 0)", () => {
    // 1/z at z=1: should be ~1. Epsilon from the 1e-6 guard.
    const [x, y] = TRANSFORMS.zInv.fn(1, 0);
    expect(x).toBeCloseTo(1, 4);
    expect(y).toBeCloseTo(0, 4);
  });

  it("zLog(1, 0) -> (0, 0)  (log(1) = 0)", () => {
    const [x, y] = TRANSFORMS.zLog.fn(1, 0);
    expect(x).toBeCloseTo(0, 4);
    expect(y).toBeCloseTo(0, 6);
  });

  it("zSin(0, 0) -> (0, 0)", () => {
    const [x, y] = TRANSFORMS.zSin.fn(0, 0);
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("zCos(0, 0) -> (1, 0)", () => {
    const [x, y] = TRANSFORMS.zCos.fn(0, 0);
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("mobius identity (a=1,b=0,c=0,d=1) -> same point", () => {
    const [x, y] = TRANSFORMS.mobius.fn(2, 3, { a: 1, b: 0, c: 0, d: 1 });
    expect(x).toBeCloseTo(2, 4);
    expect(y).toBeCloseTo(3, 4);
  });

  it("zPlusC(1, 2, {cx: 0.5, cy: -1}) -> (1.5, 1)", () => {
    const [x, y] = TRANSFORMS.zPlusC.fn(1, 2, { cx: 0.5, cy: -1 });
    expect(x).toBeCloseTo(1.5, 6);
    expect(y).toBeCloseTo(1, 6);
  });

  it("spiral(1, 0, {k: 0}) -> (1, 0)", () => {
    const [x, y] = TRANSFORMS.spiral.fn(1, 0, { k: 0 });
    expect(x).toBeCloseTo(1, 4);
    expect(y).toBeCloseTo(0, 4);
  });

  it("pinch(1, 0, {p: 1}) -> (1, 0)  (identity exponent)", () => {
    const [x, y] = TRANSFORMS.pinch.fn(1, 0, { p: 1 });
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("swirl(0, 0, {k: 2, sigma: 0.4}) -> (0, 0)  (origin stable)", () => {
    const [x, y] = TRANSFORMS.swirl.fn(0, 0, { k: 2, sigma: 0.4 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("radialSine at origin -> (0, 0)", () => {
    const [x, y] = TRANSFORMS.radialSine.fn(0, 0, { amp: 0.1, freq: 5, phase: 0 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("angleWarp(1, 0, {amp: 0, k: 3}) -> (1, 0)  (zero amplitude)", () => {
    const [x, y] = TRANSFORMS.angleWarp.fn(1, 0, { amp: 0, k: 3 });
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("sineWarp(0, 0, {amp: 0.15, freq: 4, phase: 0}) -> (0, 0)  (sin(0)=0)", () => {
    const [x, y] = TRANSFORMS.sineWarp.fn(0, 0, { amp: 0.15, freq: 4, phase: 0 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("ripple at origin -> (0, 0)", () => {
    const [x, y] = TRANSFORMS.ripple.fn(0, 0, { amp: 0.1, freq: 6, phase: 0 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("reflect(1, 0, {theta: 0}) -> (1, 0)  (reflect across x-axis)", () => {
    const [x, y] = TRANSFORMS.reflect.fn(1, 0, { theta: 0 });
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(0, 6);
  });

  it("reflect(0, 1, {theta: 0}) -> (0, -1)", () => {
    const [x, y] = TRANSFORMS.reflect.fn(0, 1, { theta: 0 });
    expect(x).toBeCloseTo(0, 6);
    expect(y).toBeCloseTo(-1, 6);
  });

  it("abs(-3, -4) -> (3, 4)", () => {
    const [x, y] = TRANSFORMS.abs.fn(-3, -4);
    expect(x).toBe(3);
    expect(y).toBe(4);
  });

  it("mixed(1, 1) -> (1, -1)", () => {
    const [x, y] = TRANSFORMS.mixed.fn(1, 1);
    expect(x).toBe(1);
    expect(y).toBe(-1);
  });

  it("blend(2, 3, {amount: 0.5}, 0, 0) -> (1, 1.5)", () => {
    const [x, y] = TRANSFORMS.blend.fn(2, 3, { amount: 0.5 }, 0, 0);
    expect(x).toBeCloseTo(1, 6);
    expect(y).toBeCloseTo(1.5, 6);
  });
});

// ============================================================
// Individual displacement tests
// ============================================================

describe("DISPLACEMENTS - individual lift operations", () => {
  it("zMagnitude(3, 4, {amp: 1}) -> [0, 0, 5]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zMagnitude.fn(3, 4, { amp: 1 });
    expect(dx).toBe(0);
    expect(dy).toBe(0);
    expect(dz).toBeCloseTo(5, 6);
  });

  it("zGaussian(0, 0, {amp: 1, sigma: 1}) -> [0, 0, 1]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zGaussian.fn(0, 0, { amp: 1, sigma: 1 });
    expect(dx).toBe(0);
    expect(dy).toBe(0);
    expect(dz).toBeCloseTo(1, 6);
  });

  it("zDisplacement at identity -> [0, 0, 0]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zDisplacement.fn(1, 0, { amp: 1 }, 1, 0);
    expect(dz).toBeCloseTo(0, 6);
  });

  it("zReal(2, 3, {amp: 1}) -> [0, 0, 2]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zReal.fn(2, 3, { amp: 1 });
    expect(dz).toBeCloseTo(2, 6);
  });

  it("zImag(2, 3, {amp: 1}) -> [0, 0, 3]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zImag.fn(2, 3, { amp: 1 });
    expect(dz).toBeCloseTo(3, 6);
  });

  it("zArg(1, 0, {amp: 1}) -> [0, 0, 0]  (angle of (1,0) = 0)", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zArg.fn(1, 0, { amp: 1 });
    expect(dz).toBeCloseTo(0, 6);
  });

  it("zLogJac with det J = 1 -> [0, 0, ~log(1)]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zLogJac.fn(0, 0, { amp: 1 }, 0, 0, 1);
    expect(dz).toBeCloseTo(0, 4); // log(1 + 1e-6) ~ 0
  });

  it("zSignedJac(0, 0, {amp: 1}, 0, 0, 4) -> [0, 0, 4]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zSignedJac.fn(0, 0, { amp: 1 }, 0, 0, 4);
    expect(dz).toBeCloseTo(4, 6);
  });

  it("zSineWave at origin with phase=0 -> [0, 0, 0]  (sin(0)=0)", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zSineWave.fn(0, 0, { amp: 1, freq: 4, phase: 0 });
    expect(dz).toBeCloseTo(0, 6);
  });

  it("zSineXY at origin -> [0, 0, 0]  (sin(0)*sin(0)=0)", () => {
    const [dx, dy, dz] = DISPLACEMENTS.zSineXY.fn(0, 0, { amp: 1, fx: 3, fy: 3 });
    expect(dz).toBeCloseTo(0, 6);
  });

  it("conical(0, 0, {amp: 1}) -> [0, 0, 0]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.conical.fn(0, 0, { amp: 1 });
    expect(dz).toBeCloseTo(0, 6);
  });

  it("conical(3, 4, {amp: 1}) -> [0, 0, 5]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.conical.fn(3, 4, { amp: 1 });
    expect(dz).toBeCloseTo(5, 6);
  });

  it("helical at origin -> [0, 0, 0]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.helical.fn(0, 0, { lift: 0.5, twist: 2 });
    expect(dx).toBeCloseTo(0, 6);
    expect(dy).toBeCloseTo(0, 6);
    expect(dz).toBeCloseTo(0, 6);
  });

  it("spherical at origin -> [0, 0, 0]", () => {
    const [dx, dy, dz] = DISPLACEMENTS.spherical.fn(0, 0, { r: 1 });
    expect(dx).toBeCloseTo(0, 6);
    expect(dy).toBeCloseTo(0, 6);
    expect(dz).toBeCloseTo(0, 6);
  });
});

// ============================================================
// Pipeline composition tests
// ============================================================

describe("pipeline composition", () => {
  it("single layer with blend=1: output matches transform", () => {
    const layer = makeLayer("scale");
    layer.params.sx.value = 2;
    layer.params.sy.value = 3;
    const [x, y] = applyPipeline(1, 1, [layer]);
    expect(x).toBeCloseTo(2, 6);
    expect(y).toBeCloseTo(3, 6);
  });

  it("single layer with blend=0.5: output is lerp(identity, transformed)", () => {
    const layer = makeLayer("scale");
    layer.params.sx.value = 3;
    layer.params.sy.value = 3;
    layer.blend.value = 0.5;
    const [x, y] = applyPipeline(1, 1, [layer]);
    // lerp(1, 3, 0.5) = 2
    expect(x).toBeCloseTo(2, 6);
    expect(y).toBeCloseTo(2, 6);
  });

  it("multi-layer non-commutativity: scale-then-translate != translate-then-scale", () => {
    const scaleLayer = makeLayer("scale");
    scaleLayer.params.sx.value = 2;
    scaleLayer.params.sy.value = 2;

    const translateLayer = makeLayer("translate");
    translateLayer.params.tx.value = 1;
    translateLayer.params.ty.value = 0;

    const [x1, y1] = applyPipeline(1, 0, [scaleLayer, translateLayer]);
    const [x2, y2] = applyPipeline(1, 0, [translateLayer, scaleLayer]);

    // scale then translate: (1,0) -> (2,0) -> (3,0)
    // translate then scale: (1,0) -> (2,0) -> (4,0)
    expect(x1).not.toBeCloseTo(x2, 6);
  });

  it("disabled layer (on: false) is skipped", () => {
    const layer = makeLayer("scale");
    layer.params.sx.value = 5;
    layer.params.sy.value = 5;
    layer.on = false;
    const [x, y] = applyPipeline(1, 1, [layer]);
    expect(x).toBe(1);
    expect(y).toBe(1);
  });

  it("pre-flattened equivalence: preFlattenLayers + applyPipelineFlat == applyPipeline", () => {
    const layer1 = makeLayer("rotate");
    layer1.params.theta.value = Math.PI / 3;
    const layer2 = makeLayer("scale");
    layer2.params.sx.value = 1.5;
    layer2.params.sy.value = 0.8;

    const layers = [layer1, layer2];
    const [x1, y1] = applyPipeline(1, 1, layers);

    const flat = preFlattenLayers(layers);
    const [x2, y2] = applyPipelineFlat(1, 1, flat);

    expect(x2).toBeCloseTo(x1, 10);
    expect(y2).toBeCloseTo(y1, 10);
  });

  it("displacement pipeline with pre-flattened equivalence", () => {
    const layer = makeDispLayer("zMagnitude");
    layer.params.amp.value = 1;

    const layers = [layer];
    const jFn = () => 1;

    const [dx1, dy1, dz1] = applyDispPipeline(3, 4, 0, 0, layers, jFn);
    const flat = preFlattenDispLayers(layers);
    const [dx2, dy2, dz2] = applyDispPipelineFlat(3, 4, 0, 0, flat, jFn);

    expect(dx2).toBeCloseTo(dx1, 10);
    expect(dy2).toBeCloseTo(dy1, 10);
    expect(dz2).toBeCloseTo(dz1, 10);
  });
});

// ============================================================
// Jacobian tests (numerical)
// ============================================================

describe("Jacobian (numerical finite differences)", () => {
  const h = 1e-5;

  function numericalJacobian(fn, x, y, params) {
    const [fx, fy] = fn(x, y, params);
    const [fxh, fyh] = fn(x + h, y, params);
    const [fxk, fyk] = fn(x, y + h, params);
    const dfdx = (fxh - fx) / h;
    const dfdy = (fxk - fx) / h;
    const dgdx = (fyh - fy) / h;
    const dgdy = (fyk - fy) / h;
    return dfdx * dgdy - dfdy * dgdx; // det J
  }

  it("identity transform (translate with 0,0): det J = 1.0", () => {
    const det = numericalJacobian(TRANSFORMS.translate.fn, 1, 1, { tx: 0, ty: 0 });
    expect(det).toBeCloseTo(1, 4);
  });

  it("scale(2, 2): det J = 4.0", () => {
    const det = numericalJacobian(TRANSFORMS.scale.fn, 1, 1, { sx: 2, sy: 2 });
    expect(det).toBeCloseTo(4, 4);
  });

  it("rotate(theta): det J = 1.0 (conformal, area-preserving)", () => {
    const det = numericalJacobian(TRANSFORMS.rotate.fn, 1, 1, { theta: Math.PI / 4 });
    expect(det).toBeCloseTo(1, 4);
  });

  it("scale(3, 2): det J = 6.0", () => {
    const det = numericalJacobian(TRANSFORMS.scale.fn, 1, 1, { sx: 3, sy: 2 });
    expect(det).toBeCloseTo(6, 4);
  });
});

// ============================================================
// Composition string tests
// ============================================================

describe("compositionString", () => {
  it("empty pipeline returns 'L'", () => {
    const result = compositionString([], [], {});
    expect(result).toBe("L");
  });

  it("single operation", () => {
    const layer = makeLayer("rotate");
    const result = compositionString([layer], [], {});
    expect(result).toBe("R(L)");
  });

  it("two operations composed right-to-left", () => {
    const r = makeLayer("rotate");
    const s = makeLayer("scale");
    // pipeline order is [rotate, scale]; compositionString reverses for math notation
    const result = compositionString([r, s], [], {});
    expect(result).toBe("S . R(L)");
  });

  it("single displacement", () => {
    const d = makeDispLayer("zMagnitude");
    const result = compositionString([], [d], {});
    expect(result).toBe("L + |T| -> z");
  });

  it("disabled layers excluded", () => {
    const layer = makeLayer("rotate");
    layer.on = false;
    const result = compositionString([layer], [], {});
    expect(result).toBe("L");
  });
});

// ============================================================
// DOF counting tests
// ============================================================

describe("countDOFs", () => {
  it("empty pipeline: struct=2, temp=0", () => {
    const result = countDOFs([], [], []);
    expect(result.struct).toBe(2);
    expect(result.temp).toBe(0);
  });

  it("single 1-param layer: struct increases by 2 (1 param + 1 blend)", () => {
    const layer = makeLayer("zPow"); // 1 param: n
    const result = countDOFs([layer], [], []);
    // base=2, + params.length(1) + blend(1) = 4
    expect(result.struct).toBe(4);
    expect(result.temp).toBe(0);
  });

  it("single 2-param layer: struct increases by 3 (2 params + 1 blend)", () => {
    const layer = makeLayer("scale"); // 2 params: sx, sy
    const result = countDOFs([layer], [], []);
    // base=2, + params.length(2) + blend(1) = 5
    expect(result.struct).toBe(5);
    expect(result.temp).toBe(0);
  });

  it("animated param adds to temp", () => {
    const layer = makeLayer("rotate");
    layer.params.theta.anim = true;
    const result = countDOFs([layer], [], []);
    expect(result.temp).toBe(1);
  });

  it("disabled layer not counted", () => {
    const layer = makeLayer("rotate");
    layer.on = false;
    const result = countDOFs([layer], [], []);
    expect(result.struct).toBe(2);
    expect(result.temp).toBe(0);
  });
});

// ============================================================
// Layer ID tests
// ============================================================

describe("generateLayerId", () => {
  it("returns 8 characters", () => {
    const id = generateLayerId();
    expect(id.length).toBe(8);
  });

  it("two calls return different values", () => {
    const id1 = generateLayerId();
    const id2 = generateLayerId();
    expect(id1).not.toBe(id2);
  });
});

// ============================================================
// Registry structure tests
// ============================================================

describe("registry structure", () => {
  it("all TRANSFORMS have fn, params, label", () => {
    for (const [key, def] of Object.entries(TRANSFORMS)) {
      expect(typeof def.fn).toBe("function");
      expect(Array.isArray(def.params)).toBe(true);
      expect(typeof def.label).toBe("string");
    }
  });

  it("all DISPLACEMENTS have fn, params, label", () => {
    for (const [key, def] of Object.entries(DISPLACEMENTS)) {
      expect(typeof def.fn).toBe("function");
      expect(Array.isArray(def.params)).toBe(true);
      expect(typeof def.label).toBe("string");
    }
  });

  it("REGISTRY has entries for all transforms and displacements", () => {
    for (const key of Object.keys(TRANSFORMS)) {
      expect(REGISTRY.operations[key]).toBeDefined();
    }
    for (const key of Object.keys(DISPLACEMENTS)) {
      expect(REGISTRY.lift[key]).toBeDefined();
    }
  });

  it("TRANSFORMS has 25 entries", () => {
    expect(Object.keys(TRANSFORMS).length).toBe(25);
  });

  it("DISPLACEMENTS has 15 entries", () => {
    expect(Object.keys(DISPLACEMENTS).length).toBe(15);
  });
});
