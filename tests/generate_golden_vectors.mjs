// Generate golden test vectors from the JS registries.
// Run: node --experimental-vm-modules tests/generate_golden_vectors.mjs

import { createRequire } from "module";
import { writeFileSync } from "fs";

// registries.js uses `export` syntax in a CJS package, so we load it via dynamic import
// with a data: URL workaround, or just inline read+eval.
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Evaluate registries.js as a module by reading the source and using Function constructor
const registrySrc = readFileSync(join(__dirname, "..", "web", "registries.js"), "utf-8");

// Strip export keywords and evaluate
const strippedSrc = registrySrc
  .replace(/^export\s+const\s+/gm, "const ")
  .replace(/^export\s+function\s+/gm, "function ");

const moduleExports = {};
const wrappedFn = new Function(
  "module", "exports", "Math",
  strippedSrc + `
  exports.TRANSFORMS = TRANSFORMS;
  exports.DISPLACEMENTS = DISPLACEMENTS;
`
);
wrappedFn({ exports: moduleExports }, moduleExports, Math);
const { TRANSFORMS, DISPLACEMENTS } = moduleExports;

const vectors = { operations: [], lifts: [] };

// Operation test vectors: each entry has { key, x, y, params, x0, y0, expected: [ex, ey] }
const opTests = [
  { key: "translate", x: 1, y: 2, params: { tx: 0.5, ty: -0.3 } },
  { key: "scale", x: 1, y: 1, params: { sx: 2, sy: 3 } },
  { key: "rotate", x: 1, y: 0, params: { theta: Math.PI / 2 } },
  { key: "shear", x: 1, y: 1, params: { kx: 0.5, ky: 0.3 } },
  { key: "matrix", x: 2, y: 3, params: { a: 1, b: 2, c: 3, d: 4 } },
  { key: "zPow", x: 0, y: 1, params: { n: 2 } },
  { key: "zPow", x: 1, y: 1, params: { n: 3 } },
  { key: "zInv", x: 1, y: 0, params: {} },
  { key: "zInv", x: 0.5, y: 0.5, params: {} },
  { key: "zExp", x: 0, y: 0, params: {} },
  { key: "zExp", x: 1, y: Math.PI / 4, params: {} },
  { key: "zLog", x: 1, y: 0, params: {} },
  { key: "zLog", x: Math.E, y: 0, params: {} },
  { key: "zSin", x: 0, y: 0, params: {} },
  { key: "zSin", x: Math.PI / 2, y: 0, params: {} },
  { key: "zCos", x: 0, y: 0, params: {} },
  { key: "zCos", x: Math.PI, y: 0, params: {} },
  { key: "mobius", x: 2, y: 3, params: { a: 1, b: 0, c: 0, d: 1 } },
  { key: "mobius", x: 1, y: 0, params: { a: 2, b: 1, c: 1, d: 2 } },
  { key: "zPlusC", x: 1, y: 2, params: { cx: 0.5, cy: -1 } },
  { key: "spiral", x: 1, y: 0, params: { k: 0 } },
  { key: "spiral", x: 1, y: 1, params: { k: 2 } },
  { key: "pinch", x: 1, y: 0, params: { p: 1 } },
  { key: "pinch", x: 2, y: 0, params: { p: 0.5 } },
  { key: "swirl", x: 0, y: 0, params: { k: 2, sigma: 0.4 } },
  { key: "swirl", x: 1, y: 0, params: { k: 3, sigma: 1 } },
  { key: "radialSine", x: 0, y: 0, params: { amp: 0.1, freq: 5, phase: 0 } },
  { key: "radialSine", x: 1, y: 0, params: { amp: 0.2, freq: 3, phase: 0.5 } },
  { key: "angleWarp", x: 1, y: 0, params: { amp: 0, k: 3 } },
  { key: "angleWarp", x: 1, y: 1, params: { amp: 0.5, k: 2 } },
  { key: "sineWarp", x: 0, y: 0, params: { amp: 0.15, freq: 4, phase: 0 } },
  { key: "sineWarp", x: 1, y: 0.5, params: { amp: 0.1, freq: 3, phase: 1 } },
  { key: "ripple", x: 0, y: 0, params: { amp: 0.1, freq: 6, phase: 0 } },
  { key: "ripple", x: 1, y: 1, params: { amp: 0.2, freq: 4, phase: 0.5 } },
  { key: "conjugate", x: 1, y: 2, params: {} },
  { key: "reflect", x: 1, y: 0, params: { theta: 0 } },
  { key: "reflect", x: 0, y: 1, params: { theta: Math.PI / 4 } },
  { key: "abs", x: -3, y: -4, params: {} },
  { key: "mixed", x: 1, y: 1, params: {} },
  { key: "mixed", x: 2, y: 3, params: {} },
  { key: "blend", x: 2, y: 3, params: { amount: 0.5 }, x0: 0, y0: 0 },
  { key: "blend", x: 4, y: 6, params: { amount: 0.25 }, x0: 2, y0: 2 },
];

