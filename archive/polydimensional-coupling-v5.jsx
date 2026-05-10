import { useState, useRef, useEffect } from "react";
import * as THREE from "three";

// ============================================================
// TRANSFORM REGISTRY
// ============================================================

const TRANSFORMS = {
  // ----- linear / affine -----
  translate: {
    label: "translate",
    category: "linear",
    note: "shift by (tx, ty)",
    params: [
      { key: "tx", default: 0.1, min: -1, max: 1 },
      { key: "ty", default: 0.0, min: -1, max: 1 },
    ],
    fn: (x, y, p) => [x + p.tx, y + p.ty],
  },
  scale: {
    label: "scale",
    category: "linear",
    note: "anisotropic scale (sx, sy)",
    params: [
      { key: "sx", default: 1.2, min: 0.1, max: 3 },
      { key: "sy", default: 1.0, min: 0.1, max: 3 },
    ],
    fn: (x, y, p) => [x * p.sx, y * p.sy],
  },
  rotate: {
    label: "rotate",
    category: "linear",
    note: "rotate by theta",
    params: [
      { key: "theta", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => {
      const c = Math.cos(p.theta), s = Math.sin(p.theta);
      return [x * c - y * s, x * s + y * c];
    },
  },
  shear: {
    label: "shear",
    category: "linear",
    note: "shear (kx, ky)",
    params: [
      { key: "kx", default: 0.3, min: -2, max: 2 },
      { key: "ky", default: 0, min: -2, max: 2 },
    ],
    fn: (x, y, p) => [x + p.kx * y, y + p.ky * x],
  },
  matrix: {
    label: "2x2 matrix",
    category: "linear",
    note: "general linear map [[a,b],[c,d]]",
    params: [
      { key: "a", default: 1, min: -3, max: 3 },
      { key: "b", default: 0, min: -3, max: 3 },
      { key: "c", default: 0, min: -3, max: 3 },
      { key: "d", default: 1, min: -3, max: 3 },
    ],
    fn: (x, y, p) => [p.a * x + p.b * y, p.c * x + p.d * y],
  },

  // ----- complex / conformal -----
  zPow: {
    label: "z^n",
    category: "complex",
    note: "complex power. n controls fold count",
    params: [
      { key: "n", default: 2, min: 0.1, max: 5 },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      const t = Math.atan2(y, x);
      const rn = Math.pow(r, p.n);
      return [rn * Math.cos(p.n * t), rn * Math.sin(p.n * t)];
    },
  },
  zInv: {
    label: "1/z",
    category: "complex",
    note: "mobius inversion. spike at origin",
    params: [],
    fn: (x, y) => {
      const d = x * x + y * y + 1e-6;
      return [x / d, -y / d];
    },
  },
  zExp: {
    label: "exp(z)",
    category: "complex",
    note: "verticals -> circles, horizontals -> rays",
    params: [],
    fn: (x, y) => [Math.exp(x) * Math.cos(y), Math.exp(x) * Math.sin(y)],
  },
  zLog: {
    label: "log(z)",
    category: "complex",
    note: "inverse of exp. branch cut at -x axis",
    params: [],
    fn: (x, y) => {
      const r = Math.hypot(x, y) + 1e-6;
      return [Math.log(r), Math.atan2(y, x)];
    },
  },
  zSin: {
    label: "sin(z)",
    category: "complex",
    note: "periodic in x, exponential in y",
    params: [],
    fn: (x, y) => [Math.sin(x) * Math.cosh(y), Math.cos(x) * Math.sinh(y)],
  },
  zCos: {
    label: "cos(z)",
    category: "complex",
    note: "periodic in x, exponential in y",
    params: [],
    fn: (x, y) => [Math.cos(x) * Math.cosh(y), -Math.sin(x) * Math.sinh(y)],
  },
  mobius: {
    label: "mobius (az+b)/(cz+d)",
    category: "complex",
    note: "linear fractional transform",
    params: [
      { key: "a", default: 1, min: -2, max: 2 },
      { key: "b", default: 0, min: -2, max: 2 },
      { key: "c", default: 0.3, min: -2, max: 2 },
      { key: "d", default: 1, min: -2, max: 2 },
    ],
    fn: (x, y, p) => {
      const nx = p.a * x + p.b;
      const ny = p.a * y;
      const dx = p.c * x + p.d;
      const dy = p.c * y;
      const den = dx * dx + dy * dy + 1e-6;
      return [(nx * dx + ny * dy) / den, (ny * dx - nx * dy) / den];
    },
  },
  zPlusC: {
    label: "z + c",
    category: "complex",
    note: "complex offset (julia-style)",
    params: [
      { key: "cx", default: 0.3, min: -2, max: 2 },
      { key: "cy", default: 0.2, min: -2, max: 2 },
    ],
    fn: (x, y, p) => [x + p.cx, y + p.cy],
  },

  // ----- polar / radial -----
  spiral: {
    label: "spiral",
    category: "polar",
    note: "rotate by k*log(r)",
    params: [
      { key: "k", default: 1, min: -5, max: 5 },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y) + 1e-6;
      const t = Math.atan2(y, x) + p.k * Math.log(r);
      return [r * Math.cos(t), r * Math.sin(t)];
    },
  },
  pinch: {
    label: "pinch / bulge",
    category: "polar",
    note: "r -> r^p. p<1 bulges, p>1 pinches",
    params: [
      { key: "p", default: 1.5, min: 0.2, max: 4 },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      if (r < 1e-6) return [x, y];
      const t = Math.atan2(y, x);
      const nr = Math.pow(r, p.p);
      return [nr * Math.cos(t), nr * Math.sin(t)];
    },
  },
  swirl: {
    label: "swirl",
    category: "polar",
    note: "angle += k*exp(-r^2/sigma^2)",
    params: [
      { key: "k", default: 2, min: -8, max: 8 },
      { key: "sigma", default: 0.4, min: 0.05, max: 2 },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      const t0 = Math.atan2(y, x);
      const t = t0 + p.k * Math.exp(-(r * r) / (p.sigma * p.sigma));
      return [r * Math.cos(t), r * Math.sin(t)];
    },
  },
  radialSine: {
    label: "radial sine",
    category: "polar",
    note: "r += amp*sin(freq*r + phase)",
    params: [
      { key: "amp", default: 0.1, min: 0, max: 0.5 },
      { key: "freq", default: 5, min: 0.5, max: 20 },
      { key: "phase", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      if (r < 1e-6) return [x, y];
      const t = Math.atan2(y, x);
      const nr = r + p.amp * Math.sin(p.freq * r + p.phase);
      return [nr * Math.cos(t), nr * Math.sin(t)];
    },
  },
  angleWarp: {
    label: "angle warp",
    category: "polar",
    note: "theta -> theta + amp*sin(k*theta)",
    params: [
      { key: "amp", default: 0.3, min: 0, max: 2 },
      { key: "k", default: 3, min: 1, max: 12 },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      const t0 = Math.atan2(y, x);
      const t = t0 + p.amp * Math.sin(p.k * t0);
      return [r * Math.cos(t), r * Math.sin(t)];
    },
  },

  // ----- wave / periodic -----
  sineWarp: {
    label: "sine warp",
    category: "wave",
    note: "x += amp*sin(freq*y), y += amp*sin(freq*x)",
    params: [
      { key: "amp", default: 0.15, min: 0, max: 0.6 },
      { key: "freq", default: 4, min: 0.5, max: 20 },
      { key: "phase", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => [
      x + p.amp * Math.sin(p.freq * y + p.phase),
      y + p.amp * Math.sin(p.freq * x + p.phase),
    ],
  },
  ripple: {
    label: "ripple",
    category: "wave",
    note: "radial sine displacement",
    params: [
      { key: "amp", default: 0.1, min: 0, max: 0.5 },
      { key: "freq", default: 6, min: 0.5, max: 20 },
      { key: "phase", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      const t = Math.atan2(y, x);
      const d = p.amp * Math.sin(p.freq * r + p.phase);
      return [(r + d) * Math.cos(t), (r + d) * Math.sin(t)];
    },
  },

  // ----- non-holomorphic -----
  conjugate: {
    label: "conjugate",
    category: "non-holo",
    note: "(x, y) -> (x, -y)",
    params: [],
    fn: (x, y) => [x, -y],
  },
  reflect: {
    label: "reflect",
    category: "non-holo",
    note: "reflect across line at angle theta",
    params: [
      { key: "theta", default: 0, min: 0, max: Math.PI, periodic: true },
    ],
    fn: (x, y, p) => {
      const c = Math.cos(2 * p.theta), s = Math.sin(2 * p.theta);
      return [x * c + y * s, x * s - y * c];
    },
  },
  abs: {
    label: "|z|",
    category: "non-holo",
    note: "fold to upper-right quadrant",
    params: [],
    fn: (x, y) => [Math.abs(x), Math.abs(y)],
  },
  mixed: {
    label: "mixed monomials",
    category: "non-holo",
    note: "[x^2 y^3, -x^3 y^2]",
    params: [],
    fn: (x, y) => [x * x * y * y * y, -x * x * x * y * y],
  },

  // ----- utility -----
  blend: {
    label: "blend with input",
    category: "utility",
    note: "mix current state with pre-pipeline grid",
    params: [
      { key: "amount", default: 0.5, min: 0, max: 1 },
    ],
    fn: (x, y, p, x0, y0) => [
      x0 * (1 - p.amount) + x * p.amount,
      y0 * (1 - p.amount) + y * p.amount,
    ],
    needsOrigin: true,
  },
};

const CATEGORIES = ["linear", "complex", "polar", "wave", "non-holo", "utility"];

// ============================================================
// DISPLACEMENT REGISTRY (Pipeline 2: 3D)
// fn signature: (x, y, p, x0, y0, jSigned) => [dx, dy, dz]
// (x, y) are post-XY-pipeline coordinates, (x0, y0) original
// ============================================================

const DISPLACEMENTS = {
  // ----- height-style (z only) -----
  zMagnitude: {
    label: "|.| -> z",
    category: "height",
    note: "magnitude of post-pipeline position pushes up",
    params: [{ key: "amp", default: 1, min: -2, max: 2 }],
    fn: (x, y, p) => [0, 0, p.amp * Math.hypot(x, y)],
  },
  zDisplacement: {
    label: "|T|-|z| -> z",
    category: "height",
    note: "expansion-contraction signed displacement",
    params: [{ key: "amp", default: 1, min: -2, max: 2 }],
    fn: (x, y, p, x0, y0) => [0, 0, p.amp * (Math.hypot(x, y) - Math.hypot(x0, y0))],
  },
  zLogJac: {
    label: "log|det J| -> z",
    category: "height",
    note: "log area expansion. needs jacobian",
    params: [{ key: "amp", default: 0.5, min: -2, max: 2 }],
    needsJacobian: true,
    fn: (x, y, p, x0, y0, jSigned) => [0, 0, p.amp * Math.log(Math.abs(jSigned) + 1e-6)],
  },
  zSignedJac: {
    label: "det J -> z",
    category: "height",
    note: "signed jacobian. negative when orientation flips",
    params: [{ key: "amp", default: 0.3, min: -2, max: 2 }],
    needsJacobian: true,
    fn: (x, y, p, x0, y0, jSigned) => [0, 0, p.amp * jSigned],
  },
  zReal: {
    label: "Re(T) -> z",
    category: "height",
    note: "real component as z",
    params: [{ key: "amp", default: 1, min: -2, max: 2 }],
    fn: (x, y, p) => [0, 0, p.amp * x],
  },
  zImag: {
    label: "Im(T) -> z",
    category: "height",
    note: "imag component as z",
    params: [{ key: "amp", default: 1, min: -2, max: 2 }],
    fn: (x, y, p) => [0, 0, p.amp * y],
  },
  zArg: {
    label: "arg(T) -> z",
    category: "height",
    note: "angle as z. branch cuts visible",
    params: [{ key: "amp", default: 0.3, min: -2, max: 2 }],
    fn: (x, y, p) => [0, 0, p.amp * Math.atan2(y, x)],
  },

  // ----- waves / bumps (z) -----
  zSineWave: {
    label: "sine wave (z)",
    category: "wave",
    note: "z = amp*sin(freq*r + phase)",
    params: [
      { key: "amp", default: 0.5, min: -2, max: 2 },
      { key: "freq", default: 4, min: 0.1, max: 20 },
      { key: "phase", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => [0, 0, p.amp * Math.sin(p.freq * Math.hypot(x, y) + p.phase)],
  },
  zSineXY: {
    label: "sine xy (z)",
    category: "wave",
    note: "z = amp*sin(fx*x)*sin(fy*y)",
    params: [
      { key: "amp", default: 0.5, min: -2, max: 2 },
      { key: "fx", default: 3, min: 0.1, max: 15 },
      { key: "fy", default: 3, min: 0.1, max: 15 },
    ],
    fn: (x, y, p) => [0, 0, p.amp * Math.sin(p.fx * x) * Math.sin(p.fy * y)],
  },
  zGaussian: {
    label: "gaussian bump (z)",
    category: "wave",
    note: "z = amp*exp(-r^2/sigma^2)",
    params: [
      { key: "amp", default: 1, min: -2, max: 2 },
      { key: "sigma", default: 0.5, min: 0.05, max: 3 },
    ],
    fn: (x, y, p) => [0, 0, p.amp * Math.exp(-(x * x + y * y) / (p.sigma * p.sigma))],
  },

  // ----- 3D displacement (full vector) -----
  spherical: {
    label: "spherical projection",
    category: "3d",
    note: "wrap onto sphere of radius r",
    params: [{ key: "r", default: 1, min: 0.1, max: 3 }],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y);
      const t = Math.atan2(y, x);
      const phi = Math.atan(r / p.r);
      const sx = p.r * Math.sin(phi) * Math.cos(t);
      const sy = p.r * Math.sin(phi) * Math.sin(t);
      const sz = p.r * Math.cos(phi);
      return [sx - x, sy - y, sz - p.r];
    },
  },
  conical: {
    label: "conical lift",
    category: "3d",
    note: "z = amp*r, a literal cone",
    params: [{ key: "amp", default: 0.8, min: -2, max: 2 }],
    fn: (x, y, p) => [0, 0, p.amp * Math.hypot(x, y)],
  },
  helical: {
    label: "helical twist",
    category: "3d",
    note: "rotate xy by k*z while lifting",
    params: [
      { key: "lift", default: 0.5, min: -2, max: 2 },
      { key: "twist", default: 2, min: -8, max: 8 },
    ],
    fn: (x, y, p) => {
      const z = p.lift * Math.hypot(x, y);
      const c = Math.cos(p.twist * z), s = Math.sin(p.twist * z);
      return [x * c - y * s - x, x * s + y * c - y, z];
    },
  },
  noise3D: {
    label: "noise displacement",
    category: "3d",
    note: "pseudo-random 3D wobble",
    params: [
      { key: "amp", default: 0.3, min: -1, max: 1 },
      { key: "freq", default: 3, min: 0.5, max: 15 },
      { key: "seed", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => {
      const nx = Math.sin(p.freq * x + 1.3 * p.seed) * Math.cos(p.freq * y * 1.7 + 2.1);
      const ny = Math.sin(p.freq * y + 2.7 * p.seed) * Math.cos(p.freq * x * 1.3 + 0.7);
      const nz = Math.sin(p.freq * (x + y) + 0.5 * p.seed) * Math.cos(p.freq * (x - y) * 1.1);
      return [p.amp * nx, p.amp * ny, p.amp * nz];
    },
  },
  ripple3D: {
    label: "ripple (z + xy)",
    category: "3d",
    note: "radial wave displacing all axes",
    params: [
      { key: "amp", default: 0.2, min: -1, max: 1 },
      { key: "freq", default: 6, min: 0.5, max: 20 },
      { key: "phase", default: 0, min: 0, max: Math.PI * 2, periodic: true },
    ],
    fn: (x, y, p) => {
      const r = Math.hypot(x, y) + 1e-6;
      const t = Math.atan2(y, x);
      const w = Math.sin(p.freq * r + p.phase);
      return [p.amp * w * Math.cos(t) * 0.3, p.amp * w * Math.sin(t) * 0.3, p.amp * w];
    },
  },
};

const DISP_CATEGORIES = ["height", "wave", "3d"];

const PHI = (1 + Math.sqrt(5)) / 2;
const SILVER = 1 + Math.sqrt(2);
const BRONZE = (3 + Math.sqrt(13)) / 2;
const PLASTIC = 1.32471795724474602596;

const PALETTES = {
  acid:  { bg: "#000", grid: "#7fff5a", warp: "#caff3a", glow: "#7fff5a" },
  cyan:  { bg: "#000", grid: "#5be3d4", warp: "#bff7ee", glow: "#5be3d4" },
  blush: { bg: "#000", grid: "#ff5e8a", warp: "#ffb3c8", glow: "#ff5e8a" },
  amber: { bg: "#0a0500", grid: "#ffb347", warp: "#fff1c2", glow: "#ffb347" },
  void:  { bg: "#0a0a12", grid: "#b39dff", warp: "#e8defc", glow: "#b39dff" },
};

const RATIO_GROUPS = [
  {
    label: "rational",
    presets: [
      { label: "1/4", value: 0.25 }, { label: "1/3", value: 1/3 },
      { label: "1/2", value: 0.5 }, { label: "2/3", value: 2/3 },
      { label: "1", value: 1 }, { label: "3/2", value: 1.5 },
      { label: "2", value: 2 }, { label: "3", value: 3 }, { label: "4", value: 4 },
    ],
  },
  {
    label: "metallic / irrational",
    presets: [
      { label: "1/phi^2", value: 1/(PHI*PHI) }, { label: "1/phi", value: 1/PHI },
      { label: "sqrt2/2", value: Math.SQRT2/2 }, { label: "1/pi", value: 1/Math.PI },
      { label: "1/e", value: 1/Math.E }, { label: "phi", value: PHI },
      { label: "sqrt2", value: Math.SQRT2 }, { label: "sqrt3", value: Math.sqrt(3) },
      { label: "e", value: Math.E }, { label: "pi", value: Math.PI },
      { label: "delta_s", value: SILVER }, { label: "delta_b", value: BRONZE },
      { label: "rho", value: PLASTIC },
    ],
  },
  {
    label: "fibonacci ratios",
    presets: [
      { label: "3/5", value: 3/5 }, { label: "5/8", value: 5/8 },
      { label: "8/13", value: 8/13 }, { label: "13/21", value: 13/21 },
      { label: "5/3", value: 5/3 }, { label: "8/5", value: 8/5 },
      { label: "13/8", value: 13/8 }, { label: "21/13", value: 21/13 },
    ],
  },
];

// ============================================================
// LAYER FACTORY + PIPELINE
// ============================================================

function makeLayer(type) {
  const def = TRANSFORMS[type];
  const params = {};
  for (const p of def.params) {
    params[p.key] = {
      value: p.default, default: p.default,
      anim: false, animMode: p.periodic ? "loop" : "sweep",
      ratio: 1,
      min: p.min, max: p.max, periodic: !!p.periodic,
    };
  }
  return {
    id: Math.random().toString(36).slice(2, 9),
    type, on: true, params,
  };
}

function makeDispLayer(type) {
  const def = DISPLACEMENTS[type];
  const params = {};
  for (const p of def.params) {
    params[p.key] = {
      value: p.default, default: p.default,
      anim: false, animMode: p.periodic ? "loop" : "sweep",
      ratio: 1,
      min: p.min, max: p.max, periodic: !!p.periodic,
    };
  }
  return {
    id: Math.random().toString(36).slice(2, 9),
    type, on: true, params,
  };
}

function applyPipeline(x0, y0, layers) {
  let x = x0, y = y0;
  for (const layer of layers) {
    if (!layer.on) continue;
    const def = TRANSFORMS[layer.type];
    if (!def) continue;
    const flat = {};
    for (const k in layer.params) flat[k] = layer.params[k].value;
    const out = def.needsOrigin ? def.fn(x, y, flat, x0, y0) : def.fn(x, y, flat);
    x = out[0]; y = out[1];
  }
  return [x, y];
}

// Apply 3D displacement pipeline sequentially.
function applyDispPipeline(x, y, x0, y0, dispLayers, jSignedFn) {
  let cx = x, cy = y, cz = 0;
  let jSigned = null;
  for (const layer of dispLayers) {
    if (!layer.on) continue;
    const def = DISPLACEMENTS[layer.type];
    if (!def) continue;
    if (def.needsJacobian && jSigned === null) jSigned = jSignedFn();
    const flat = {};
    for (const k in layer.params) flat[k] = layer.params[k].value;
    const out = def.fn(cx, cy, flat, x0, y0, jSigned ?? 0);
    cx += out[0];
    cy += out[1];
    cz += out[2];
  }
  return [cx - x, cy - y, cz];
}

function compositionString(layers, dispLayers) {
  const xy = layers.filter(l => l.on).map(l => TRANSFORMS[l.type]?.label || l.type);
  const dz = (dispLayers || []).filter(l => l.on).map(l => DISPLACEMENTS[l.type]?.label || l.type);
  let s = xy.length === 0 ? "M_0" : xy.reverse().join(" o ") + " (M_0)";
  if (dz.length > 0) s += "  +  Sum[" + dz.join(", ") + "]";
  return s;
}

function countDOFs(layers, dispLayers, view) {
  const active = [{ label: "base 2-manifold", struct: 2, temp: 0 }];
  let struct = 2, temp = 0;
  const tally = (layer, registry) => {
    if (!layer.on) return;
    const def = registry[layer.type];
    if (!def) return;
    const s = def.params.length;
    let t = 0, firstAnimRatio = null;
    for (const k in layer.params) {
      if (layer.params[k].anim) {
        t += 1;
        if (firstAnimRatio === null) firstAnimRatio = layer.params[k].ratio;
      }
    }
    active.push({ label: def.label, struct: s, temp: t, ratio: firstAnimRatio, animated: t > 0 });
    struct += s; temp += t;
  };
  for (const layer of layers) tally(layer, TRANSFORMS);
  for (const layer of (dispLayers || [])) tally(layer, DISPLACEMENTS);
  for (const v of view) {
    if (!v.on) continue;
    const t = v.anim ? 1 : 0;
    active.push({ label: v.label, struct: v.struct, temp: t, ratio: v.anim ? v.ratio : null, animated: t > 0 });
    struct += v.struct; temp += t;
  }
  return { active, struct, temp, total: struct + temp };
}

// ============================================================
// COMPONENT
// ============================================================

export default function PolydimensionalCoupling() {
  const mountRef = useRef(null);
  const sceneRef = useRef(null);
  const stateRef = useRef({});

  const [layers, setLayers] = useState([makeLayer("zPow")]);
  const [dispLayers, setDispLayers] = useState([makeDispLayer("zDisplacement")]);

  const [paletteKey, setPaletteKey] = useState("acid");
  const [tempo, setTempo] = useState(0.3);

  // ===== Camera system =====
  // Position
  const [orbitX, setOrbitX] = useState({ on: true, value: 0, default: 0, anim: false, animMode: "loop", ratio: 1, min: 0, max: Math.PI * 2, periodic: true });
  const [orbitY, setOrbitY] = useState({ on: true, value: 0.6, default: 0.6, anim: false, animMode: "sweep", ratio: 1/PHI, min: 0.1, max: Math.PI - 0.1, periodic: true });
  const [zoomP, setZoomP] = useState({ on: true, value: 3.5, default: 3.5, anim: false, animMode: "sweep", ratio: 1, min: 0.8, max: 12 });
  const [targetX, setTargetX] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -2, max: 2 });
  const [targetY, setTargetY] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -2, max: 2 });
  const [roll, setRoll] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "loop", ratio: 1, min: 0, max: Math.PI * 2, periodic: true });

  // Lens
  const [projection, setProjection] = useState("perspective");
  const [focalLength, setFocalLength] = useState({ on: true, value: 35, default: 35, anim: false, animMode: "sweep", ratio: 1, min: 8, max: 200 });
  const [sensorSize, setSensorSize] = useState({ on: true, value: 36, default: 36, anim: false, animMode: "sweep", ratio: 1, min: 4, max: 80 });
  const [orthoZoom, setOrthoZoom] = useState({ on: true, value: 2, default: 2, anim: false, animMode: "sweep", ratio: 1, min: 0.3, max: 6 });
  const [anamorphic, setAnamorphic] = useState({ on: false, value: 1, default: 1, anim: false, animMode: "sweep", ratio: 1, min: 0.3, max: 3 });
  const [shiftX, setShiftX] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });
  const [shiftY, setShiftY] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });

  // Distortion (radial Brown-Conrady k1, k2, applied post-projection in NDC)
  const [distortK1, setDistortK1] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });
  const [distortK2, setDistortK2] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });

  // Grid sampling (not strictly camera but lives in view layers)
  const [scaleP, setScaleP] = useState({ on: true, value: 0.6, default: 0.6, anim: false, animMode: "sweep", ratio: 1, min: 0.1, max: 1.5 });
  const [densityP, setDensityP] = useState({ on: true, value: 27, default: 27, anim: false, animMode: "sweep", ratio: 1, min: 7, max: 51 });

  const [quat, setQuat] = useState({
    on: false, mode: "fixed", anim: true,
    angle: 0, angleRatio: 1,
    ax: 0.5, ay: 1, az: 0.3,
    axRatio: PHI, ayRatio: 1, azRatio: 1/PHI,
    axAnim: false, ayAnim: false, azAnim: false,
    driftSpeed: 0.1,
  });

  const [showFormula, setShowFormula] = useState(true);
  const [showDimReadout, setShowDimReadout] = useState(true);
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [showDispAddMenu, setShowDispAddMenu] = useState(false);

  const P = PALETTES[paletteKey];

  const viewSummaries = [
    { on: orbitX.on, label: "camera azimuth", struct: 1, anim: orbitX.anim, ratio: orbitX.ratio },
    { on: orbitY.on, label: "camera elevation", struct: 1, anim: orbitY.anim, ratio: orbitY.ratio },
    { on: zoomP.on, label: "camera distance", struct: 1, anim: zoomP.anim, ratio: zoomP.ratio },
    { on: targetX.on, label: "target offset x", struct: 1, anim: targetX.anim, ratio: targetX.ratio },
    { on: targetY.on, label: "target offset y", struct: 1, anim: targetY.anim, ratio: targetY.ratio },
    { on: roll.on, label: "camera roll", struct: 1, anim: roll.anim, ratio: roll.ratio },
    { on: projection === "perspective" && focalLength.on, label: "focal length", struct: 1, anim: focalLength.anim, ratio: focalLength.ratio },
    { on: projection === "perspective" && sensorSize.on, label: "sensor size", struct: 1, anim: sensorSize.anim, ratio: sensorSize.ratio },
    { on: projection === "orthographic" && orthoZoom.on, label: "ortho zoom", struct: 1, anim: orthoZoom.anim, ratio: orthoZoom.ratio },
    { on: anamorphic.on, label: "anamorphic squeeze", struct: 1, anim: anamorphic.anim, ratio: anamorphic.ratio },
    { on: shiftX.on, label: "lens shift x", struct: 1, anim: shiftX.anim, ratio: shiftX.ratio },
    { on: shiftY.on, label: "lens shift y", struct: 1, anim: shiftY.anim, ratio: shiftY.ratio },
    { on: distortK1.on, label: "distortion k1", struct: 1, anim: distortK1.anim, ratio: distortK1.ratio },
    { on: distortK2.on, label: "distortion k2", struct: 1, anim: distortK2.anim, ratio: distortK2.ratio },
    { on: scaleP.on, label: "input scale", struct: 1, anim: scaleP.anim, ratio: scaleP.ratio },
    { on: densityP.on, label: "grid density", struct: 1, anim: densityP.anim, ratio: densityP.ratio },
  ];
  if (quat.on) {
    const phases = (quat.anim ? 1 : 0) + (quat.mode === "drift" ? 1 : 0)
      + (quat.axAnim ? 1 : 0) + (quat.ayAnim ? 1 : 0) + (quat.azAnim ? 1 : 0);
    viewSummaries.push({ on: true, label: "SO(3) tumble", struct: 3, anim: phases > 0, ratio: quat.angleRatio });
  }
  const dimAccount = countDOFs(layers, dispLayers, viewSummaries);
  const compString = compositionString(layers, dispLayers);

  stateRef.current = {
    layers, dispLayers, P,
    orbitX, orbitY, zoomP, targetX, targetY, roll,
    projection, focalLength, sensorSize, orthoZoom, anamorphic, shiftX, shiftY,
    distortK1, distortK2,
    scaleP, densityP, quat,
  };

  // Scene setup
  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;
    const w = mount.clientWidth, h = mount.clientHeight;
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(P.bg);
    const camera = new THREE.PerspectiveCamera(50, w / h, 0.1, 100);
    const orthoCam = new THREE.OrthographicCamera(-2, 2, 2, -2, 0.1, 100);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.setSize(w, h);
    mount.appendChild(renderer.domElement);

    const worldGroup = new THREE.Group();
    scene.add(worldGroup);
    const linesGroup = new THREE.Group();
    worldGroup.add(linesGroup);
    const glowGroup = new THREE.Group();
    worldGroup.add(glowGroup);

    sceneRef.current = { scene, camera, orthoCam, renderer, worldGroup, linesGroup, glowGroup, mount };

    let dragging = false, lastX = 0, lastY = 0;
    const onDown = (e) => {
      dragging = true; lastX = e.clientX; lastY = e.clientY;
      setOrbitX(p => ({ ...p, anim: false }));
      setOrbitY(p => ({ ...p, anim: false }));
    };
    const onUp = () => { dragging = false; };
    const onMove = (e) => {
      if (!dragging) return;
      const dx = e.clientX - lastX, dy = e.clientY - lastY;
      lastX = e.clientX; lastY = e.clientY;
      // Azimuth wraps; elevation clamps to its own range so we never cross the pole.
      setOrbitX(p => {
        const span = p.max - p.min || (Math.PI * 2);
        let v = p.value - dx * 0.01;
        v = ((v - p.min) % span + span) % span + p.min;
        return { ...p, value: v };
      });
      setOrbitY(p => {
        const v = Math.max(p.min, Math.min(p.max, p.value + dy * 0.01));
        return { ...p, value: v };
      });
    };
    renderer.domElement.addEventListener("mousedown", onDown);
    window.addEventListener("mouseup", onUp);
    window.addEventListener("mousemove", onMove);

    const onTouchStart = (e) => {
      if (e.touches.length === 1) {
        dragging = true; lastX = e.touches[0].clientX; lastY = e.touches[0].clientY;
        setOrbitX(p => ({ ...p, anim: false }));
        setOrbitY(p => ({ ...p, anim: false }));
      }
    };
    const onTouchMove = (e) => {
      if (!dragging || e.touches.length !== 1) return;
      e.preventDefault();
      onMove({ clientX: e.touches[0].clientX, clientY: e.touches[0].clientY });
    };
    const onTouchEnd = () => { dragging = false; };
    renderer.domElement.addEventListener("touchstart", onTouchStart);
    renderer.domElement.addEventListener("touchmove", onTouchMove, { passive: false });
    renderer.domElement.addEventListener("touchend", onTouchEnd);

    const onWheel = (e) => {
      e.preventDefault();
      const factor = e.deltaY > 0 ? 1.1 : 1 / 1.1;
      setZoomP(p => ({ ...p, value: Math.max(p.min, Math.min(p.max, p.value * factor)) }));
    };
    renderer.domElement.addEventListener("wheel", onWheel, { passive: false });

    const onResize = () => {
      if (!sceneRef.current) return;
      const w = mount.clientWidth, h = mount.clientHeight;
      sceneRef.current.camera.aspect = w / h;
      sceneRef.current.camera.updateProjectionMatrix();
      const aspect = w / h;
      const oc = sceneRef.current.orthoCam;
      const ozS = stateRef.current.orthoZoom;
      const z = (ozS && ozS.on ? ozS.value : 2) || 2;
      oc.left = -z * aspect; oc.right = z * aspect;
      oc.top = z; oc.bottom = -z;
      oc.updateProjectionMatrix();
      sceneRef.current.renderer.setSize(w, h);
    };
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("mouseup", onUp);
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("resize", onResize);
      renderer.domElement.removeEventListener("mousedown", onDown);
      renderer.domElement.removeEventListener("wheel", onWheel);
      renderer.domElement.removeEventListener("touchstart", onTouchStart);
      renderer.domElement.removeEventListener("touchmove", onTouchMove);
      renderer.domElement.removeEventListener("touchend", onTouchEnd);
      mount.removeChild(renderer.domElement);
      renderer.dispose();
    };
  }, []);

  useEffect(() => {
    if (sceneRef.current) sceneRef.current.scene.background = new THREE.Color(P.bg);
  }, [P.bg]);

  // Animation loop
  useEffect(() => {
    let raf;
    let last = performance.now();
    const driftStart = performance.now();

    const tick = (now) => {
      const dt = (now - last) / 1000;
      last = now;
      const baseRate = tempo;

      // Sweep a view param. Mode determines wrap behavior:
      //   "loop"  -> wraps min..max periodically (good for orbit angles, rotations)
      //   "sweep" -> ping-pongs smoothly between min and max (good for orbit Y, focal length, etc)
      const sweepView = (p) => {
        if (!p.anim) return p;
        const mode = p.animMode || (p.periodic ? "loop" : "sweep");
        const speed = baseRate * p.ratio;
        if (mode === "loop") {
          const span = p.max - p.min;
          if (span <= 0) return p;
          let v = p.value + dt * speed * span;
          v = ((v - p.min) % span + span) % span + p.min;
          return { ...p, value: v };
        }
        // sweep: smooth ping-pong using a phase locked to wall clock so it doesn't drift
        const phase = (now / 1000) * speed;
        const t = 0.5 - 0.5 * Math.cos(phase * Math.PI * 2);
        return { ...p, value: p.min + t * (p.max - p.min) };
      };

      setLayers(ls => ls.map(layer => {
        if (!layer.on) return layer;
        const newParams = {};
        let changed = false;
        for (const k in layer.params) {
          const param = layer.params[k];
          if (!param.anim) {
            newParams[k] = param;
          } else {
            const mode = param.animMode || (param.periodic ? "loop" : "sweep");
            const speed = baseRate * param.ratio;
            if (mode === "loop") {
              const span = param.max - param.min;
              if (span <= 0) {
                newParams[k] = param;
              } else {
                let v = param.value + dt * speed * span;
                v = ((v - param.min) % span + span) % span + param.min;
                newParams[k] = { ...param, value: v };
              }
            } else {
              const phase = (now / 1000) * speed;
              const t = 0.5 - 0.5 * Math.cos(phase * Math.PI * 2);
              newParams[k] = { ...param, value: param.min + t * (param.max - param.min) };
            }
            changed = true;
          }
        }
        return changed ? { ...layer, params: newParams } : layer;
      }));

      setDispLayers(ls => ls.map(layer => {
        if (!layer.on) return layer;
        const newParams = {};
        let changed = false;
        for (const k in layer.params) {
          const param = layer.params[k];
          if (!param.anim) {
            newParams[k] = param;
          } else {
            const mode = param.animMode || (param.periodic ? "loop" : "sweep");
            const speed = baseRate * param.ratio;
            if (mode === "loop") {
              const span = param.max - param.min;
              if (span <= 0) {
                newParams[k] = param;
              } else {
                let v = param.value + dt * speed * span;
                v = ((v - param.min) % span + span) % span + param.min;
                newParams[k] = { ...param, value: v };
              }
            } else {
              const phase = (now / 1000) * speed;
              const t = 0.5 - 0.5 * Math.cos(phase * Math.PI * 2);
              newParams[k] = { ...param, value: param.min + t * (param.max - param.min) };
            }
            changed = true;
          }
        }
        return changed ? { ...layer, params: newParams } : layer;
      }));

      setOrbitX(p => sweepView(p));
      setOrbitY(p => sweepView(p));
      setZoomP(p => sweepView(p));
      setTargetX(p => sweepView(p));
      setTargetY(p => sweepView(p));
      setRoll(p => sweepView(p));
      setFocalLength(p => sweepView(p));
      setSensorSize(p => sweepView(p));
      setOrthoZoom(p => sweepView(p));
      setAnamorphic(p => sweepView(p));
      setShiftX(p => sweepView(p));
      setShiftY(p => sweepView(p));
      setDistortK1(p => sweepView(p));
      setDistortK2(p => sweepView(p));
      setScaleP(p => sweepView(p));
      setDensityP(p => {
        const swept = sweepView(p);
        if (!p.anim) return swept;
        const r = Math.round(swept.value);
        return { ...swept, value: r % 2 === 0 ? r + 1 : r };
      });

      setQuat(q => {
        if (!q.on) return q;
        let next = { ...q };
        if (q.anim) {
          const speed = baseRate * q.angleRatio;
          next.angle = (q.angle + dt * speed * Math.PI * 2) % (Math.PI * 2);
        }
        if (q.mode === "drift") {
          const t = (now - driftStart) / 1000 * q.driftSpeed;
          next.ax = Math.sin(t * 0.7) + 0.3 * Math.cos(t * 1.3);
          next.ay = Math.cos(t * 0.5) + 0.3 * Math.sin(t * 1.7);
          next.az = Math.sin(t * 0.9 + 1.2);
        } else {
          if (q.axAnim) next.ax = Math.cos((now / 1000) * baseRate * q.axRatio * Math.PI * 2);
          if (q.ayAnim) next.ay = Math.cos((now / 1000) * baseRate * q.ayRatio * Math.PI * 2);
          if (q.azAnim) next.az = Math.cos((now / 1000) * baseRate * q.azRatio * Math.PI * 2);
        }
        return next;
      });

      if (sceneRef.current) {
        const s = stateRef.current;
        const cam = sceneRef.current.camera;
        const oc = sceneRef.current.orthoCam;
        const ax = s.orbitX.on ? s.orbitX.value : 0;
        const ay = s.orbitY.on ? s.orbitY.value : 0.6;
        const z = s.zoomP.on ? s.zoomP.value : 3.5;
        const tx = s.targetX.on ? s.targetX.value : 0;
        const ty = s.targetY.on ? s.targetY.value : 0;
        const rollA = s.roll.on ? s.roll.value : 0;

        // Position relative to target offset (offset is in world XZ ground plane)
        const px = tx + z * Math.sin(ay) * Math.sin(ax);
        const py = z * Math.cos(ay);
        const pz = ty + z * Math.sin(ay) * Math.cos(ax);

        // Build up vector with roll applied around the view direction
        const flipY = Math.sin(ay) >= 0 ? 1 : -1;
        // Forward direction (target - position), normalized
        const fwd = new THREE.Vector3(tx - px, 0 - py, ty - pz).normalize();
        // Pick a baseUp that isn't parallel to fwd. Default world Y, but at the
        // pole (looking straight up/down) fall back to world Z.
        let baseUp = new THREE.Vector3(0, flipY, 0);
        if (Math.abs(fwd.dot(baseUp)) > 0.9999) {
          baseUp = new THREE.Vector3(0, 0, 1);
        }
        // Project baseUp onto plane perpendicular to fwd to get a stable "world up after look-at"
        const upStable = baseUp.clone().sub(fwd.clone().multiplyScalar(baseUp.dot(fwd))).normalize();
        // Rotate that up around fwd by rollA
        const rolledUp = upStable.clone().applyAxisAngle(fwd, rollA);

        // Compute FOV from focal length + sensor (35mm-equivalent style)
        const f = s.focalLength.on ? s.focalLength.value : 35;
        const sensor = s.sensorSize.on ? s.sensorSize.value : 36;
        const fovDeg = (2 * Math.atan(sensor / (2 * f))) * (180 / Math.PI);

        cam.fov = fovDeg;
        cam.position.set(px, py, pz);
        cam.up.copy(rolledUp);
        cam.lookAt(tx, 0, ty);
        cam.updateProjectionMatrix();

        // Apply lens shift + anamorphic squeeze by post-multiplying the projection matrix.
        // Shift translates clip-space X/Y; anamorphic scales clip-space X.
        const sx = s.shiftX.on ? s.shiftX.value : 0;
        const sy = s.shiftY.on ? s.shiftY.value : 0;
        const ana = s.anamorphic.on ? s.anamorphic.value : 1;
        if (sx !== 0 || sy !== 0 || ana !== 1) {
          const M = new THREE.Matrix4().makeTranslation(sx, sy, 0);
          M.multiply(new THREE.Matrix4().makeScale(ana, 1, 1));
          cam.projectionMatrix.premultiply(M);
          cam.projectionMatrixInverse.copy(cam.projectionMatrix).invert();
        }

        const wPx = sceneRef.current.mount.clientWidth;
        const hPx = sceneRef.current.mount.clientHeight;
        const aspect = wPx / hPx;
        const oz = s.orthoZoom.on ? s.orthoZoom.value : 2;
        oc.left = -oz * aspect; oc.right = oz * aspect;
        oc.top = oz; oc.bottom = -oz;
        oc.position.set(px, py, pz);
        oc.up.copy(rolledUp);
        oc.lookAt(tx, 0, ty);
        oc.updateProjectionMatrix();
        if (sx !== 0 || sy !== 0 || ana !== 1) {
          const M = new THREE.Matrix4().makeTranslation(sx, sy, 0);
          M.multiply(new THREE.Matrix4().makeScale(ana, 1, 1));
          oc.projectionMatrix.premultiply(M);
          oc.projectionMatrixInverse.copy(oc.projectionMatrix).invert();
        }

        if (s.quat.on) {
          const a = new THREE.Vector3(s.quat.ax, s.quat.ay, s.quat.az);
          if (a.lengthSq() < 1e-6) a.set(0, 1, 0);
          a.normalize();
          const q = new THREE.Quaternion().setFromAxisAngle(a, s.quat.angle);
          sceneRef.current.worldGroup.quaternion.copy(q);
        } else {
          sceneRef.current.worldGroup.quaternion.identity();
        }
      }

      rebuildGeometry();
      if (sceneRef.current) {
        const activeCam = stateRef.current.projection === "orthographic"
          ? sceneRef.current.orthoCam
          : sceneRef.current.camera;
        sceneRef.current.renderer.render(sceneRef.current.scene, activeCam);
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [tempo]);

  const rebuildGeometry = () => {
    if (!sceneRef.current) return;
    const { linesGroup, glowGroup } = sceneRef.current;
    const s = stateRef.current;

    while (linesGroup.children.length) {
      const m = linesGroup.children.pop();
      m.geometry.dispose(); m.material.dispose();
    }
    while (glowGroup.children.length) {
      const m = glowGroup.children.pop();
      m.geometry.dispose(); m.material.dispose();
    }

    const N = Math.round(s.densityP.on ? s.densityP.value : 27);
    const SAMPLES = 80;
    const ext = s.scaleP.on ? s.scaleP.value : 0.6;

    const computeJSigned = (x0, y0) => {
      const eps = 0.01;
      const p0 = applyPipeline(x0, y0, s.layers);
      const px = applyPipeline(x0 + eps, y0, s.layers);
      const py = applyPipeline(x0, y0 + eps, s.layers);
      const dux = (px[0] - p0[0]) / eps;
      const duy = (py[0] - p0[0]) / eps;
      const dvx = (px[1] - p0[1]) / eps;
      const dvy = (py[1] - p0[1]) / eps;
      return dux * dvy - duy * dvx;
    };

    const buildLine = (paramFn) => {
      const points = [];
      for (let i = 0; i <= SAMPLES; i++) {
        const t = i / SAMPLES;
        const [x0, y0] = paramFn(t);
        const [u, v] = applyPipeline(x0, y0, s.layers);
        const [dx, dy, dz] = applyDispPipeline(u, v, x0, y0, s.dispLayers, () => computeJSigned(x0, y0));
        points.push([u + dx, dz, v + dy]);
      }
      return { points };
    };

    const lines = [];
    for (let i = 0; i <= N; i++) {
      const a = -ext + (2 * ext * i) / N;
      lines.push(buildLine((t) => [a, -ext + 2 * ext * t]));
      lines.push(buildLine((t) => [-ext + 2 * ext * t, a]));
    }

    const hasActiveLayers = s.layers.some(l => l.on) || s.dispLayers.some(l => l.on);
    const sharpMat = new THREE.LineBasicMaterial({
      color: new THREE.Color(hasActiveLayers ? s.P.warp : s.P.grid),
      transparent: true, opacity: 0.95,
    });
    const glowMat = new THREE.LineBasicMaterial({
      color: new THREE.Color(s.P.glow),
      transparent: true, opacity: 0.18,
    });

    // Lens distortion: applied per-vertex in NDC after camera projection.
    // We project worldGroup-local -> clip -> NDC, apply the distortion polynomial,
    // then invert back so the (still-active) camera projects each vertex to the
    // distorted NDC location during normal rendering.
    const k1 = s.distortK1.on ? s.distortK1.value : 0;
    const k2 = s.distortK2.on ? s.distortK2.value : 0;
    const distortActive = k1 !== 0 || k2 !== 0;

    let projView = null;
    let invProjView = null;
    if (distortActive) {
      const activeCam = s.projection === "orthographic"
        ? sceneRef.current.orthoCam
        : sceneRef.current.camera;
      // Ensure matrices are current before we read them.
      activeCam.updateMatrixWorld(true);
      sceneRef.current.worldGroup.updateMatrixWorld(true);
      // Bake worldGroup's model matrix into the projection so the round trip
      // returns a worldGroup-local position.
      projView = activeCam.projectionMatrix.clone()
        .multiply(activeCam.matrixWorldInverse)
        .multiply(sceneRef.current.worldGroup.matrixWorld);
      invProjView = projView.clone().invert();
    }

    const tmpV = new THREE.Vector3();
    const tmpV2 = new THREE.Vector3();

    const distortVertex = (x, y, z) => {
      tmpV.set(x, y, z).applyMatrix4(projView);
      const u = tmpV.x, v = tmpV.y, depth = tmpV.z;
      const r2 = u * u + v * v;
      const factor = 1 + k1 * r2 + k2 * r2 * r2;
      tmpV2.set(u * factor, v * factor, depth).applyMatrix4(invProjView);
      return [tmpV2.x, tmpV2.y, tmpV2.z];
    };

    for (const { points } of lines) {
      const verts = [];
      for (let i = 0; i < points.length; i++) {
        const [x, z, y] = points[i];
        if (distortActive) {
          const [dx, dy, dz] = distortVertex(x, z, y);
          verts.push(dx, dy, dz);
        } else {
          verts.push(x, z, y);
        }
      }
      const geom = new THREE.BufferGeometry();
      geom.setAttribute("position", new THREE.Float32BufferAttribute(verts, 3));
      linesGroup.add(new THREE.Line(geom, sharpMat));

      const glowGeom = new THREE.BufferGeometry();
      glowGeom.setAttribute("position", new THREE.Float32BufferAttribute(verts, 3));
      glowGroup.add(new THREE.Line(glowGeom, glowMat));
    }
  };

  // Layer manipulation
  const addLayer = (type) => {
    setLayers(ls => [...ls, makeLayer(type)]);
    setShowAddMenu(false);
  };
  const removeLayer = (id) => setLayers(ls => ls.filter(l => l.id !== id));
  const duplicateLayer = (id) => setLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const dup = { ...ls[idx], id: Math.random().toString(36).slice(2, 9),
      params: JSON.parse(JSON.stringify(ls[idx].params)) };
    const next = [...ls];
    next.splice(idx + 1, 0, dup);
    return next;
  });
  const moveLayer = (id, dir) => setLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const newIdx = idx + dir;
    if (newIdx < 0 || newIdx >= ls.length) return ls;
    const next = [...ls];
    [next[idx], next[newIdx]] = [next[newIdx], next[idx]];
    return next;
  });
  const updateLayer = (id, updates) => setLayers(ls => ls.map(l => l.id === id ? { ...l, ...updates } : l));
  const updateLayerParam = (id, key, updates) => setLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, params: { ...l.params, [key]: { ...l.params[key], ...updates } } };
  }));

  // Displacement layer manipulation
  const addDispLayer = (type) => {
    setDispLayers(ls => [...ls, makeDispLayer(type)]);
    setShowDispAddMenu(false);
  };
  const removeDispLayer = (id) => setDispLayers(ls => ls.filter(l => l.id !== id));
  const duplicateDispLayer = (id) => setDispLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const dup = { ...ls[idx], id: Math.random().toString(36).slice(2, 9),
      params: JSON.parse(JSON.stringify(ls[idx].params)) };
    const next = [...ls];
    next.splice(idx + 1, 0, dup);
    return next;
  });
  const moveDispLayer = (id, dir) => setDispLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const newIdx = idx + dir;
    if (newIdx < 0 || newIdx >= ls.length) return ls;
    const next = [...ls];
    [next[idx], next[newIdx]] = [next[newIdx], next[idx]];
    return next;
  });
  const updateDispLayer = (id, updates) => setDispLayers(ls => ls.map(l => l.id === id ? { ...l, ...updates } : l));
  const updateDispLayerParam = (id, key, updates) => setDispLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, params: { ...l.params, [key]: { ...l.params[key], ...updates } } };
  }));

  return (
    <div style={{
      minHeight: "100vh", background: "#050505", color: "#e8e8e8",
      fontFamily: "ui-monospace, 'SF Mono', Menlo, monospace",
      padding: 16, boxSizing: "border-box",
    }}>
      <div style={{ maxWidth: 1400, margin: "0 auto" }}>
        <header style={{ marginBottom: 16 }}>
          <div style={{
            fontFamily: "Georgia, 'Times New Roman', serif", fontStyle: "italic",
            fontSize: 28, letterSpacing: "0.02em", color: P.grid,
            textShadow: `0 0 20px ${P.glow}66`,
          }}>
            Polydimensional Coupling
          </div>
          <div style={{ fontSize: 11, opacity: 0.5, marginTop: 4, lineHeight: 1.5 }}>
            stackable transformation layers . ordered composition . n countable degrees of freedom
          </div>
        </header>

        <div style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr)",
          gap: 16,
        }}
        className="pdc-layout">
          <style>{`
            @media (min-width: 1024px) {
              .pdc-layout {
                grid-template-columns: minmax(0, 1fr) 460px !important;
                align-items: start !important;
              }
              .pdc-canvas-col {
                position: sticky !important;
                top: 16px !important;
                align-self: start !important;
              }
            }
          `}</style>

          <div className="pdc-canvas-col">
            <div style={{
              position: "relative", aspectRatio: "1 / 1",
              border: "1px solid #1a1a1a", borderRadius: 4,
              overflow: "hidden",
            }}>
              <div ref={mountRef} style={{ width: "100%", height: "100%", cursor: "grab" }} />
              {showFormula && (
                <div style={{
                  position: "absolute", bottom: 12, left: 12, right: 12,
                  fontFamily: "Georgia, serif", fontStyle: "italic",
                  fontSize: 14, color: P.warp,
                  textShadow: `0 0 8px ${P.glow}aa`, pointerEvents: "none",
                  maxWidth: "70%", lineHeight: 1.4,
                }}>
                  {compString}
                </div>
              )}
              {showDimReadout && (
                <div style={{
                  position: "absolute", top: 12, left: 12, right: 12,
                  fontFamily: "ui-monospace, monospace",
                  fontSize: 9, lineHeight: 1.6,
                  color: P.grid, opacity: 0.7,
                  pointerEvents: "none",
                  display: "flex", justifyContent: "space-between", gap: 12,
                }}>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ opacity: 0.5, marginBottom: 4 }}>active layers</div>
                    {dimAccount.active.map((a, i) => (
                      <div key={i} style={{
                        opacity: a.animated ? 1 : 0.65,
                        whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
                      }}>
                        {a.label} . {a.struct}{a.temp > 0 ? `+${a.temp}t` : ""}
                        {a.ratio !== null && a.animated && (
                          <span style={{ opacity: 0.5 }}> . x{a.ratio.toFixed(3)}</span>
                        )}
                      </div>
                    ))}
                  </div>
                  <div style={{ textAlign: "right" }}>
                    <div style={{ opacity: 0.5, marginBottom: 4 }}>dim count</div>
                    <div>structural . {dimAccount.struct}</div>
                    <div>temporal . {dimAccount.temp}</div>
                    <div style={{ marginTop: 4, color: P.warp }}>total . {dimAccount.total}D</div>
                    <div style={{ marginTop: 6, opacity: 0.6 }}>tempo . {tempo.toFixed(2)} c/s</div>
                  </div>
                </div>
              )}
            </div>
          </div>

          <div style={{
            display: "flex", flexDirection: "column", gap: 16,
            background: "#0c0c0c", border: "1px solid #1a1a1a",
            padding: 16, borderRadius: 4,
          }}>
            <Section label="tempo">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                base rate . animated parameters run at <span style={{ color: P.grid }}>tempo x ratio</span>
              </div>
              <RangeSlider label="tempo" value={tempo} onChange={setTempo}
                onReset={() => setTempo(0.3)}
                min={0} max={2} step={0.01} accent={P.grid} />
            </Section>

            <Section label="pipeline (top applied first . drag to reorder)">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                composition is non-commutative . ordering changes the result . stack as many transforms as you like
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {layers.map((layer, idx) => (
                  <PipelineCard
                    key={layer.id}
                    layer={layer}
                    index={idx}
                    total={layers.length}
                    onUpdate={(u) => updateLayer(layer.id, u)}
                    onUpdateParam={(k, u) => updateLayerParam(layer.id, k, u)}
                    onMoveUp={() => moveLayer(layer.id, -1)}
                    onMoveDown={() => moveLayer(layer.id, 1)}
                    onRemove={() => removeLayer(layer.id)}
                    onDuplicate={() => duplicateLayer(layer.id)}
                    accent={P.grid}
                  />
                ))}
                <button onClick={() => setShowAddMenu(true)} style={{
                  padding: "10px", border: `1px dashed ${P.grid}66`,
                  background: "transparent", color: P.grid,
                  borderRadius: 2, cursor: "pointer", fontSize: 11,
                  fontFamily: "ui-monospace, monospace",
                  letterSpacing: "0.1em", textTransform: "uppercase",
                }}>+ add transform</button>
              </div>
            </Section>

            {showAddMenu && <AddMenu onSelect={addLayer} onClose={() => setShowAddMenu(false)} accent={P.grid} />}

            <Section label="displacement pipeline (3D . sequential)">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                each layer reads the cumulative position from prior layers and adds its own (dx, dy, dz) . ordering matters . sign of amp inverts the displacement
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {dispLayers.map((layer, idx) => (
                  <DispPipelineCard
                    key={layer.id}
                    layer={layer}
                    index={idx}
                    total={dispLayers.length}
                    onUpdate={(u) => updateDispLayer(layer.id, u)}
                    onUpdateParam={(k, u) => updateDispLayerParam(layer.id, k, u)}
                    onMoveUp={() => moveDispLayer(layer.id, -1)}
                    onMoveDown={() => moveDispLayer(layer.id, 1)}
                    onRemove={() => removeDispLayer(layer.id)}
                    onDuplicate={() => duplicateDispLayer(layer.id)}
                    accent={P.grid}
                  />
                ))}
                <button onClick={() => setShowDispAddMenu(true)} style={{
                  padding: "10px", border: `1px dashed ${P.grid}66`,
                  background: "transparent", color: P.grid,
                  borderRadius: 2, cursor: "pointer", fontSize: 11,
                  fontFamily: "ui-monospace, monospace",
                  letterSpacing: "0.1em", textTransform: "uppercase",
                }}>+ add displacement</button>
              </div>
            </Section>

            {showDispAddMenu && <DispAddMenu onSelect={addDispLayer} onClose={() => setShowDispAddMenu(false)} accent={P.grid} />}

            <Section label="palette">
              <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                {Object.entries(PALETTES).map(([k, v]) => (
                  <button key={k} onClick={() => setPaletteKey(k)}
                    style={{
                      width: 28, height: 28, borderRadius: "50%",
                      background: v.grid,
                      border: paletteKey === k ? "2px solid #fff" : "2px solid transparent",
                      cursor: "pointer", boxShadow: `0 0 12px ${v.glow}88`,
                    }} title={k}/>
                ))}
              </div>
            </Section>

            <Section label="camera . position">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                where the eye is . target offset moves the look-at point . roll spins around the view axis
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="orbit x" sublabel="azimuth . 1 DOF"
                  param={orbitX} setParam={setOrbitX} accent={P.grid} step={0.001} periodic unit="rad" />
                <ViewLayerCard label="orbit y" sublabel="elevation . 1 DOF"
                  param={orbitY} setParam={setOrbitY} accent={P.grid} step={0.001} periodic unit="rad" />
                <ViewLayerCard label="distance" sublabel="camera distance to target . 1 DOF"
                  param={zoomP} setParam={setZoomP} accent={P.grid} step={0.05} />
                <ViewLayerCard label="target x" sublabel="look-at offset along world x . 1 DOF"
                  param={targetX} setParam={setTargetX} accent={P.grid} step={0.01} />
                <ViewLayerCard label="target y" sublabel="look-at offset along world z . 1 DOF"
                  param={targetY} setParam={setTargetY} accent={P.grid} step={0.01} />
                <ViewLayerCard label="roll" sublabel="rotation around view axis . 1 DOF"
                  param={roll} setParam={setRoll} accent={P.grid} step={0.001} periodic unit="rad" />
              </div>
            </Section>

            <Section label="camera . lens">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                how the eye sees . focal length + sensor size determine FOV . shift = tilt-shift . anamorphic = squeeze
              </div>
              <div style={{ display: "flex", gap: 6, marginBottom: 12 }}>
                {["perspective", "orthographic"].map(m => (
                  <Pill key={m} active={projection === m} onClick={() => setProjection(m)} accent={P.grid}>{m}</Pill>
                ))}
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                {projection === "perspective" && (
                  <>
                    <ViewLayerCard label="focal length" sublabel={`mm . FOV ${(2 * Math.atan((sensorSize.value || 36) / (2 * (focalLength.value || 35))) * 180 / Math.PI).toFixed(1)} deg`}
                      param={focalLength} setParam={setFocalLength} accent={P.grid} step={0.5} />
                    <ViewLayerCard label="sensor size" sublabel="mm . wider sensor = wider FOV at same focal length . 1 DOF"
                      param={sensorSize} setParam={setSensorSize} accent={P.grid} step={0.5} />
                  </>
                )}
                {projection === "orthographic" && (
                  <ViewLayerCard label="ortho zoom" sublabel="visible half-extent . 1 DOF"
                    param={orthoZoom} setParam={setOrthoZoom} accent={P.grid} step={0.05} />
                )}
                <ViewLayerCard label="anamorphic" sublabel="horizontal squeeze . <1 narrows, >1 stretches . 1 DOF"
                  param={anamorphic} setParam={setAnamorphic} accent={P.grid} step={0.01} />
                <ViewLayerCard label="lens shift x" sublabel="off-axis horizontal . tilt-shift . 1 DOF"
                  param={shiftX} setParam={setShiftX} accent={P.grid} step={0.01} />
                <ViewLayerCard label="lens shift y" sublabel="off-axis vertical . tilt-shift . 1 DOF"
                  param={shiftY} setParam={setShiftY} accent={P.grid} step={0.01} />
              </div>
            </Section>

            <Section label="camera . distortion">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                radial lens distortion . r' = r (1 + k1 r^2 + k2 r^4) . negative k = barrel (fisheye) . positive k = pincushion
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="k1" sublabel="quadratic distortion . dominant term . 1 DOF"
                  param={distortK1} setParam={setDistortK1} accent={P.grid} step={0.01} />
                <ViewLayerCard label="k2" sublabel="quartic distortion . higher-order shaping . 1 DOF"
                  param={distortK2} setParam={setDistortK2} accent={P.grid} step={0.01} />
              </div>
            </Section>

            <Section label="grid sampling">
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="input scale" sublabel="grid extent in domain . 1 DOF"
                  param={scaleP} setParam={setScaleP} accent={P.grid} step={0.01} />
                <ViewLayerCard label="grid density" sublabel="line count (odd) . 1 DOF"
                  param={densityP} setParam={setDensityP} accent={P.grid} step={2} oddOnly />

                <QuaternionCard quat={quat} setQuat={setQuat} accent={P.grid} />
              </div>
            </Section>

            <Section label="display">
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                <Toggle active={showFormula} onClick={() => setShowFormula(!showFormula)} accent={P.grid}>composition</Toggle>
                <Toggle active={showDimReadout} onClick={() => setShowDimReadout(!showDimReadout)} accent={P.grid}>dim readout</Toggle>
              </div>
            </Section>
          </div>
        </div>

        <div style={{ marginTop: 16, fontSize: 10, opacity: 0.45, lineHeight: 1.7 }}>
          <strong style={{ color: P.grid, fontWeight: "normal" }}>pipeline</strong> &mdash; each layer applies in order to the 2D grid. add as many as you want; reorder to change the resulting object since composition is non-commutative.
          <br/><br/>
          <strong style={{ color: P.grid, fontWeight: "normal" }}>ratios</strong> &mdash; animated params run at tempo x ratio. rational ratios &rarr; repeating polyrhythms . irrational/metallic &rarr; non-repeating coupling.
        </div>
      </div>
    </div>
  );
}

function DispPipelineCard({ layer, index, total, onUpdate, onUpdateParam,
                            onMoveUp, onMoveDown, onRemove, onDuplicate, accent }) {
  const def = DISPLACEMENTS[layer.type];
  const [expanded, setExpanded] = useState(true);
  if (!def) return null;
  const hasParams = def.params.length > 0;
  const anyAnim = Object.values(layer.params).some(p => p.anim);

  return (
    <div style={{
      border: `1px solid ${layer.on ? accent + "55" : "#222"}`,
      borderRadius: 4, padding: 10,
      background: layer.on ? accent + "08" : "transparent",
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          <button onClick={onMoveUp} disabled={index === 0} style={arrowBtn(index === 0)}>&#9650;</button>
          <button onClick={onMoveDown} disabled={index === total - 1} style={arrowBtn(index === total - 1)}>&#9660;</button>
        </div>
        <Switch on={layer.on} onClick={() => onUpdate({ on: !layer.on })} accent={accent} />
        <div style={{ flex: 1, minWidth: 0, cursor: "pointer" }} onClick={() => setExpanded(!expanded)}>
          <div style={{
            fontSize: 11, letterSpacing: "0.1em",
            color: layer.on ? accent : "#888", textTransform: "uppercase",
            display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap",
          }}>
            <span style={{ opacity: 0.5, fontSize: 9 }}>{index + 1}.</span>
            {def.label}
            <span style={{ fontSize: 8, opacity: 0.4 }}>. {def.category}</span>
            {anyAnim && <span style={{ fontSize: 8, opacity: 0.7, color: accent }}>. anim</span>}
          </div>
          <div style={{ fontSize: 9, opacity: 0.5, marginTop: 1 }}>{def.note}</div>
        </div>
        <button onClick={onDuplicate} style={iconBtn()} title="duplicate">+</button>
        <button onClick={onRemove} style={iconBtn()} title="remove">x</button>
        {hasParams && (
          <button onClick={() => setExpanded(!expanded)} style={iconBtn()}>{expanded ? "-" : "+"}</button>
        )}
      </div>
      {layer.on && expanded && hasParams && (
        <div style={{ marginTop: 10, display: "flex", flexDirection: "column", gap: 8 }}>
          {def.params.map(pdef => (
            <ParamControl
              key={pdef.key}
              pdef={pdef}
              param={layer.params[pdef.key]}
              onUpdate={(u) => onUpdateParam(pdef.key, u)}
              accent={accent}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function DispAddMenu({ onSelect, onClose, accent }) {
  return (
    <div style={{
      border: `1px solid ${accent}55`, borderRadius: 4, padding: 12,
      background: accent + "08",
    }}>
      <div style={{
        display: "flex", justifyContent: "space-between", alignItems: "center",
        marginBottom: 10,
      }}>
        <div style={{ fontSize: 10, letterSpacing: "0.15em", color: accent, textTransform: "uppercase" }}>
          add displacement
        </div>
        <button onClick={onClose} style={iconBtn()}>x</button>
      </div>
      {DISP_CATEGORIES.map(cat => {
        const items = Object.entries(DISPLACEMENTS).filter(([_, t]) => t.category === cat);
        if (items.length === 0) return null;
        return (
          <div key={cat} style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 9, opacity: 0.5, marginBottom: 4, letterSpacing: "0.1em", textTransform: "uppercase" }}>
              {cat}
            </div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
              {items.map(([key, t]) => (
                <button key={key} onClick={() => onSelect(key)} style={{
                  padding: "5px 9px",
                  background: "transparent", color: "#ccc",
                  border: `1px solid #333`, borderRadius: 2,
                  cursor: "pointer", fontSize: 11,
                  fontFamily: "ui-monospace, monospace",
                }} title={t.note}>{t.label}</button>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function PipelineCard({ layer, index, total, onUpdate, onUpdateParam,
                        onMoveUp, onMoveDown, onRemove, onDuplicate, accent }) {
  const def = TRANSFORMS[layer.type];
  const [expanded, setExpanded] = useState(true);
  if (!def) return null;
  const hasParams = def.params.length > 0;
  const anyAnim = Object.values(layer.params).some(p => p.anim);

  return (
    <div style={{
      border: `1px solid ${layer.on ? accent + "55" : "#222"}`,
      borderRadius: 4, padding: 10,
      background: layer.on ? accent + "08" : "transparent",
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          <button onClick={onMoveUp} disabled={index === 0} style={arrowBtn(index === 0)}>&#9650;</button>
          <button onClick={onMoveDown} disabled={index === total - 1} style={arrowBtn(index === total - 1)}>&#9660;</button>
        </div>
        <Switch on={layer.on} onClick={() => onUpdate({ on: !layer.on })} accent={accent} />
        <div style={{ flex: 1, minWidth: 0, cursor: "pointer" }} onClick={() => setExpanded(!expanded)}>
          <div style={{
            fontSize: 11, letterSpacing: "0.1em",
            color: layer.on ? accent : "#888", textTransform: "uppercase",
            display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap",
          }}>
            <span style={{ opacity: 0.5, fontSize: 9 }}>{index + 1}.</span>
            {def.label}
            <span style={{ fontSize: 8, opacity: 0.4 }}>. {def.category}</span>
            {anyAnim && <span style={{ fontSize: 8, opacity: 0.7, color: accent }}>. anim</span>}
          </div>
          <div style={{ fontSize: 9, opacity: 0.5, marginTop: 1 }}>{def.note}</div>
        </div>
        <button onClick={onDuplicate} style={iconBtn()} title="duplicate">+</button>
        <button onClick={onRemove} style={iconBtn()} title="remove">x</button>
        {hasParams && (
          <button onClick={() => setExpanded(!expanded)} style={iconBtn()}>{expanded ? "-" : "+"}</button>
        )}
      </div>
      {layer.on && expanded && hasParams && (
        <div style={{ marginTop: 10, display: "flex", flexDirection: "column", gap: 8 }}>
          {def.params.map(pdef => (
            <ParamControl
              key={pdef.key}
              pdef={pdef}
              param={layer.params[pdef.key]}
              onUpdate={(u) => onUpdateParam(pdef.key, u)}
              accent={accent}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function ParamControl({ pdef, param, onUpdate, accent }) {
  const display = pdef.periodic
    ? (param.value / Math.PI).toFixed(3) + "pi"
    : param.value.toFixed(3);
  const animMode = param.animMode || (param.periodic ? "loop" : "sweep");
  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <AnimToggle anim={param.anim} onClick={() => onUpdate({ anim: !param.anim })} accent={accent} small />
        <div style={{ flex: 1 }}>
          <RangeSlider
            label={pdef.key}
            value={param.value}
            min={param.min} max={param.max}
            step={pdef.periodic ? 0.001 : 0.01}
            onChange={(v) => onUpdate({ value: v, anim: false })}
            onRangeChange={(min, max) => onUpdate({ min, max })}
            onReset={() => onUpdate({ value: pdef.default, anim: false })}
            accent={accent} editableRange
            display={display}
          />
        </div>
      </div>
      {param.anim && (
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
          <AnimModeToggle mode={animMode}
            onClick={() => onUpdate({ animMode: animMode === "loop" ? "sweep" : "loop" })}
            accent={accent} />
          <div style={{ flex: 1 }}>
            <RatioRow ratio={param.ratio} onChange={(r) => onUpdate({ ratio: r })} accent={accent} compact />
          </div>
        </div>
      )}
    </div>
  );
}

function AddMenu({ onSelect, onClose, accent }) {
  return (
    <div style={{
      border: `1px solid ${accent}55`, borderRadius: 4, padding: 12,
      background: accent + "08",
    }}>
      <div style={{
        display: "flex", justifyContent: "space-between", alignItems: "center",
        marginBottom: 10,
      }}>
        <div style={{ fontSize: 10, letterSpacing: "0.15em", color: accent, textTransform: "uppercase" }}>
          add transform
        </div>
        <button onClick={onClose} style={iconBtn()}>x</button>
      </div>
      {CATEGORIES.map(cat => {
        const transforms = Object.entries(TRANSFORMS).filter(([_, t]) => t.category === cat);
        if (transforms.length === 0) return null;
        return (
          <div key={cat} style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 9, opacity: 0.5, marginBottom: 4, letterSpacing: "0.1em", textTransform: "uppercase" }}>
              {cat}
            </div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
              {transforms.map(([key, t]) => (
                <button key={key} onClick={() => onSelect(key)} style={{
                  padding: "5px 9px",
                  background: "transparent", color: "#ccc",
                  border: `1px solid #333`, borderRadius: 2,
                  cursor: "pointer", fontSize: 11,
                  fontFamily: "ui-monospace, monospace",
                }} title={t.note}>{t.label}</button>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function ViewLayerCard({ label, sublabel, param, setParam, accent, step, periodic, unit, integer, oddOnly, extra }) {
  const quantize = (v) => {
    if (oddOnly) {
      const r = Math.round(v);
      return r % 2 === 0 ? r + 1 : r;
    }
    if (integer) return Math.round(v);
    return v;
  };
  const animMode = param.animMode || (param.periodic ? "loop" : "sweep");
  return (
    <div style={{
      border: `1px solid ${param.on ? accent + "55" : "#222"}`,
      borderRadius: 4, padding: 12,
      background: param.on ? accent + "08" : "transparent",
    }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: param.on ? 10 : 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <Switch on={param.on} onClick={() => setParam(p => ({ ...p, on: !p.on }))} accent={accent} />
          <div>
            <div style={{
              fontSize: 11, letterSpacing: "0.15em",
              color: param.on ? accent : "#888", textTransform: "uppercase",
            }}>{label}</div>
            <div style={{ fontSize: 9, opacity: 0.5, marginTop: 2 }}>{sublabel}</div>
          </div>
        </div>
        {param.on && (
          <AnimToggle anim={param.anim} onClick={() => setParam(p => ({ ...p, anim: !p.anim }))} accent={accent} />
        )}
      </div>
      {param.on && (
        <>
          {extra}
          <RangeSlider
            label="value"
            value={param.value}
            min={param.min} max={param.max} step={step}
            onChange={(v) => setParam(p => ({ ...p, value: quantize(v), anim: false }))}
            onRangeChange={(min, max) => setParam(p => ({ ...p, min: oddOnly || integer ? Math.round(min) : min, max: oddOnly || integer ? Math.round(max) : max }))}
            onReset={() => setParam(p => ({ ...p, value: quantize(p.default !== undefined ? p.default : p.value), anim: false }))}
            accent={accent} editableRange
            display={oddOnly ? quantize(param.value).toString()
              : integer ? Math.round(param.value).toString()
              : unit === "rad" ? (param.value / Math.PI).toFixed(3) + "pi"
              : param.value.toFixed(2)}
          />
          {param.anim && (
            <>
              <div style={{ height: 8 }} />
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <AnimModeToggle mode={animMode}
                  onClick={() => setParam(p => ({ ...p, animMode: animMode === "loop" ? "sweep" : "loop" }))}
                  accent={accent} />
                <div style={{ flex: 1 }}>
                  <RatioRow ratio={param.ratio} onChange={(r) => setParam(p => ({ ...p, ratio: r }))} accent={accent} />
                </div>
              </div>
            </>
          )}
        </>
      )}
    </div>
  );
}

function QuaternionCard({ quat, setQuat, accent }) {
  return (
    <div style={{
      border: `1px solid ${quat.on ? accent + "55" : "#222"}`,
      borderRadius: 4, padding: 12,
      background: quat.on ? accent + "08" : "transparent",
    }}>
      <div style={{
        display: "flex", alignItems: "center",
        justifyContent: "space-between", marginBottom: quat.on ? 12 : 0,
      }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <Switch on={quat.on} onClick={() => setQuat(q => ({ ...q, on: !q.on }))} accent={accent} />
          <div>
            <div style={{
              fontSize: 11, letterSpacing: "0.15em",
              color: quat.on ? accent : "#888", textTransform: "uppercase",
            }}>quaternion tumble</div>
            <div style={{ fontSize: 9, opacity: 0.5, marginTop: 2 }}>SO(3) action on ambient R^3 . 3 DOF</div>
          </div>
        </div>
      </div>
      {quat.on && (
        <>
          <div style={{ marginBottom: 12 }}>
            <div style={{ fontSize: 9, letterSpacing: "0.2em", opacity: 0.5, marginBottom: 6, textTransform: "uppercase" }}>axis mode</div>
            <div style={{ display: "flex", gap: 6 }}>
              {["fixed", "drift"].map(m => (
                <Pill key={m} active={quat.mode === m} onClick={() => setQuat(q => ({ ...q, mode: m }))} accent={accent}>{m}</Pill>
              ))}
            </div>
          </div>
          {quat.mode === "fixed" && (
            <>
              <AxisRow label="ax" value={quat.ax} defaultValue={0.5}
                onChange={(v) => setQuat(q => ({ ...q, ax: v }))}
                anim={quat.axAnim} onAnim={(b) => setQuat(q => ({ ...q, axAnim: b }))}
                ratio={quat.axRatio} onRatio={(r) => setQuat(q => ({ ...q, axRatio: r }))} accent={accent} />
              <AxisRow label="ay" value={quat.ay} defaultValue={1}
                onChange={(v) => setQuat(q => ({ ...q, ay: v }))}
                anim={quat.ayAnim} onAnim={(b) => setQuat(q => ({ ...q, ayAnim: b }))}
                ratio={quat.ayRatio} onRatio={(r) => setQuat(q => ({ ...q, ayRatio: r }))} accent={accent} />
              <AxisRow label="az" value={quat.az} defaultValue={0.3}
                onChange={(v) => setQuat(q => ({ ...q, az: v }))}
                anim={quat.azAnim} onAnim={(b) => setQuat(q => ({ ...q, azAnim: b }))}
                ratio={quat.azRatio} onRatio={(r) => setQuat(q => ({ ...q, azRatio: r }))} accent={accent} />
            </>
          )}
          {quat.mode === "drift" && (
            <RangeSlider label="drift speed" value={quat.driftSpeed}
              min={0.01} max={1} step={0.01}
              onReset={() => setQuat(q => ({ ...q, driftSpeed: 0.1 }))}
              onChange={(v) => setQuat(q => ({ ...q, driftSpeed: v }))} accent={accent} />
          )}
          <div style={{ height: 10 }} />
          <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 6 }}>
            <AnimToggle anim={quat.anim} onClick={() => setQuat(q => ({ ...q, anim: !q.anim }))} accent={accent} />
            <div style={{ flex: 1 }}>
              <RangeSlider label="angle theta" value={quat.angle / Math.PI}
                min={0} max={2} step={0.001}
                onReset={() => setQuat(q => ({ ...q, angle: 0 }))}
                onChange={(v) => setQuat(q => ({ ...q, angle: v * Math.PI, anim: false }))} accent={accent}
                display={(quat.angle / Math.PI).toFixed(3) + "pi"} />
            </div>
          </div>
          <RatioRow ratio={quat.angleRatio} onChange={(r) => setQuat(q => ({ ...q, angleRatio: r }))} accent={accent} />
        </>
      )}
    </div>
  );
}

function AxisRow({ label, value, defaultValue, onChange, anim, onAnim, ratio, onRatio, accent }) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <AnimToggle anim={anim} onClick={() => onAnim(!anim)} accent={accent} small />
        <div style={{ flex: 1 }}>
          <RangeSlider label={label} value={value} min={-1} max={1} step={0.01}
            onChange={onChange}
            onReset={defaultValue !== undefined ? () => onChange(defaultValue) : undefined}
            accent={accent} />
        </div>
      </div>
      {anim && <RatioRow ratio={ratio} onChange={onRatio} accent={accent} compact />}
    </div>
  );
}

function RatioRow({ ratio, onChange, accent, compact }) {
  return (
    <div style={{ marginTop: compact ? 4 : 0 }}>
      <div style={{
        display: "flex", justifyContent: "space-between", alignItems: "center",
        fontSize: 9, letterSpacing: "0.15em", opacity: 0.6,
        marginBottom: 6, textTransform: "uppercase",
      }}>
        <span>ratio to tempo</span>
        <span style={{ color: accent, opacity: 0.9 }}>x{ratio.toFixed(4)}</span>
      </div>
      {RATIO_GROUPS.map((group) => (
        <div key={group.label} style={{ marginBottom: 6 }}>
          <div style={{ fontSize: 8, opacity: 0.4, marginBottom: 3, letterSpacing: "0.1em" }}>{group.label}</div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 3 }}>
            {group.presets.map(({ label, value }) => (
              <button key={label} onClick={() => onChange(value)} style={{
                padding: "3px 6px",
                background: Math.abs(ratio - value) < 1e-4 ? accent : "transparent",
                color: Math.abs(ratio - value) < 1e-4 ? "#000" : "#999",
                border: `1px solid ${Math.abs(ratio - value) < 1e-4 ? accent : "#333"}`,
                borderRadius: 2, cursor: "pointer", fontSize: 9,
                fontFamily: "ui-monospace, monospace",
              }}>{label}</button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function RangeSlider({ label, value, min, max, step, onChange, onRangeChange, accent, editableRange, display, onReset }) {
  const onContextMenu = (e) => {
    if (onReset) {
      e.preventDefault();
      onReset();
    }
  };
  return (
    <div>
      <div style={{
        display: "flex", justifyContent: "space-between",
        fontSize: 10, letterSpacing: "0.15em", opacity: 0.6,
        marginBottom: 6, textTransform: "uppercase",
      }}>
        <span title={onReset ? "right-click to reset to default" : undefined}>{label}</span>
        <span style={{ color: accent, opacity: 0.9 }}>
          {display !== undefined ? display : value.toFixed(2)}
        </span>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        {editableRange && (
          <EditableNumber value={min} accent={accent}
            onCommit={(v) => {
              if (v < max - 0.001 && onRangeChange) onRangeChange(v, max);
            }} />
        )}
        <input type="range" min={min} max={max} step={step}
          value={Math.max(min, Math.min(max, value))}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          onContextMenu={onContextMenu}
          style={{ flex: 1, accentColor: accent }} />
        {editableRange && (
          <EditableNumber value={max} accent={accent}
            onCommit={(v) => {
              if (v > min + 0.001 && onRangeChange) onRangeChange(min, v);
            }} />
        )}
      </div>
    </div>
  );
}

// Click to edit, type a value, Enter or blur to commit. Escape to cancel.
function EditableNumber({ value, onCommit, accent }) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const inputRef = useRef(null);

  useEffect(() => {
    if (editing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editing]);

  const startEditing = () => {
    setDraft(formatNumber(value));
    setEditing(true);
  };

  const commit = () => {
    const parsed = parseFloat(draft);
    if (!Number.isNaN(parsed)) onCommit(parsed);
    setEditing(false);
  };

  if (editing) {
    return (
      <input
        ref={inputRef}
        type="number"
        value={draft}
        step="any"
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          else if (e.key === "Escape") setEditing(false);
        }}
        style={{
          fontSize: 10, color: accent,
          background: "#000",
          border: `1px solid ${accent}`,
          borderRadius: 2, padding: "2px 4px",
          width: 56, textAlign: "center",
          fontFamily: "ui-monospace, monospace",
          outline: "none",
          MozAppearance: "textfield",
        }}
      />
    );
  }
  return (
    <button
      onClick={startEditing}
      style={{
        fontSize: 9, opacity: 0.6,
        color: "#aaa",
        cursor: "text", userSelect: "none",
        padding: "2px 4px",
        border: "1px dashed #333",
        background: "transparent",
        borderRadius: 2, minWidth: 36, textAlign: "center",
        fontFamily: "ui-monospace, monospace",
      }}
      title="click to type a value"
    >
      {formatNumber(value)}
    </button>
  );
}

function formatNumber(v) {
  if (v === Math.round(v)) return v.toString();
  // Trim trailing zeros to stay compact
  return parseFloat(v.toFixed(4)).toString();
}

function Section({ label, children }) {
  return (
    <div>
      <div style={{
        fontSize: 9, letterSpacing: "0.2em", opacity: 0.5,
        marginBottom: 8, textTransform: "uppercase",
      }}>{label}</div>
      {children}
    </div>
  );
}

function Pill({ children, active, onClick, accent }) {
  return (
    <button onClick={onClick} style={{
      padding: "5px 10px",
      background: active ? accent : "transparent",
      color: active ? "#000" : "#ccc",
      border: `1px solid ${active ? accent : "#333"}`,
      borderRadius: 2, cursor: "pointer", fontSize: 11,
      fontFamily: "ui-monospace, monospace",
    }}>{children}</button>
  );
}

function Switch({ on, onClick, accent }) {
  return (
    <button onClick={onClick} style={{
      width: 32, height: 18, borderRadius: 9,
      background: on ? accent : "#222", border: "none",
      position: "relative", cursor: "pointer",
      padding: 0, transition: "background 150ms", flexShrink: 0,
    }}>
      <div style={{
        position: "absolute", top: 2, left: on ? 16 : 2,
        width: 14, height: 14, borderRadius: "50%",
        background: on ? "#000" : "#666",
        transition: "left 150ms",
      }}/>
    </button>
  );
}

function AnimToggle({ anim, onClick, accent, small }) {
  return (
    <button onClick={onClick} style={{
      padding: small ? "2px 6px" : "3px 8px",
      background: anim ? accent : "transparent",
      color: anim ? "#000" : "#888",
      border: `1px solid ${anim ? accent : "#333"}`,
      borderRadius: 2, cursor: "pointer",
      fontSize: small ? 9 : 10,
      fontFamily: "ui-monospace, monospace",
      flexShrink: 0,
    }}>{anim ? "[]" : ">"}</button>
  );
}

// Toggles the wrap behavior of an animated parameter:
//   loop   -> wraps min..max periodically (good for angular DOFs that should rotate continuously)
//   sweep  -> ping-pongs smoothly between min and max (good for non-angular DOFs)
function AnimModeToggle({ mode, onClick, accent }) {
  const isLoop = mode === "loop";
  return (
    <button onClick={onClick} title={isLoop ? "loop -> sweep" : "sweep -> loop"} style={{
      padding: "2px 8px",
      background: "transparent",
      color: accent, opacity: 0.85,
      border: `1px solid ${accent}55`,
      borderRadius: 2, cursor: "pointer",
      fontSize: 9, letterSpacing: "0.1em",
      fontFamily: "ui-monospace, monospace",
      flexShrink: 0, textTransform: "uppercase",
    }}>{isLoop ? "loop" : "sweep"}</button>
  );
}

function Toggle({ children, active, onClick, accent }) {
  return (
    <button onClick={onClick} style={{
      padding: "6px 12px",
      background: active ? `${accent}22` : "transparent",
      color: active ? accent : "#888",
      border: `1px solid ${active ? accent : "#333"}`,
      borderRadius: 2, cursor: "pointer", fontSize: 11,
      fontFamily: "ui-monospace, monospace",
    }}>{children}</button>
  );
}

function arrowBtn(disabled) {
  return {
    padding: "2px 6px", fontSize: 10, lineHeight: 1,
    background: "transparent",
    color: disabled ? "#444" : "#888",
    border: `1px solid ${disabled ? "#222" : "#333"}`,
    borderRadius: 2, cursor: disabled ? "default" : "pointer",
    fontFamily: "ui-monospace, monospace",
  };
}

function iconBtn() {
  return {
    padding: "4px 8px", fontSize: 11, lineHeight: 1,
    background: "transparent", color: "#888",
    border: "1px solid #333", borderRadius: 2, cursor: "pointer",
    fontFamily: "ui-monospace, monospace",
  };
}