for (const t of opTests) {
  const def = TRANSFORMS[t.key];
  const x0 = t.x0 ?? 0;
  const y0 = t.y0 ?? 0;
  const result = def.needsOrigin
    ? def.fn(t.x, t.y, t.params, x0, y0)
    : def.fn(t.x, t.y, t.params);
  vectors.operations.push({
    key: t.key,
    x: t.x, y: t.y,
    params: t.params,
    x0, y0,
    expected: result,
  });
}

// Lift test vectors: { key, x, y, params, x0, y0, jSigned, expected: [dx, dy, dz] }
const liftTests = [
  { key: "zMagnitude", x: 3, y: 4, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zMagnitude", x: 1, y: 1, params: { amp: 2 }, x0: 0, y0: 0, j: 0 },
  { key: "zDisplacement", x: 1, y: 0, params: { amp: 1 }, x0: 1, y0: 0, j: 0 },
  { key: "zDisplacement", x: 2, y: 0, params: { amp: 1 }, x0: 1, y0: 0, j: 0 },
  { key: "zLogJac", x: 0, y: 0, params: { amp: 1 }, x0: 0, y0: 0, j: 1 },
  { key: "zLogJac", x: 0, y: 0, params: { amp: 1 }, x0: 0, y0: 0, j: 4 },
  { key: "zSignedJac", x: 0, y: 0, params: { amp: 1 }, x0: 0, y0: 0, j: 4 },
  { key: "zSignedJac", x: 0, y: 0, params: { amp: 0.5 }, x0: 0, y0: 0, j: -2 },
  { key: "zReal", x: 2, y: 3, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zImag", x: 2, y: 3, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zArg", x: 1, y: 0, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zArg", x: 0, y: 1, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zSineWave", x: 0, y: 0, params: { amp: 1, freq: 4, phase: 0 }, x0: 0, y0: 0, j: 0 },
  { key: "zSineWave", x: 1, y: 0, params: { amp: 0.5, freq: 3, phase: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zSineXY", x: 0, y: 0, params: { amp: 1, fx: 3, fy: 3 }, x0: 0, y0: 0, j: 0 },
  { key: "zSineXY", x: 1, y: 1, params: { amp: 2, fx: 2, fy: 3 }, x0: 0, y0: 0, j: 0 },
  { key: "zGaussian", x: 0, y: 0, params: { amp: 1, sigma: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "zGaussian", x: 1, y: 1, params: { amp: 2, sigma: 0.5 }, x0: 0, y0: 0, j: 0 },
  { key: "spherical", x: 0, y: 0, params: { r: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "spherical", x: 1, y: 0, params: { r: 2 }, x0: 0, y0: 0, j: 0 },
  { key: "conical", x: 0, y: 0, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "conical", x: 3, y: 4, params: { amp: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "helical", x: 0, y: 0, params: { lift: 0.5, twist: 2 }, x0: 0, y0: 0, j: 0 },
  { key: "helical", x: 1, y: 0, params: { lift: 1, twist: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "noise3D", x: 1, y: 1, params: { amp: 0.3, freq: 3, seed: 0 }, x0: 0, y0: 0, j: 0 },
  { key: "noise3D", x: 0.5, y: -0.5, params: { amp: 1, freq: 2, seed: 1 }, x0: 0, y0: 0, j: 0 },
  { key: "ripple3D", x: 1, y: 0, params: { amp: 0.2, freq: 6, phase: 0 }, x0: 0, y0: 0, j: 0 },
  { key: "ripple3D", x: 1, y: 1, params: { amp: 0.5, freq: 4, phase: 1 }, x0: 0, y0: 0, j: 0 },
];

for (const t of liftTests) {
  const def = DISPLACEMENTS[t.key];
  const result = def.fn(t.x, t.y, t.params, t.x0, t.y0, t.j);
  vectors.lifts.push({
    key: t.key,
    x: t.x, y: t.y,
    params: t.params,
    x0: t.x0, y0: t.y0,
    j_signed: t.j,
    expected: result,
  });
}

writeFileSync(
  "tests/fixtures/golden_vectors.json",
  JSON.stringify(vectors, null, 2) + "\n"
);

console.log(`Generated ${vectors.operations.length} operation vectors + ${vectors.lifts.length} lift vectors`);
