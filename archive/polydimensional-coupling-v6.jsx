import React, { useState, useRef, useEffect } from "react";
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
    label: "blend (utility)",
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
    label: "integer",
    presets: [
      { label: "1", value: 1 }, { label: "2", value: 2 }, { label: "3", value: 3 },
      { label: "4", value: 4 }, { label: "5", value: 5 }, { label: "6", value: 6 },
      { label: "7", value: 7 }, { label: "8", value: 8 },
      { label: "12", value: 12 }, { label: "16", value: 16 },
      { label: "24", value: 24 }, { label: "32", value: 32 },
      { label: "48", value: 48 }, { label: "64", value: 64 },
    ],
  },
  {
    label: "1/n",
    presets: [
      { label: "1/2", value: 1/2 }, { label: "1/3", value: 1/3 },
      { label: "1/4", value: 1/4 }, { label: "1/5", value: 1/5 },
      { label: "1/6", value: 1/6 }, { label: "1/7", value: 1/7 },
      { label: "1/8", value: 1/8 }, { label: "1/10", value: 1/10 },
      { label: "1/12", value: 1/12 }, { label: "1/16", value: 1/16 },
      { label: "1/24", value: 1/24 }, { label: "1/32", value: 1/32 },
      { label: "1/48", value: 1/48 }, { label: "1/64", value: 1/64 },
    ],
  },
  {
    label: "harmonic",
    presets: [
      { label: "2/3", value: 2/3 }, { label: "3/4", value: 3/4 },
      { label: "4/5", value: 4/5 }, { label: "5/6", value: 5/6 },
      { label: "3/2", value: 3/2 }, { label: "4/3", value: 4/3 },
      { label: "5/4", value: 5/4 }, { label: "6/5", value: 6/5 },
      { label: "5/3", value: 5/3 }, { label: "7/4", value: 7/4 },
      { label: "7/5", value: 7/5 }, { label: "9/8", value: 9/8 },
      { label: "8/7", value: 8/7 }, { label: "7/6", value: 7/6 },
    ],
  },
  {
    label: "just intonation",
    presets: [
      { label: "16/15", value: 16/15 }, { label: "9/8", value: 9/8 },
      { label: "6/5", value: 6/5 }, { label: "5/4", value: 5/4 },
      { label: "4/3", value: 4/3 }, { label: "3/2", value: 3/2 },
      { label: "8/5", value: 8/5 }, { label: "5/3", value: 5/3 },
      { label: "15/8", value: 15/8 }, { label: "7/4", value: 7/4 },
      { label: "2", value: 2 },
    ],
  },
  {
    label: "fibonacci",
    presets: [
      { label: "1/8", value: 1/8 }, { label: "1/5", value: 1/5 },
      { label: "2/5", value: 2/5 }, { label: "3/5", value: 3/5 },
      { label: "5/8", value: 5/8 }, { label: "8/13", value: 8/13 },
      { label: "13/21", value: 13/21 }, { label: "21/34", value: 21/34 },
      { label: "5/3", value: 5/3 }, { label: "8/5", value: 8/5 },
      { label: "13/8", value: 13/8 }, { label: "21/13", value: 21/13 },
      { label: "34/21", value: 34/21 },
    ],
  },
  {
    label: "metallic",
    presets: [
      { label: "1/phi^2", value: 1/(PHI*PHI) }, { label: "1/phi", value: 1/PHI },
      { label: "phi", value: PHI }, { label: "phi^2", value: PHI*PHI },
      { label: "delta_s", value: SILVER }, { label: "1/delta_s", value: 1/SILVER },
      { label: "delta_b", value: BRONZE }, { label: "1/delta_b", value: 1/BRONZE },
      { label: "rho", value: PLASTIC }, { label: "1/rho", value: 1/PLASTIC },
    ],
  },
  {
    label: "irrational",
    presets: [
      { label: "1/pi", value: 1/Math.PI }, { label: "1/e", value: 1/Math.E },
      { label: "sqrt2/2", value: Math.SQRT2/2 }, { label: "sqrt3/2", value: Math.sqrt(3)/2 },
      { label: "sqrt2", value: Math.SQRT2 }, { label: "sqrt3", value: Math.sqrt(3) },
      { label: "sqrt5", value: Math.sqrt(5) }, { label: "e", value: Math.E },
      { label: "pi", value: Math.PI }, { label: "pi/2", value: Math.PI/2 },
      { label: "2pi", value: Math.PI*2 }, { label: "e/pi", value: Math.E/Math.PI },
      { label: "pi/e", value: Math.PI/Math.E },
    ],
  },
  {
    label: "powers of 2",
    presets: [
      { label: "1/128", value: 1/128 }, { label: "1/64", value: 1/64 },
      { label: "1/32", value: 1/32 }, { label: "1/16", value: 1/16 },
      { label: "1/8", value: 1/8 }, { label: "1/4", value: 1/4 },
      { label: "1/2", value: 1/2 }, { label: "2", value: 2 },
      { label: "4", value: 4 }, { label: "8", value: 8 },
      { label: "16", value: 16 }, { label: "32", value: 32 },
      { label: "64", value: 64 }, { label: "128", value: 128 },
    ],
  },
];

const BPM_PRESETS = [1, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200];

// ============================================================
// STABLE LAYER IDS
// ============================================================

function generateLayerId() {
  return Math.random().toString(36).slice(2, 10).padEnd(8, "0");
}

// ============================================================
// PRESET SYSTEM (inlined from preset-schema.ts + preset-io.ts)
// ============================================================

const SCHEMA_NAME = "polyform.preset";
const SCHEMA_VERSION = 1;

// --- defaults ---

function defaultBlend() {
  return { value: 1.0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
}

function defaultParam(value, min, max, periodic) {
  return { value, min, max, anim: false, animMode: periodic ? "loop" : "sweep", ratio: 1 };
}

function defaultToggleParam(on, value, min, max, periodic) {
  return { ...defaultParam(value, min, max, periodic), on };
}

function defaultMetadata() {
  const now = new Date().toISOString();
  return { name: "Untitled", description: "", author: "", created: now, modified: now, tags: [] };
}

function defaultCamera() {
  return {
    position: {
      orbitX: defaultToggleParam(true, 0, 0, Math.PI * 2, true),
      orbitY: { ...defaultToggleParam(true, 0.1, 0.1, Math.PI - 0.1, true), ratio: 1 / PHI },
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

function defaultQuaternion() {
  return {
    on: false, mode: "fixed", anim: true,
    angle: 0, angleRatio: 1,
    axes: {
      ax: { value: 0.5, anim: false, ratio: PHI },
      ay: { value: 1.0, anim: false, ratio: 1 },
      az: { value: 0.3, anim: false, ratio: 1 / PHI },
    },
    driftSpeed: 0.1,
  };
}

function defaultSampling() {
  return {
    scale: defaultToggleParam(true, 0.6, 0.1, 1.5),
    density: defaultToggleParam(true, 27, 7, 51),
  };
}

function defaultDisplay() {
  return { showFormula: true, showDimCount: true, showActiveLayers: true, showParamValues: true };
}

function defaultDefaults() {
  return { blendTime: 1.0 };
}

function defaultTempo() {
  return defaultParam(0.3, 0, 2);
}

function emptyPreset() {
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

// --- validation ---

function isObject(x) {
  return typeof x === "object" && x !== null && !Array.isArray(x);
}

function asNumber(x, fallback, ctx, path) {
  if (typeof x === "number" && Number.isFinite(x)) return x;
  if (x !== undefined) ctx.warnings.push(path + ": expected number, using default " + fallback);
  return fallback;
}

function asString(x, fallback) {
  return typeof x === "string" ? x : fallback;
}

function asBool(x, fallback) {
  return typeof x === "boolean" ? x : fallback;
}

function asAnimMode(x, fallback) {
  return x === "loop" || x === "sweep" ? x : fallback;
}

function asProjectionMode(x, fallback) {
  return x === "perspective" || x === "orthographic" ? x : fallback;
}

function asQuaternionMode(x, fallback) {
  return x === "fixed" || x === "drift" ? x : fallback;
}

function defaultPhaseOffset() {
  return { value: 0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
}

function readParam(input, fallback, ctx, path) {
  if (!isObject(input)) {
    if (input !== undefined) ctx.warnings.push(path + ": not an object, using default");
    return { ...fallback };
  }
  const result = {
    value: asNumber(input.value, fallback.value, ctx, path + ".value"),
    min: asNumber(input.min, fallback.min, ctx, path + ".min"),
    max: asNumber(input.max, fallback.max, ctx, path + ".max"),
    anim: asBool(input.anim, fallback.anim),
    animMode: asAnimMode(input.animMode, fallback.animMode),
    ratio: asNumber(input.ratio, fallback.ratio, ctx, path + ".ratio"),
  };
  if (input.phaseOffset !== undefined) {
    result.phaseOffset = readParam(input.phaseOffset, defaultPhaseOffset(), ctx, path + ".phaseOffset");
  }
  return result;
}

function readToggleParam(input, fallback, ctx, path) {
  const base = readParam(input, fallback, ctx, path);
  const on = isObject(input) ? asBool(input.on, fallback.on) : fallback.on;
  return { ...base, on };
}

function readLayer(input, ctx, path) {
  if (!isObject(input)) {
    ctx.warnings.push(path + ": not an object, dropping layer");
    return null;
  }
  const type = asString(input.type, "");
  if (!type) {
    ctx.warnings.push(path + ": missing layer type, dropping layer");
    return null;
  }
  const id = asString(input.id, "") || generateLayerId();
  const params = {};
  if (isObject(input.params)) {
    for (const k of Object.keys(input.params)) {
      params[k] = readParam(input.params[k], defaultParam(0, -1, 1), ctx, path + ".params." + k);
    }
  }
  return {
    id,
    type,
    enabled: asBool(input.enabled, true),
    blend: readParam(input.blend, defaultBlend(), ctx, path + ".blend"),
    blendTimeOverride: typeof input.blendTimeOverride === "number" ? input.blendTimeOverride : null,
    params,
  };
}

function readLayerArray(input, ctx, path) {
  if (!Array.isArray(input)) return [];
  return input.map((entry, i) => readLayer(entry, ctx, path + "[" + i + "]")).filter(l => l !== null);
}

function readCamera(input, ctx) {
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

function readQuaternion(input, ctx) {
  const fallback = defaultQuaternion();
  if (!isObject(input)) return fallback;
  const axes = isObject(input.axes) ? input.axes : {};
  const readAxis = (raw, fb, path) => {
    if (!isObject(raw)) return { ...fb };
    return {
      value: asNumber(raw.value, fb.value, ctx, path + ".value"),
      anim: asBool(raw.anim, fb.anim),
      ratio: asNumber(raw.ratio, fb.ratio, ctx, path + ".ratio"),
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

function readSampling(input, ctx) {
  const fallback = defaultSampling();
  if (!isObject(input)) return fallback;
  return {
    scale: readToggleParam(input.scale, fallback.scale, ctx, "sampling.scale"),
    density: readToggleParam(input.density, fallback.density, ctx, "sampling.density"),
  };
}

function readDisplay(input) {
  const fb = defaultDisplay();
  if (!isObject(input)) return fb;
  return {
    showFormula: asBool(input.showFormula, fb.showFormula),
    showDimCount: asBool(input.showDimCount, fb.showDimCount),
    showActiveLayers: asBool(input.showActiveLayers, fb.showActiveLayers),
    showParamValues: asBool(input.showParamValues, fb.showParamValues),
  };
}

function readMetadata(input) {
  const fb = defaultMetadata();
  if (!isObject(input)) return fb;
  return {
    name: asString(input.name, fb.name),
    description: asString(input.description, fb.description),
    author: asString(input.author, fb.author),
    created: asString(input.created, fb.created),
    modified: asString(input.modified, fb.modified),
    tags: Array.isArray(input.tags) ? input.tags.filter(t => typeof t === "string") : [],
  };
}

function readDefaults(input, ctx) {
  const fb = defaultDefaults();
  if (!isObject(input)) return fb;
  return { blendTime: asNumber(input.blendTime, fb.blendTime, ctx, "defaults.blendTime") };
}

function validatePreset(raw) {
  if (!isObject(raw)) {
    return { ok: false, error: "Preset is not a JSON object." };
  }
  const ctx = { warnings: [] };
  const schema = asString(raw.schema, "");
  if (schema && schema !== SCHEMA_NAME) {
    ctx.warnings.push('Schema name "' + schema + '" is unrecognized; expected "' + SCHEMA_NAME + '".');
  }
  const version = asNumber(raw.version, SCHEMA_VERSION, ctx, "version");
  if (version > SCHEMA_VERSION) {
    ctx.warnings.push("Preset is from a newer schema version (" + version + " > " + SCHEMA_VERSION + "). Some fields may be ignored.");
  } else if (version < SCHEMA_VERSION) {
    ctx.warnings.push("Preset is from an older schema version (" + version + " < " + SCHEMA_VERSION + "). Loaded with current defaults.");
  }
  const pipeline = isObject(raw.pipeline)
    ? {
        operations: readLayerArray(raw.pipeline.operations, ctx, "pipeline.operations"),
        lift: readLayerArray(raw.pipeline.lift, ctx, "pipeline.lift"),
      }
    : { operations: [], lift: [] };
  const preset = {
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

// --- registry for preset reconciliation ---

const REGISTRY = {
  operations: Object.fromEntries(
    Object.entries(TRANSFORMS).map(([key, def]) => [key, { key, params: def.params }])
  ),
  lift: Object.fromEntries(
    Object.entries(DISPLACEMENTS).map(([key, def]) => [key, { key, params: def.params }])
  ),
};

// --- cloning helpers ---

function cloneParam(p) { const c = { ...p }; if (p.phaseOffset) c.phaseOffset = { ...p.phaseOffset }; return c; }
function cloneToggleParam(p) { const c = { ...p }; if (p.phaseOffset) c.phaseOffset = { ...p.phaseOffset }; return c; }
function cloneBlend(b) { const c = { ...b }; if (b.phaseOffset) c.phaseOffset = { ...b.phaseOffset }; return c; }

function cloneLayer(l) {
  const params = {};
  for (const k of Object.keys(l.params)) params[k] = cloneParam(l.params[k]);
  return { id: l.id, type: l.type, enabled: l.enabled, blend: cloneBlend(l.blend), blendTimeOverride: l.blendTimeOverride, params };
}

function cloneLayerStructure(p) {
  return { operations: p.operations.map(cloneLayer), lift: p.lift.map(cloneLayer) };
}

function cloneCamera(c) {
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

function cloneQuaternion(q) {
  return {
    on: q.on, mode: q.mode, anim: q.anim,
    angle: q.angle, angleRatio: q.angleRatio,
    axes: { ax: { ...q.axes.ax }, ay: { ...q.axes.ay }, az: { ...q.axes.az } },
    driftSpeed: q.driftSpeed,
  };
}

function cloneSampling(s) {
  return { scale: cloneToggleParam(s.scale), density: cloneToggleParam(s.density) };
}

// --- export: app state -> preset ---

function exportPreset(state, metadata) {
  const base = emptyPreset();
  const now = new Date().toISOString();
  return {
    ...base,
    metadata: { ...base.metadata, ...(metadata || {}), modified: now, created: (metadata && metadata.created) || base.metadata.created },
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

// --- import: JSON -> preset -> app state ---

function importPresetJSON(json, registry) {
  let raw;
  try { raw = JSON.parse(json); } catch (e) { return { ok: false, error: "Invalid JSON: " + e.message }; }
  return importPresetValue(raw, registry);
}

function importPresetValue(raw, registry) {
  const result = validatePreset(raw);
  if (!result.ok) return { ok: false, error: result.error };
  const { state, registryWarnings } = reconcileWithRegistry(result.preset, registry);
  return { ok: true, state, warnings: [...result.warnings, ...registryWarnings] };
}

function reconcileWithRegistry(preset, registry) {
  const warnings = [];
  const reconcileLayers = (layers, defs, kind) => {
    const result = [];
    for (const layer of layers) {
      const def = defs[layer.type];
      if (!def) { warnings.push('Unknown ' + kind + ' type "' + layer.type + '" - layer dropped.'); continue; }
      const reconciledParams = {};
      for (const pdef of def.params) {
        const existing = layer.params[pdef.key];
        if (existing) { reconciledParams[pdef.key] = existing; }
        else { reconciledParams[pdef.key] = { value: pdef.default, min: pdef.min, max: pdef.max, anim: false, animMode: pdef.periodic ? "loop" : "sweep", ratio: 1 }; }
      }
      for (const k of Object.keys(layer.params)) {
        if (!def.params.some(p => p.key === k)) { warnings.push(kind + ' "' + layer.type + '" has unknown param "' + k + '" - dropped.'); }
      }
      result.push({ ...layer, params: reconciledParams });
    }
    return result;
  };
  const pipeline = {
    operations: reconcileLayers(preset.pipeline.operations, registry.operations, "operation"),
    lift: reconcileLayers(preset.pipeline.lift, registry.lift, "lift"),
  };
  const state = {
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

// --- browser file helpers ---

function downloadPreset(preset, filename) {
  const json = JSON.stringify(preset, null, 2);
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  const safeName = (preset.metadata.name || "preset").replace(/[^a-z0-9-_ ]/gi, "_");
  a.download = filename || (safeName + ".polyform.json");
  a.click();
  URL.revokeObjectURL(url);
}

function pickAndLoadPreset(registry) {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json,.json";
    input.onchange = async () => {
      const file = input.files && input.files[0];
      if (!file) { resolve(null); return; }
      try {
        const text = await file.text();
        resolve(importPresetJSON(text, registry));
      } catch (e) { resolve({ ok: false, error: "Failed to read file: " + e.message }); }
    };
    input.click();
  });
}

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
    id: generateLayerId(),
    type, on: true, params,
    blend: { value: 1, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 },
    blendTimeOverride: null,
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
    id: generateLayerId(),
    type, on: true, params,
    blend: { value: 1, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 },
    blendTimeOverride: null,
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
    const b = layer.blend ? layer.blend.value : 1;
    x = x + (out[0] - x) * b;
    y = y + (out[1] - y) * b;
  }
  return [x, y];
}

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
    const b = layer.blend ? layer.blend.value : 1;
    cx += out[0] * b;
    cy += out[1] * b;
    cz += out[2] * b;
  }
  return [cx - x, cy - y, cz];
}

// Map operator labels to mathematical notation
const FORMULA_SYM = {
  // operations (R2 -> R2)
  "translate": "T", "scale": "S", "rotate": "R",
  "shear": "K", "2x2 matrix": "GL(2)",
  "z^n": "z^n", "1/z": "z^-1", "exp(z)": "e^z", "log(z)": "log z",
  "sin(z)": "sin z", "cos(z)": "cos z",
  "mobius (az+b)/(cz+d)": "(az+b)/(cz+d)",
  "z + c": "z+c",
  "spiral": "r e^(ik ln r)", "pinch / bulge": "r^p",
  "swirl": "e^(-r^2) d/dth",
  "radial sine": "r+a sin(wr)",
  "angle warp": "th+a sin(n th)", "sine warp": "sin(kx, ky)",
  "ripple": "r+a sin(kr)",
  "conjugate": "z*", "reflect": "-x",
  "|z|": "|z|", "mixed monomials": "x^a y^b",
  "blend (utility)": "I",
  // lift (displacements)
  "|.| -> z": "|T| -> z", "|T|-|z| -> z": "|T|-|z|",
  "log|det J| -> z": "ln|J|", "det J -> z": "det J",
  "Re(T) -> z": "Re(T)", "Im(T) -> z": "Im(T)", "arg(T) -> z": "arg(T)",
  "sine wave (z)": "sin(kx) -> z", "sine xy (z)": "sin(kx,ky) -> z",
  "gaussian bump (z)": "e^(-r^2) -> z",
  "spherical projection": "S^2", "conical lift": "C_r,th",
  "helical twist": "H_th,p", "noise displacement": "N_f,a",
  "ripple (z + xy)": "sin(kr) -> xyz",
};

function compositionString(layers, dispLayers, pipeline) {
  var sym = function(label) { return FORMULA_SYM[label] || label; };

  // Innermost: lattice
  var core = "L";

  // Operations: composed right-to-left (math convention)
  var ops = layers.filter(function(l) { return l.on; }).map(function(l) { return sym(TRANSFORMS[l.type] ? TRANSFORMS[l.type].label : l.type); });
  if (ops.length > 0) core = ops.reverse().join(" . ") + "(" + core + ")";

  // Lift: additive
  var dz = (dispLayers || []).filter(function(l) { return l.on; }).map(function(l) { return sym(DISPLACEMENTS[l.type] ? DISPLACEMENTS[l.type].label : l.type); });
  if (dz.length === 1) core = core + " + " + dz[0];
  else if (dz.length > 1) core = core + " + Sum[" + dz.join(", ") + "]";

  // Quaternion tumble
  if (pipeline.quat) core = "Q(" + core + ")";

  // View transform
  var vParts = [];
  if (pipeline.az) vParts.push("th");
  if (pipeline.el) vParts.push("ph");
  if (pipeline.d) vParts.push("r");
  if (pipeline.tx) vParts.push("tx");
  if (pipeline.ty) vParts.push("ty");
  if (pipeline.roll) vParts.push("rho");
  if (vParts.length > 0) core = "V_{" + vParts.join(",") + "}(" + core + ")";

  // Projection
  var pParts = [];
  if (pipeline.f) pParts.push("f");
  if (pipeline.sensor) pParts.push("s");
  if (pipeline.ortho) pParts.push("z_o");
  if (pipeline.anam) pParts.push("alpha");
  if (pipeline.sx) pParts.push("sx");
  if (pipeline.sy) pParts.push("sy");
  if (pParts.length > 0) core = "P_{" + pParts.join(",") + "}(" + core + ")";

  // Distortion: outermost
  var dParts = [];
  if (pipeline.k1) dParts.push("k1");
  if (pipeline.k2) dParts.push("k2");
  if (dParts.length > 0) core = "D_{" + dParts.join(",") + "}(" + core + ")";

  return core;
}

function countDOFs(layers, dispLayers, view) {
  const active = [{ label: "base 2-manifold", struct: 2, temp: 0 }];
  let struct = 2, temp = 0;
  const tally = (layer, registry) => {
    if (!layer.on) return;
    const def = registry[layer.type];
    if (!def) return;
    const s = def.params.length + 1; // +1 for blend
    let t = 0, firstAnimRatio = null;
    const details = [];
    if (layer.blend && layer.blend.anim) {
      t += 1;
      if (firstAnimRatio === null) firstAnimRatio = layer.blend.ratio;
      details.push({ key: "blend", value: layer.blend.value, mode: layer.blend.animMode || "sweep", ratio: layer.blend.ratio, phaseOffset: (layer.blend.phaseOffset && layer.blend.phaseOffset.anim) ? layer.blend.phaseOffset.value : null });
    }
    if (layer.blend && layer.blend.phaseOffset && layer.blend.phaseOffset.anim) { t += 1; }
    for (const k in layer.params) {
      if (layer.params[k].anim) {
        t += 1;
        if (firstAnimRatio === null) firstAnimRatio = layer.params[k].ratio;
        details.push({ key: k, value: layer.params[k].value, mode: layer.params[k].animMode || "sweep", ratio: layer.params[k].ratio, phaseOffset: (layer.params[k].phaseOffset && layer.params[k].phaseOffset.anim) ? layer.params[k].phaseOffset.value : null });
      }
      if (layer.params[k].phaseOffset && layer.params[k].phaseOffset.anim) { t += 1; }
    }
    const blendValue = layer.blend ? layer.blend.value : 1;
    active.push({ label: def.label, struct: s, temp: t, ratio: firstAnimRatio, animated: t > 0, details, blendValue });
    struct += s; temp += t;
  };
  const opLabels = [];
  for (const layer of layers) {
    tally(layer, TRANSFORMS);
    if (layer.on && TRANSFORMS[layer.type]) opLabels.push(TRANSFORMS[layer.type].label);
  }
  for (const layer of (dispLayers || [])) {
    tally(layer, DISPLACEMENTS);
    if (layer.on && DISPLACEMENTS[layer.type]) opLabels.push(DISPLACEMENTS[layer.type].label);
  }
  for (const v of view) {
    if (!v.on) continue;
    const t = v.anim ? 1 : 0;
    const details = v.value !== undefined && v.value !== null
      ? [{ key: v.label, value: v.value, mode: v.anim ? (v.animMode || "sweep") : "-", ratio: v.ratio || 0, phaseOffset: null }]
      : [];
    active.push({ label: v.label, struct: v.struct, temp: t, ratio: v.anim ? v.ratio : null, animated: t > 0, details });
    struct += v.struct; temp += t;
    opLabels.push(v.label);
  }
  const operators = opLabels.length;
  const total = struct + temp;
  return { active, struct, temp, operators, total };
}

// ============================================================
// TRANSITION MANAGER
// ============================================================

function createTransitionManager() {
  const transitions = [];
  let nextId = 1;

  function startTransition(getRef, setRef, toValue, duration, onComplete) {
    const fromValue = getRef();
    const id = nextId++;
    transitions.push({ id, getRef, setRef, fromValue, toValue, startTime: null, duration, onComplete: onComplete || null });
    return id;
  }

  function cancelTransition(id) {
    const idx = transitions.findIndex(t => t.id === id);
    if (idx !== -1) transitions.splice(idx, 1);
  }

  function cancelTransitionsFor(setRef) {
    for (let i = transitions.length - 1; i >= 0; i--) {
      if (transitions[i].setRef === setRef) transitions.splice(i, 1);
    }
  }

  function tick(now) {
    for (let i = transitions.length - 1; i >= 0; i--) {
      const t = transitions[i];
      if (t.startTime === null) t.startTime = now;
      const elapsed = (now - t.startTime) / 1000;
      const progress = Math.min(1, elapsed / t.duration);
      const eased = 0.5 - 0.5 * Math.cos(progress * Math.PI);
      const value = t.fromValue + (t.toValue - t.fromValue) * eased;
      t.setRef(value);
      if (progress >= 1) {
        transitions.splice(i, 1);
        if (t.onComplete) t.onComplete();
      }
    }
  }

  return { startTransition, cancelTransition, cancelTransitionsFor, tick, transitions };
}

// ============================================================
// APP STATE <-> PRESET BRIDGE
// ============================================================

function layerToPresetLayer(layer) {
  const params = {};
  for (const k of Object.keys(layer.params)) {
    const p = layer.params[k];
    const exported = { value: p.value, min: p.min, max: p.max, anim: p.anim, animMode: p.animMode || "sweep", ratio: p.ratio };
    if (p.phaseOffset) {
      exported.phaseOffset = { value: p.phaseOffset.value, min: p.phaseOffset.min, max: p.phaseOffset.max, anim: p.phaseOffset.anim, animMode: p.phaseOffset.animMode || "sweep", ratio: p.phaseOffset.ratio };
    }
    params[k] = exported;
  }
  const blendExport = layer.blend ? { ...layer.blend } : defaultBlend();
  if (layer.blend && layer.blend.phaseOffset) {
    blendExport.phaseOffset = { value: layer.blend.phaseOffset.value, min: layer.blend.phaseOffset.min, max: layer.blend.phaseOffset.max, anim: layer.blend.phaseOffset.anim, animMode: layer.blend.phaseOffset.animMode || "sweep", ratio: layer.blend.phaseOffset.ratio };
  }
  return {
    id: layer.id,
    type: layer.type,
    enabled: layer.on,
    blend: blendExport,
    blendTimeOverride: layer.blendTimeOverride || null,
    params,
  };
}

function presetLayerToLayer(pl) {
  const registry = TRANSFORMS[pl.type] ? TRANSFORMS : DISPLACEMENTS;
  const def = registry[pl.type];
  const params = {};
  if (def) {
    for (const pdef of def.params) {
      const pp = pl.params[pdef.key];
      if (pp) {
        const restored = { ...pp, default: pdef.default, periodic: !!pdef.periodic };
        if (pp.phaseOffset) restored.phaseOffset = { ...pp.phaseOffset };
        params[pdef.key] = restored;
      } else {
        params[pdef.key] = {
          value: pdef.default, default: pdef.default,
          anim: false, animMode: pdef.periodic ? "loop" : "sweep", ratio: 1,
          min: pdef.min, max: pdef.max, periodic: !!pdef.periodic,
        };
      }
    }
  }
  const blend = pl.blend ? { ...pl.blend } : defaultBlend();
  if (pl.blend && pl.blend.phaseOffset) blend.phaseOffset = { ...pl.blend.phaseOffset };
  return {
    id: pl.id,
    type: pl.type,
    on: pl.enabled,
    params,
    blend,
    blendTimeOverride: pl.blendTimeOverride,
  };
}

function toggleParamToView(tp, extras) {
  return { ...tp, default: tp.value, ...(extras || {}) };
}

function viewToToggleParam(vp) {
  const result = { on: vp.on, value: vp.value, min: vp.min, max: vp.max, anim: vp.anim, animMode: vp.animMode || "sweep", ratio: vp.ratio };
  if (vp.phaseOffset) {
    result.phaseOffset = { value: vp.phaseOffset.value, min: vp.phaseOffset.min, max: vp.phaseOffset.max, anim: vp.phaseOffset.anim, animMode: vp.phaseOffset.animMode || "sweep", ratio: vp.phaseOffset.ratio };
  }
  return result;
}

// ============================================================
// COMPONENT
// ============================================================

export default function PolydimensionalCoupling() {
  const mountRef = useRef(null);
  const sceneRef = useRef(null);
  const stateRef = useRef({});
  const transitionsRef = useRef(createTransitionManager());
  const overlayCache = useRef({ dimAccount: null, systemInfo: null, lastUpdate: 0 });
  const fpsRef = useRef({ frames: 0, lastSample: performance.now(), value: 0 });

  const PARAM_NEUTRALS = useRef({
    orbitX: 0, orbitY: 0.1, zoomP: 3.5,
    targetX: 0, targetY: 0, roll: 0,
    focalLength: 35, sensorSize: 36, orthoZoom: 2,
    anamorphic: 1, shiftX: 0, shiftY: 0,
    distortK1: 0, distortK2: 0,
    scaleP: 0.6, densityP: 27,
  }).current;
  const savedValuesRef = useRef({});
  const viewTransitionIdsRef = useRef({});

  // "normal" | "fading-out" | "neutral" | "restoring"
  const [neutralizeState, setNeutralizeState] = useState("normal");
  const neutralizingRef = useRef(false);
  const neutralizeBypassRef = useRef(false);
  const neutralizeSnapshotRef = useRef(null);
  const [recordPhase, setRecordPhase] = useState("idle");
  const mediaRecorderRef = useRef(null);
  const recordedChunksRef = useRef([]);

  const [layers, setLayers] = useState([makeLayer("zPow")]);
  const [dispLayers, setDispLayers] = useState([makeDispLayer("zDisplacement")]);

  const [paletteKey, setPaletteKey] = useState("acid");
  const [tempo, setTempo] = useState(0.3);
  const [paused, setPaused] = useState(false);
  const animClockRef = useRef(0);

  const [importWarnings, setImportWarnings] = useState(null);

  // ===== Camera system =====
  const [orbitX, setOrbitX] = useState({ on: true, value: 0, default: 0, anim: false, animMode: "loop", ratio: 1, min: 0, max: Math.PI * 2, periodic: true });
  const [orbitY, setOrbitY] = useState({ on: true, value: 0.1, default: 0.1, anim: false, animMode: "sweep", ratio: 1/PHI, min: 0.1, max: Math.PI - 0.1, periodic: true });
  const [zoomP, setZoomP] = useState({ on: true, value: 3.5, default: 3.5, anim: false, animMode: "sweep", ratio: 1, min: 0.8, max: 12 });
  const [targetX, setTargetX] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -2, max: 2 });
  const [targetY, setTargetY] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -2, max: 2 });
  const [roll, setRoll] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "loop", ratio: 1, min: 0, max: Math.PI * 2, periodic: true });

  const [projection, setProjection] = useState("perspective");
  const [focalLength, setFocalLength] = useState({ on: true, value: 35, default: 35, anim: false, animMode: "sweep", ratio: 1, min: 8, max: 200 });
  const [sensorSize, setSensorSize] = useState({ on: true, value: 36, default: 36, anim: false, animMode: "sweep", ratio: 1, min: 4, max: 80 });
  const [orthoZoom, setOrthoZoom] = useState({ on: true, value: 2, default: 2, anim: false, animMode: "sweep", ratio: 1, min: 0.3, max: 6 });
  const [anamorphic, setAnamorphic] = useState({ on: false, value: 1, default: 1, anim: false, animMode: "sweep", ratio: 1, min: 0.3, max: 3 });
  const [shiftX, setShiftX] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });
  const [shiftY, setShiftY] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });

  const [distortK1, setDistortK1] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });
  const [distortK2, setDistortK2] = useState({ on: false, value: 0, default: 0, anim: false, animMode: "sweep", ratio: 1, min: -1, max: 1 });

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
  const [showDimCount, setShowDimCount] = useState(true);
  const [showActiveLayers, setShowActiveLayers] = useState(true);
  const [showParamValues, setShowParamValues] = useState(true);
  const [showHelp, setShowHelp] = useState(false);
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [showDispAddMenu, setShowDispAddMenu] = useState(false);

  const P = PALETTES[paletteKey];

  // ===== Preset Save/Load =====
  const getAppState = () => {
    return {
      tempo: { value: tempo, min: 0, max: 200 / 60, anim: false, animMode: "sweep", ratio: 1 },
      palette: paletteKey,
      pipeline: {
        operations: layers.map(layerToPresetLayer),
        lift: dispLayers.map(layerToPresetLayer),
      },
      camera: {
        position: {
          orbitX: viewToToggleParam(orbitX),
          orbitY: viewToToggleParam(orbitY),
          distance: viewToToggleParam(zoomP),
          targetX: viewToToggleParam(targetX),
          targetY: viewToToggleParam(targetY),
          roll: viewToToggleParam(roll),
        },
        lens: {
          projection,
          focalLength: viewToToggleParam(focalLength),
          sensorSize: viewToToggleParam(sensorSize),
          orthoZoom: viewToToggleParam(orthoZoom),
          anamorphic: viewToToggleParam(anamorphic),
          shiftX: viewToToggleParam(shiftX),
          shiftY: viewToToggleParam(shiftY),
        },
        distortion: {
          k1: viewToToggleParam(distortK1),
          k2: viewToToggleParam(distortK2),
        },
      },
      quaternion: {
        on: quat.on, mode: quat.mode, anim: quat.anim,
        angle: quat.angle, angleRatio: quat.angleRatio,
        axes: {
          ax: { value: quat.ax, anim: quat.axAnim, ratio: quat.axRatio },
          ay: { value: quat.ay, anim: quat.ayAnim, ratio: quat.ayRatio },
          az: { value: quat.az, anim: quat.azAnim, ratio: quat.azRatio },
        },
        driftSpeed: quat.driftSpeed,
      },
      sampling: {
        scale: viewToToggleParam(scaleP),
        density: viewToToggleParam(densityP),
      },
      display: { showFormula, showDimCount, showActiveLayers, showParamValues },
      defaults: { blendTime: 1.0 },
    };
  };

  const applyAppState = (state) => {
    setTempo(state.tempo.value);
    setPaletteKey(state.palette);
    setLayers(state.pipeline.operations.map(presetLayerToLayer));
    setDispLayers(state.pipeline.lift.map(presetLayerToLayer));

    const cp = state.camera.position;
    setOrbitX(p => ({ ...p, ...toggleParamToView(cp.orbitX, { periodic: true }) }));
    setOrbitY(p => ({ ...p, ...toggleParamToView(cp.orbitY, { periodic: true }) }));
    setZoomP(p => ({ ...p, ...toggleParamToView(cp.distance) }));
    setTargetX(p => ({ ...p, ...toggleParamToView(cp.targetX) }));
    setTargetY(p => ({ ...p, ...toggleParamToView(cp.targetY) }));
    setRoll(p => ({ ...p, ...toggleParamToView(cp.roll, { periodic: true }) }));

    setProjection(state.camera.lens.projection);
    const cl = state.camera.lens;
    setFocalLength(p => ({ ...p, ...toggleParamToView(cl.focalLength) }));
    setSensorSize(p => ({ ...p, ...toggleParamToView(cl.sensorSize) }));
    setOrthoZoom(p => ({ ...p, ...toggleParamToView(cl.orthoZoom) }));
    setAnamorphic(p => ({ ...p, ...toggleParamToView(cl.anamorphic) }));
    setShiftX(p => ({ ...p, ...toggleParamToView(cl.shiftX) }));
    setShiftY(p => ({ ...p, ...toggleParamToView(cl.shiftY) }));

    const cd = state.camera.distortion;
    setDistortK1(p => ({ ...p, ...toggleParamToView(cd.k1) }));
    setDistortK2(p => ({ ...p, ...toggleParamToView(cd.k2) }));

    setScaleP(p => ({ ...p, ...toggleParamToView(state.sampling.scale) }));
    setDensityP(p => ({ ...p, ...toggleParamToView(state.sampling.density) }));

    const q = state.quaternion;
    setQuat({
      on: q.on, mode: q.mode, anim: q.anim,
      angle: q.angle, angleRatio: q.angleRatio,
      ax: q.axes.ax.value, ay: q.axes.ay.value, az: q.axes.az.value,
      axRatio: q.axes.ax.ratio, ayRatio: q.axes.ay.ratio, azRatio: q.axes.az.ratio,
      axAnim: q.axes.ax.anim, ayAnim: q.axes.ay.anim, azAnim: q.axes.az.anim,
      driftSpeed: q.driftSpeed,
    });

    setShowFormula(state.display.showFormula);
    setShowDimCount(state.display.showDimCount);
    setShowActiveLayers(state.display.showActiveLayers);
    setShowParamValues(state.display.showParamValues);
  };

  const handleSave = () => {
    const state = getAppState();
    const preset = exportPreset(state);
    downloadPreset(preset);
  };

  const handleLoad = async () => {
    const result = await pickAndLoadPreset(REGISTRY);
    if (!result) return;
    if (!result.ok) { alert("Failed to load preset: " + result.error); return; }
    applyAppState(result.state);
    if (result.warnings.length > 0) setImportWarnings(result.warnings);
  };

  const viewSummaries = [
    { on: orbitX.on, label: "camera azimuth", struct: 1, anim: orbitX.anim, animMode: orbitX.animMode, ratio: orbitX.ratio, value: orbitX.value },
    { on: orbitY.on, label: "camera elevation", struct: 1, anim: orbitY.anim, animMode: orbitY.animMode, ratio: orbitY.ratio, value: orbitY.value },
    { on: zoomP.on, label: "camera distance", struct: 1, anim: zoomP.anim, animMode: zoomP.animMode, ratio: zoomP.ratio, value: zoomP.value },
    { on: targetX.on, label: "target offset x", struct: 1, anim: targetX.anim, animMode: targetX.animMode, ratio: targetX.ratio, value: targetX.value },
    { on: targetY.on, label: "target offset y", struct: 1, anim: targetY.anim, animMode: targetY.animMode, ratio: targetY.ratio, value: targetY.value },
    { on: roll.on, label: "camera roll", struct: 1, anim: roll.anim, animMode: roll.animMode, ratio: roll.ratio, value: roll.value },
    { on: projection === "perspective" && focalLength.on, label: "focal length", struct: 1, anim: focalLength.anim, animMode: focalLength.animMode, ratio: focalLength.ratio, value: focalLength.value },
    { on: projection === "perspective" && sensorSize.on, label: "sensor size", struct: 1, anim: sensorSize.anim, animMode: sensorSize.animMode, ratio: sensorSize.ratio, value: sensorSize.value },
    { on: projection === "orthographic" && orthoZoom.on, label: "ortho zoom", struct: 1, anim: orthoZoom.anim, animMode: orthoZoom.animMode, ratio: orthoZoom.ratio, value: orthoZoom.value },
    { on: anamorphic.on, label: "anamorphic squeeze", struct: 1, anim: anamorphic.anim, animMode: anamorphic.animMode, ratio: anamorphic.ratio, value: anamorphic.value },
    { on: shiftX.on, label: "lens shift x", struct: 1, anim: shiftX.anim, animMode: shiftX.animMode, ratio: shiftX.ratio, value: shiftX.value },
    { on: shiftY.on, label: "lens shift y", struct: 1, anim: shiftY.anim, animMode: shiftY.animMode, ratio: shiftY.ratio, value: shiftY.value },
    { on: distortK1.on, label: "distortion k1", struct: 1, anim: distortK1.anim, animMode: distortK1.animMode, ratio: distortK1.ratio, value: distortK1.value },
    { on: distortK2.on, label: "distortion k2", struct: 1, anim: distortK2.anim, animMode: distortK2.animMode, ratio: distortK2.ratio, value: distortK2.value },
    { on: scaleP.on, label: "lattice scale", struct: 1, anim: scaleP.anim, animMode: scaleP.animMode, ratio: scaleP.ratio, value: scaleP.value },
    { on: densityP.on, label: "lattice density", struct: 1, anim: densityP.anim, animMode: densityP.animMode, ratio: densityP.ratio, value: densityP.value },
  ];
  if (quat.on) {
    const phases = (quat.anim ? 1 : 0) + (quat.mode === "drift" ? 1 : 0)
      + (quat.axAnim ? 1 : 0) + (quat.ayAnim ? 1 : 0) + (quat.azAnim ? 1 : 0);
    viewSummaries.push({ on: true, label: "SO(3) tumble", struct: 3, anim: phases > 0, ratio: quat.angleRatio, value: quat.angle });
  }
  // compString only changes on structural toggle, always recompute (cheap)
  const compString = compositionString(layers, dispLayers, {
    quat: quat.on,
    az: orbitX.on, el: orbitY.on, d: zoomP.on,
    tx: targetX.on, ty: targetY.on, roll: roll.on,
    f: projection === "perspective" && focalLength.on,
    sensor: projection === "perspective" && sensorSize.on,
    ortho: projection === "orthographic" && orthoZoom.on,
    anam: anamorphic.on, sx: shiftX.on, sy: shiftY.on,
    k1: distortK1.on, k2: distortK2.on,
  });

  // Throttle overlay data to ~8 fps (values change every anim frame but readout doesn't need 60fps)
  // Skip computation entirely when nothing needs it
  const overlayActive = showDimCount || showActiveLayers;
  const now = performance.now();
  if (overlayActive && (!overlayCache.current.dimAccount || now - overlayCache.current.lastUpdate > 125)) {
    overlayCache.current.dimAccount = countDOFs(layers, dispLayers, viewSummaries);
    overlayCache.current.systemInfo = {
      palette: paletteKey,
      projection,
      fov: projection === "perspective"
        ? (2 * Math.atan((sensorSize.value || 36) / (2 * (focalLength.value || 35))) * 180 / Math.PI).toFixed(1)
        : null,
      orthoZ: projection === "orthographic" ? orthoZoom.value.toFixed(2) : null,
      orbitX: orbitX.on ? (orbitX.value / Math.PI).toFixed(2) + "pi" : null,
      orbitY: orbitY.on ? (orbitY.value / Math.PI).toFixed(2) + "pi" : null,
      distance: zoomP.on ? zoomP.value.toFixed(2) : null,
      scale: scaleP.value.toFixed(2),
      density: Math.round(densityP.value),
      quatActive: quat.on,
      quatAngle: quat.on ? (quat.angle / Math.PI).toFixed(3) + "pi" : null,
      quatAxis: quat.on ? "(" + quat.ax.toFixed(2) + ", " + quat.ay.toFixed(2) + ", " + quat.az.toFixed(2) + ")" : null,
      fps: fpsRef.current.value,
    };
    overlayCache.current.lastUpdate = now;
  }
  const dimAccount = overlayActive ? overlayCache.current.dimAccount : null;
  const systemInfo = overlayActive ? overlayCache.current.systemInfo : null;

  neutralizingRef.current = neutralizeState !== "normal";

  stateRef.current = {
    layers, dispLayers, P, paused,
    orbitX, orbitY, zoomP, targetX, targetY, roll,
    projection, focalLength, sensorSize, orthoZoom, anamorphic, shiftX, shiftY,
    distortK1, distortK2,
    scaleP, densityP, quat,
  };

  // ===== Layer toggle with blend transition =====
  const toggleLayerOn = (id) => {
    if (neutralizingRef.current && !neutralizeBypassRef.current) return;
    const tm = transitionsRef.current;
    setLayers(ls => ls.map(l => {
      if (l.id !== id) return l;
      if (l.on) {
        // Fade out: animate blend -> 0, then disable
        tm.startTransition(
          () => l.blend.value,
          (v) => setLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
          0, l.blendTimeOverride || 0.5,
          () => setLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, on: false } : l2))
        );
        return l;
      } else {
        // Enable with blend=0, then fade in
        const updated = { ...l, on: true, blend: { ...l.blend, value: 0 } };
        tm.startTransition(
          () => 0,
          (v) => setLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
          1, l.blendTimeOverride || 0.5
        );
        return updated;
      }
    }));
  };

  const toggleDispLayerOn = (id) => {
    if (neutralizingRef.current && !neutralizeBypassRef.current) return;
    const tm = transitionsRef.current;
    setDispLayers(ls => ls.map(l => {
      if (l.id !== id) return l;
      if (l.on) {
        tm.startTransition(
          () => l.blend.value,
          (v) => setDispLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
          0, l.blendTimeOverride || 0.5,
          () => setDispLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, on: false } : l2))
        );
        return l;
      } else {
        const updated = { ...l, on: true, blend: { ...l.blend, value: 0 } };
        tm.startTransition(
          () => 0,
          (v) => setDispLayers(ls2 => ls2.map(l2 => l2.id === id ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
          1, l.blendTimeOverride || 0.5
        );
        return updated;
      }
    }));
  };

  // ===== Reorder via fade =====
  const moveLayerFade = (id, dir) => {
    if (neutralizingRef.current) return;
    const tm = transitionsRef.current;
    setLayers(ls => {
      const idx = ls.findIndex(l => l.id === id);
      if (idx === -1) return ls;
      const newIdx = idx + dir;
      if (newIdx < 0 || newIdx >= ls.length) return ls;
      const layerId = ls[idx].id;
      // Phase 1: fade out
      tm.startTransition(
        () => ls[idx].blend.value,
        (v) => setLayers(ls2 => ls2.map(l2 => l2.id === layerId ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
        0, 0.3,
        () => {
          // Phase 2: swap, then fade in
          setLayers(ls2 => {
            const i = ls2.findIndex(l => l.id === layerId);
            if (i === -1) return ls2;
            const ni = i + dir;
            if (ni < 0 || ni >= ls2.length) return ls2;
            const next = [...ls2];
            [next[i], next[ni]] = [next[ni], next[i]];
            return next;
          });
          tm.startTransition(
            () => 0,
            (v) => setLayers(ls2 => ls2.map(l2 => l2.id === layerId ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
            1, 0.3
          );
        }
      );
      return ls;
    });
  };

  const moveDispLayerFade = (id, dir) => {
    if (neutralizingRef.current) return;
    const tm = transitionsRef.current;
    setDispLayers(ls => {
      const idx = ls.findIndex(l => l.id === id);
      if (idx === -1) return ls;
      const newIdx = idx + dir;
      if (newIdx < 0 || newIdx >= ls.length) return ls;
      const layerId = ls[idx].id;
      tm.startTransition(
        () => ls[idx].blend.value,
        (v) => setDispLayers(ls2 => ls2.map(l2 => l2.id === layerId ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
        0, 0.3,
        () => {
          setDispLayers(ls2 => {
            const i = ls2.findIndex(l => l.id === layerId);
            if (i === -1) return ls2;
            const ni = i + dir;
            if (ni < 0 || ni >= ls2.length) return ls2;
            const next = [...ls2];
            [next[i], next[ni]] = [next[ni], next[i]];
            return next;
          });
          tm.startTransition(
            () => 0,
            (v) => setDispLayers(ls2 => ls2.map(l2 => l2.id === layerId ? { ...l2, blend: { ...l2.blend, value: v } } : l2)),
            1, 0.3
          );
        }
      );
      return ls;
    });
  };

  const toggleViewParam = (paramName, setParam) => {
    if (neutralizingRef.current && !neutralizeBypassRef.current) return;
    const tm = transitionsRef.current;
    const neutral = PARAM_NEUTRALS[paramName];

    setParam(p => {
      if (p.on) {
        // Toggle OFF: save current value, animate to neutral
        savedValuesRef.current[paramName] = p.value;
        if (viewTransitionIdsRef.current[paramName]) {
          tm.cancelTransition(viewTransitionIdsRef.current[paramName]);
        }
        const tid = tm.startTransition(
          () => p.value,
          (v) => setParam(pp => ({ ...pp, value: v })),
          neutral, 0.5,
          () => {
            setParam(pp => ({ ...pp, on: false, anim: false }));
            delete viewTransitionIdsRef.current[paramName];
          }
        );
        viewTransitionIdsRef.current[paramName] = tid;
        return { ...p, anim: false };
      } else {
        // Toggle ON: set value to neutral, animate toward saved value
        const target = savedValuesRef.current[paramName] !== undefined
          ? savedValuesRef.current[paramName] : p.default !== undefined ? p.default : neutral;
        if (viewTransitionIdsRef.current[paramName]) {
          tm.cancelTransition(viewTransitionIdsRef.current[paramName]);
        }
        const tid = tm.startTransition(
          () => neutral,
          (v) => setParam(pp => ({ ...pp, value: v })),
          target, 0.5,
          () => { delete viewTransitionIdsRef.current[paramName]; }
        );
        viewTransitionIdsRef.current[paramName] = tid;
        return { ...p, on: true, value: neutral };
      }
    });
  };

  const toggleQuat = () => {
    if (neutralizingRef.current && !neutralizeBypassRef.current) return;
    const tm = transitionsRef.current;
    setQuat(q => {
      if (q.on) {
        // Toggle OFF: save state, animate angle toward 0
        savedValuesRef.current._quat = { angle: q.angle, anim: q.anim };
        if (viewTransitionIdsRef.current._quat) {
          tm.cancelTransition(viewTransitionIdsRef.current._quat);
        }
        const currentAngle = q.angle;
        const tid = tm.startTransition(
          () => currentAngle,
          (v) => setQuat(qq => ({ ...qq, angle: v })),
          0, 0.5,
          () => {
            setQuat(qq => ({ ...qq, on: false, anim: false }));
            delete viewTransitionIdsRef.current._quat;
          }
        );
        viewTransitionIdsRef.current._quat = tid;
        return { ...q, anim: false };
      } else {
        // Toggle ON: start at angle=0 (identity), animate toward saved angle
        const saved = savedValuesRef.current._quat;
        const targetAngle = saved ? saved.angle : 0;
        const restoreAnim = saved ? saved.anim : false;
        if (viewTransitionIdsRef.current._quat) {
          tm.cancelTransition(viewTransitionIdsRef.current._quat);
        }
        const tid = tm.startTransition(
          () => 0,
          (v) => setQuat(qq => ({ ...qq, angle: v })),
          targetAngle, 0.5,
          () => {
            if (restoreAnim) setQuat(qq => ({ ...qq, anim: true }));
            delete viewTransitionIdsRef.current._quat;
          }
        );
        viewTransitionIdsRef.current._quat = tid;
        return { ...q, on: true, angle: 0, anim: false };
      }
    });
  };

  const smoothSetTempo = (target) => {
    const tm = transitionsRef.current;
    if (viewTransitionIdsRef.current._tempo) {
      tm.cancelTransition(viewTransitionIdsRef.current._tempo);
    }
    const current = tempo;
    const tid = tm.startTransition(
      () => current,
      (v) => setTempo(v),
      target, 0.3,
      () => { delete viewTransitionIdsRef.current._tempo; }
    );
    viewTransitionIdsRef.current._tempo = tid;
  };

  // ===== Neutralize: peel away effects in reverse pipeline order =====
  const BLEND_DELAY = 500;

  const bypassCall = (fn) => {
    neutralizeBypassRef.current = true;
    fn();
    neutralizeBypassRef.current = false;
  };

  // Phase 1: turn everything off in reverse pipeline order, store snapshot
  const runNeutralizeOff = (onComplete) => {
    let delay = 0;
    const wasOn = { distort: [], lens: [], position: [], quat: false, disps: [], ops: [], latticeAnim: [] };

    // 1. Distortion
    const distortParams = [
      ["distortK1", setDistortK1, distortK1],
      ["distortK2", setDistortK2, distortK2],
    ];
    let anyDistort = false;
    for (const [name, setter, param] of distortParams) {
      if (param.on) { wasOn.distort.push([name, setter]); anyDistort = true; }
    }
    if (anyDistort) {
      const d = delay;
      for (const [name, setter] of wasOn.distort) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      delay += BLEND_DELAY;
    }

    // 2. Camera lens (includes focal length, sensor size, ortho zoom)
    const lensParams = [
      ["focalLength", setFocalLength, focalLength],
      ["sensorSize", setSensorSize, sensorSize],
      ["orthoZoom", setOrthoZoom, orthoZoom],
      ["anamorphic", setAnamorphic, anamorphic],
      ["shiftX", setShiftX, shiftX],
      ["shiftY", setShiftY, shiftY],
    ];
    let anyLens = false;
    for (const [name, setter, param] of lensParams) {
      if (param.on) { wasOn.lens.push([name, setter]); anyLens = true; }
    }
    if (anyLens) {
      const d = delay;
      for (const [name, setter] of wasOn.lens) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      delay += BLEND_DELAY;
    }

    // 3. Camera position
    const posParams = [
      ["orbitX", setOrbitX, orbitX],
      ["orbitY", setOrbitY, orbitY],
      ["zoomP", setZoomP, zoomP],
      ["targetX", setTargetX, targetX],
      ["targetY", setTargetY, targetY],
      ["roll", setRoll, roll],
    ];
    let anyPos = false;
    for (const [name, setter, param] of posParams) {
      if (param.on) { wasOn.position.push([name, setter]); anyPos = true; }
    }
    if (anyPos) {
      const d = delay;
      for (const [name, setter] of wasOn.position) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      delay += BLEND_DELAY;
    }

    // 4. Quaternion
    if (quat.on) {
      wasOn.quat = true;
      const d = delay;
      setTimeout(() => bypassCall(() => toggleQuat()), d);
      delay += BLEND_DELAY;
    }

    // 5. Lift layers: last -> first
    const currentDisps = [...dispLayers];
    for (let i = currentDisps.length - 1; i >= 0; i--) {
      if (currentDisps[i].on) {
        wasOn.disps.unshift(currentDisps[i].id);
        const d = delay;
        const lid = currentDisps[i].id;
        setTimeout(() => bypassCall(() => toggleDispLayerOn(lid)), d);
        delay += BLEND_DELAY;
      }
    }

    // 6. Operation layers: last -> first
    const currentOps = [...layers];
    for (let i = currentOps.length - 1; i >= 0; i--) {
      if (currentOps[i].on) {
        wasOn.ops.unshift(currentOps[i].id);
        const d = delay;
        const lid = currentOps[i].id;
        setTimeout(() => bypassCall(() => toggleLayerOn(lid)), d);
        delay += BLEND_DELAY;
      }
    }

    // 7. Lattice: don't toggle off, just freeze animation if running
    const latticeParams = [
      ["scaleP", setScaleP, scaleP],
      ["densityP", setDensityP, densityP],
    ];
    for (const [name, setter, param] of latticeParams) {
      if (param.anim) {
        wasOn.latticeAnim.push([name, setter]);
        setter(p => ({ ...p, anim: false }));
      }
    }

    neutralizeSnapshotRef.current = wasOn;
    setTimeout(() => { if (onComplete) onComplete(); }, delay + BLEND_DELAY);
  };

  // Phase 2: restore everything from snapshot in forward pipeline order
  const runNeutralizeRestore = (onComplete) => {
    const wasOn = neutralizeSnapshotRef.current;
    if (!wasOn) { if (onComplete) onComplete(); return; }
    let rDelay = 0;

    // 0. Lattice: restore animation immediately (no delay needed, just re-enable)
    for (const [name, setter] of wasOn.latticeAnim) {
      setter(p => ({ ...p, anim: true }));
    }

    // 1. Operations: first -> last
    for (const id of wasOn.ops) {
      const d = rDelay;
      setTimeout(() => bypassCall(() => toggleLayerOn(id)), d);
      rDelay += BLEND_DELAY;
    }

    // 2. Lift layers: first -> last
    for (const id of wasOn.disps) {
      const d = rDelay;
      setTimeout(() => bypassCall(() => toggleDispLayerOn(id)), d);
      rDelay += BLEND_DELAY;
    }

    // 3. Quaternion
    if (wasOn.quat) {
      const d = rDelay;
      setTimeout(() => bypassCall(() => toggleQuat()), d);
      rDelay += BLEND_DELAY;
    }

    // 4. Camera position
    if (wasOn.position.length > 0) {
      const d = rDelay;
      for (const [name, setter] of wasOn.position) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      rDelay += BLEND_DELAY;
    }

    // 5. Camera lens
    if (wasOn.lens.length > 0) {
      const d = rDelay;
      for (const [name, setter] of wasOn.lens) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      rDelay += BLEND_DELAY;
    }

    // 6. Distortion
    if (wasOn.distort.length > 0) {
      const d = rDelay;
      for (const [name, setter] of wasOn.distort) {
        setTimeout(() => bypassCall(() => toggleViewParam(name, setter)), d);
      }
      rDelay += BLEND_DELAY;
    }

    setTimeout(() => {
      neutralizeSnapshotRef.current = null;
      if (onComplete) onComplete();
    }, rDelay + BLEND_DELAY);
  };

  // Full cycle: off -> callback at neutral -> restore -> callback at end (used by record)
  const runNeutralizeCycle = (onNeutralReached, onRestoreComplete) => {
    runNeutralizeOff(() => {
      if (onNeutralReached) onNeutralReached();
      runNeutralizeRestore(onRestoreComplete);
    });
  };

  const exportPNG = () => {
    if (!sceneRef.current) return;
    const gl = sceneRef.current.renderer.domElement;
    const w = gl.width, h = gl.height;
    const composite = document.createElement("canvas");
    composite.width = w;
    composite.height = h;
    const ctx2d = composite.getContext("2d");

    // Draw WebGL canvas
    ctx2d.drawImage(gl, 0, 0);

    // Draw overlays if active
    const s = stateRef.current;
    const dpr = window.devicePixelRatio || 1;

    if (showFormula) {
      ctx2d.font = "italic " + (14 * dpr) + "px Georgia, serif";
      ctx2d.fillStyle = s.P.warp;
      ctx2d.shadowColor = s.P.glow + "aa";
      ctx2d.shadowBlur = 8 * dpr;
      ctx2d.fillText(compString, 12 * dpr, h - 12 * dpr);
      ctx2d.shadowBlur = 0;
    }

    if (overlayActive && dimAccount) {
      const fontSize = 9 * dpr;
      ctx2d.font = fontSize + "px ui-monospace, monospace";
      ctx2d.fillStyle = s.P.grid;
      ctx2d.globalAlpha = 0.7;
      const lh = fontSize * 1.6;

      // Left column: active layers
      if (showActiveLayers) {
        var y = 12 * dpr + fontSize;
        ctx2d.globalAlpha = 0.35;
        ctx2d.fillText("active layers", 12 * dpr, y);
        y += lh;
        ctx2d.globalAlpha = 0.7;
        for (var ai = 0; ai < dimAccount.active.length; ai++) {
          var a = dimAccount.active[ai];
          var line = a.label + " . " + a.struct + (a.temp > 0 ? "+" + a.temp + "t" : "");
          if (a.blendValue !== undefined && a.blendValue < 1.0) {
            line += " . blend " + a.blendValue.toFixed(2);
          }
          ctx2d.globalAlpha = a.animated ? 0.7 : 0.45;
          ctx2d.fillText(line, 12 * dpr, y);
          y += lh;
          // detail lines for animated params
          if (showParamValues && a.details) {
            for (var di = 0; di < a.details.length; di++) {
              var d = a.details[di];
              ctx2d.globalAlpha = 0.5;
              var dl = "  " + d.key + "=" + (typeof d.value === "number" ? d.value.toFixed(3) : d.value) + " . " + d.mode + " . x" + d.ratio.toFixed(4);
              if (d.phaseOffset !== null && d.phaseOffset !== undefined) {
                dl += " . phase " + d.phaseOffset.toFixed(3);
              }
              ctx2d.fillText(dl, 12 * dpr, y);
              y += lh;
            }
          }
        }
      }

      // Right column: dim count + system info
      if (showDimCount) {
        ctx2d.textAlign = "right";
        var rx = w - 12 * dpr;
        var ry = 12 * dpr + fontSize;
        ctx2d.globalAlpha = 0.35;
        ctx2d.fillText("dim count", rx, ry); ry += lh;
        ctx2d.globalAlpha = 0.7;
        ctx2d.fillText("structural . " + dimAccount.struct, rx, ry); ry += lh;
        ctx2d.fillText("temporal . " + dimAccount.temp, rx, ry); ry += lh;
        ctx2d.fillStyle = s.P.warp;
        ctx2d.fillText("total . " + dimAccount.total + "D", rx, ry); ry += lh;
        ctx2d.fillStyle = s.P.grid;
        ctx2d.globalAlpha = 0.45;
        ctx2d.fillText("operators . " + dimAccount.operators, rx, ry); ry += lh;
        ctx2d.globalAlpha = 0.6;
        ctx2d.fillText("tempo . " + Math.round(tempo * 60) + " bpm" + (paused ? " . paused" : ""), rx, ry); ry += lh;
        // system info lines
        ctx2d.fillText("lattice . " + systemInfo.scale + " x " + systemInfo.density, rx, ry); ry += lh;
        ctx2d.fillText(systemInfo.projection + (systemInfo.fov ? " . " + systemInfo.fov + " deg" : "") + (systemInfo.orthoZ ? " . z" + systemInfo.orthoZ : ""), rx, ry); ry += lh;
        ctx2d.fillText("palette . " + systemInfo.palette, rx, ry); ry += lh;
        if (systemInfo.quatActive) {
          ctx2d.fillText("quat . " + systemInfo.quatAngle + " . axis " + systemInfo.quatAxis, rx, ry); ry += lh;
        }
        ctx2d.textAlign = "left";
      }

      ctx2d.globalAlpha = 1;
    }

    // Download composite
    var dataURL = composite.toDataURL("image/png");
    var link = document.createElement("a");
    link.href = dataURL;
    link.download = "polyform-" + Date.now() + ".png";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  // ===== Neutralize + Record handlers =====
  const handleNeutralize = () => {
    if (neutralizeState === "fading-out" || neutralizeState === "restoring") return;
    if (neutralizeState === "normal") {
      setNeutralizeState("fading-out");
      runNeutralizeOff(() => setNeutralizeState("neutral"));
    } else {
      // "neutral" -> restore
      setNeutralizeState("restoring");
      runNeutralizeRestore(() => setNeutralizeState("normal"));
    }
  };

  const startMediaRecorder = () => {
    if (!sceneRef.current) return;
    const canvas = sceneRef.current.renderer.domElement;
    const stream = canvas.captureStream(60);
    const mimeType = MediaRecorder.isTypeSupported("video/webm; codecs=vp9")
      ? "video/webm; codecs=vp9"
      : MediaRecorder.isTypeSupported("video/webm; codecs=vp8")
        ? "video/webm; codecs=vp8"
        : "video/webm";
    recordedChunksRef.current = [];
    const recorder = new MediaRecorder(stream, {
      mimeType,
      videoBitsPerSecond: 40000000,
    });
    recorder.ondataavailable = (e) => {
      if (e.data && e.data.size > 0) recordedChunksRef.current.push(e.data);
    };
    recorder.onstop = () => {
      const blob = new Blob(recordedChunksRef.current, { type: mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "polyform-" + Date.now() + ".webm";
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      recordedChunksRef.current = [];
    };
    recorder.start();
    mediaRecorderRef.current = recorder;
  };

  const handleRecord = () => {
    if (neutralizeState !== "normal" && recordPhase === "idle") return;
    if (recordPhase === "idle") {
      // Start: neutralize -> record at neutral -> restore while recording
      setNeutralizeState("fading-out");
      setRecordPhase("neutralizing");
      runNeutralizeCycle(
        () => {
          // Reached neutral: start recording
          startMediaRecorder();
          setRecordPhase("restoring");
        },
        () => {
          // Restore complete: fully recording
          setNeutralizeState("normal");
          setRecordPhase("recording");
        }
      );
    } else if (recordPhase === "recording") {
      // Stop: neutralize -> stop recording at neutral -> restore
      setNeutralizeState("fading-out");
      setRecordPhase("fadeout");
      runNeutralizeCycle(
        () => {
          // Reached neutral: stop recording
          if (mediaRecorderRef.current && mediaRecorderRef.current.state !== "inactive") {
            mediaRecorderRef.current.stop();
          }
          mediaRecorderRef.current = null;
          setRecordPhase("finalizing");
        },
        () => {
          // Restore complete
          setNeutralizeState("normal");
          setRecordPhase("idle");
        }
      );
    }
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
    const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
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
      const z = (ozS && ozS.value) || 2;
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
    const driftStartClock = animClockRef.current;

    const tick = (now) => {
      const dt = (now - last) / 1000;
      last = now;
      const animDt = paused ? 0 : dt;
      animClockRef.current += animDt;
      const animClock = animClockRef.current;
      const baseRate = tempo;

      // Process transitions
      transitionsRef.current.tick(now);

      const sweepView = (p) => {
        if (!p.anim) return p;

        // Animate phase offset if present and animated
        let po = p.phaseOffset;
        if (po && po.anim) {
          const poMode = po.animMode || "sweep";
          const poSpeed = baseRate * po.ratio;
          if (poMode === "loop") {
            const poSpan = po.max - po.min;
            if (poSpan > 0) {
              let pv = po.value + animDt * poSpeed * poSpan;
              pv = ((pv - po.min) % poSpan + poSpan) % poSpan + po.min;
              po = { ...po, value: pv };
            }
          } else {
            const poPhase = animClock * poSpeed;
            const pot = 0.5 - 0.5 * Math.cos(poPhase * Math.PI * 2);
            po = { ...po, value: po.min + pot * (po.max - po.min) };
          }
        }
        const phaseOff = po ? po.value : 0;

        const mode = p.animMode || (p.periodic ? "loop" : "sweep");
        const speed = baseRate * p.ratio;
        if (mode === "loop") {
          const span = p.max - p.min;
          if (span <= 0) return { ...p, phaseOffset: po };
          const base = p._loopBase !== undefined ? p._loopBase : p.value;
          let v = base + animDt * speed * span;
          v = ((v - p.min) % span + span) % span + p.min;
          const vOff = ((v - p.min + phaseOff * span) % span + span) % span + p.min;
          return { ...p, value: vOff, _loopBase: v, phaseOffset: po };
        }
        const phase = animClock * speed + phaseOff;
        const t = 0.5 - 0.5 * Math.cos(phase * Math.PI * 2);
        return { ...p, value: p.min + t * (p.max - p.min), phaseOffset: po };
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
            newParams[k] = sweepView(param);
            changed = true;
          }
        }
        let newBlend = layer.blend;
        if (layer.blend && layer.blend.anim) {
          newBlend = sweepView(layer.blend);
          changed = true;
        }
        return changed ? { ...layer, params: newParams, blend: newBlend } : layer;
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
            newParams[k] = sweepView(param);
            changed = true;
          }
        }
        let newBlend = layer.blend;
        if (layer.blend && layer.blend.anim) {
          newBlend = sweepView(layer.blend);
          changed = true;
        }
        return changed ? { ...layer, params: newParams, blend: newBlend } : layer;
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
          next.angle = (q.angle + animDt * speed * Math.PI * 2) % (Math.PI * 2);
        }
        if (q.mode === "drift") {
          const t = (animClock - driftStartClock) * q.driftSpeed;
          next.ax = Math.sin(t * 0.7) + 0.3 * Math.cos(t * 1.3);
          next.ay = Math.cos(t * 0.5) + 0.3 * Math.sin(t * 1.7);
          next.az = Math.sin(t * 0.9 + 1.2);
        } else {
          if (q.axAnim) next.ax = Math.cos(animClock * baseRate * q.axRatio * Math.PI * 2);
          if (q.ayAnim) next.ay = Math.cos(animClock * baseRate * q.ayRatio * Math.PI * 2);
          if (q.azAnim) next.az = Math.cos(animClock * baseRate * q.azRatio * Math.PI * 2);
        }
        return next;
      });

      if (sceneRef.current) {
        const s = stateRef.current;
        const cam = sceneRef.current.camera;
        const oc = sceneRef.current.orthoCam;
        const ax = s.orbitX.value;
        const ay = s.orbitY.value;
        const z = s.zoomP.value;
        const tx = s.targetX.value;
        const ty = s.targetY.value;
        const rollA = s.roll.value;

        const px = tx + z * Math.sin(ay) * Math.sin(ax);
        const py = z * Math.cos(ay);
        const pz = ty + z * Math.sin(ay) * Math.cos(ax);

        const flipY = Math.sin(ay) >= 0 ? 1 : -1;
        const fwd = new THREE.Vector3(tx - px, 0 - py, ty - pz).normalize();
        let baseUp = new THREE.Vector3(0, flipY, 0);
        if (Math.abs(fwd.dot(baseUp)) > 0.9999) {
          baseUp = new THREE.Vector3(0, 0, 1);
        }
        const upStable = baseUp.clone().sub(fwd.clone().multiplyScalar(baseUp.dot(fwd))).normalize();
        const rolledUp = upStable.clone().applyAxisAngle(fwd, rollA);

        const f = s.focalLength.value;
        const sensor = s.sensorSize.value;
        const fovDeg = (2 * Math.atan(sensor / (2 * f))) * (180 / Math.PI);

        cam.fov = fovDeg;
        cam.position.set(px, py, pz);
        cam.up.copy(rolledUp);
        cam.lookAt(tx, 0, ty);
        cam.updateProjectionMatrix();

        const sx = s.shiftX.value;
        const sy = s.shiftY.value;
        const ana = s.anamorphic.value;
        if (sx !== 0 || sy !== 0 || ana !== 1) {
          const M = new THREE.Matrix4().makeTranslation(sx, sy, 0);
          M.multiply(new THREE.Matrix4().makeScale(ana, 1, 1));
          cam.projectionMatrix.premultiply(M);
          cam.projectionMatrixInverse.copy(cam.projectionMatrix).invert();
        }

        const wPx = sceneRef.current.mount.clientWidth;
        const hPx = sceneRef.current.mount.clientHeight;
        const aspect = wPx / hPx;
        const oz = s.orthoZoom.value;
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

        const qa = new THREE.Vector3(s.quat.ax, s.quat.ay, s.quat.az);
        if (qa.lengthSq() < 1e-6) qa.set(0, 1, 0);
        qa.normalize();
        const qr = new THREE.Quaternion().setFromAxisAngle(qa, s.quat.angle);
        sceneRef.current.worldGroup.quaternion.copy(qr);
      }

      rebuildGeometry();
      if (sceneRef.current) {
        const activeCam = stateRef.current.projection === "orthographic"
          ? sceneRef.current.orthoCam
          : sceneRef.current.camera;
        sceneRef.current.renderer.render(sceneRef.current.scene, activeCam);
      }
      fpsRef.current.frames++;
      if (now - fpsRef.current.lastSample >= 1000) {
        fpsRef.current.value = fpsRef.current.frames;
        fpsRef.current.frames = 0;
        fpsRef.current.lastSample = now;
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [tempo, paused]);

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

    const N = Math.round(s.densityP.value);
    const SAMPLES = 80;
    const ext = s.scaleP.value;

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

    const k1 = s.distortK1.value;
    const k2 = s.distortK2.value;
    const distortActive = k1 !== 0 || k2 !== 0;

    let projView = null;
    let invProjView = null;
    if (distortActive) {
      const activeCam = s.projection === "orthographic"
        ? sceneRef.current.orthoCam
        : sceneRef.current.camera;
      activeCam.updateMatrixWorld(true);
      sceneRef.current.worldGroup.updateMatrixWorld(true);
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
    if (neutralizingRef.current) return;
    setLayers(ls => [...ls, makeLayer(type)]);
    setShowAddMenu(false);
  };
  const removeLayer = (id) => { if (neutralizingRef.current) return; setLayers(ls => ls.filter(l => l.id !== id)); };
  const duplicateLayer = (id) => { if (neutralizingRef.current) return; setLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const dup = { ...ls[idx], id: generateLayerId(),
      params: JSON.parse(JSON.stringify(ls[idx].params)),
      blend: { ...ls[idx].blend } };
    const next = [...ls];
    next.splice(idx + 1, 0, dup);
    return next;
  }); };
  const updateLayer = (id, updates) => setLayers(ls => ls.map(l => l.id === id ? { ...l, ...updates } : l));
  const updateLayerParam = (id, key, updates) => setLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, params: { ...l.params, [key]: { ...l.params[key], ...updates } } };
  }));
  const updateLayerBlend = (id, updates) => setLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, blend: { ...l.blend, ...updates } };
  }));

  // Displacement layer manipulation
  const addDispLayer = (type) => {
    if (neutralizingRef.current) return;
    setDispLayers(ls => [...ls, makeDispLayer(type)]);
    setShowDispAddMenu(false);
  };
  const removeDispLayer = (id) => { if (neutralizingRef.current) return; setDispLayers(ls => ls.filter(l => l.id !== id)); };
  const duplicateDispLayer = (id) => { if (neutralizingRef.current) return; setDispLayers(ls => {
    const idx = ls.findIndex(l => l.id === id);
    if (idx === -1) return ls;
    const dup = { ...ls[idx], id: generateLayerId(),
      params: JSON.parse(JSON.stringify(ls[idx].params)),
      blend: { ...ls[idx].blend } };
    const next = [...ls];
    next.splice(idx + 1, 0, dup);
    return next;
  }); };
  const updateDispLayer = (id, updates) => setDispLayers(ls => ls.map(l => l.id === id ? { ...l, ...updates } : l));
  const updateDispLayerParam = (id, key, updates) => setDispLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, params: { ...l.params, [key]: { ...l.params[key], ...updates } } };
  }));
  const updateDispLayerBlend = (id, updates) => setDispLayers(ls => ls.map(l => {
    if (l.id !== id) return l;
    return { ...l, blend: { ...l.blend, ...updates } };
  }));

  return (
    <div style={{
      minHeight: "100vh", background: "#050505", color: "#e8e8e8",
      fontFamily: "ui-monospace, 'SF Mono', Menlo, monospace",
      padding: 16, boxSizing: "border-box",
    }}>
      <div style={{ maxWidth: 1400, margin: "0 auto" }}>
        <div style={{
          display: "grid",
          gridTemplateColumns: "minmax(0, 1fr)",
          gap: 16,
        }}
        className="pdc-layout">
          <style>{`
            html, body { margin: 0; padding: 0; }
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
            @keyframes pdc-rec-blink {
              0%, 100% { opacity: 1; }
              50% { opacity: 0.3; }
            }
          `}</style>

          <div className="pdc-canvas-col">
            <div style={{
              position: "relative", aspectRatio: "1 / 1",
              maxHeight: "calc(100vh - 48px)", maxWidth: "calc(100vh - 48px)",
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
              {recordPhase !== "idle" && (
                <div style={{
                  position: "absolute", bottom: 12, right: 12,
                  display: "flex", alignItems: "center", gap: 6,
                  pointerEvents: "none",
                  fontFamily: "ui-monospace, monospace", fontSize: 9,
                  color: "#ff4444", letterSpacing: "0.1em",
                  textTransform: "uppercase",
                  animation: recordPhase === "recording" ? "pdc-rec-blink 1s ease-in-out infinite" : "none",
                }}>
                  <span style={{
                    width: 8, height: 8, borderRadius: "50%",
                    background: "#ff4444",
                    boxShadow: "0 0 6px #ff4444",
                  }} />
                  {recordPhase === "recording" ? "rec" : recordPhase}
                </div>
              )}
              {overlayActive && dimAccount && (
                <div style={{
                  position: "absolute", top: 12, left: 12, right: 12,
                  fontFamily: "ui-monospace, monospace",
                  fontSize: 9, lineHeight: 1.6,
                  color: P.grid, opacity: 0.7,
                  pointerEvents: "none",
                  display: "flex", justifyContent: "space-between", gap: 12,
                }}>
                  {showActiveLayers && (
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ opacity: 0.5, marginBottom: 4 }}>active layers</div>
                      {dimAccount.active.map((a, i) => (
                        <div key={i}>
                          <div style={{
                            opacity: a.animated ? 1 : 0.65,
                            whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis",
                          }}>
                            {a.label} . {a.struct}{a.temp > 0 ? `+${a.temp}t` : ""}
                            {a.ratio !== null && a.animated && (
                              <span style={{ opacity: 0.5 }}> . x{a.ratio.toFixed(3)}</span>
                            )}
                            {a.blendValue !== undefined && a.blendValue < 1.0 && (
                              <span style={{ opacity: 0.4 }}> . blend {a.blendValue.toFixed(2)}</span>
                            )}
                          </div>
                          {showParamValues && a.details && a.details.map((d, j) => (
                            <div key={j} style={{ fontSize: 8, opacity: 0.5, paddingLeft: 8 }}>
                              {d.key}={typeof d.value === "number" ? d.value.toFixed(3) : d.value} . {d.mode} . x{d.ratio.toFixed(4)}
                              {d.phaseOffset !== null && d.phaseOffset !== undefined && <span> . phase {d.phaseOffset.toFixed(3)}</span>}
                            </div>
                          ))}
                        </div>
                      ))}
                    </div>
                  )}
                  {showDimCount && (
                    <div style={{ textAlign: "right" }}>
                      <div style={{ opacity: 0.5, marginBottom: 4 }}>dim count</div>
                      <div>structural . {dimAccount.struct}</div>
                      <div>temporal . {dimAccount.temp}</div>
                      <div style={{ marginTop: 4, color: P.warp }}>total . {dimAccount.total}D</div>
                      <div style={{ marginTop: 4, opacity: 0.45 }}>operators . {dimAccount.operators}</div>
                      <div style={{ marginTop: 6, opacity: 0.6 }}>tempo . {Math.round(tempo * 60)} bpm{paused ? " . paused" : ""}</div>
                      <div style={{ opacity: 0.5 }}>lattice . {systemInfo.scale} x {systemInfo.density}</div>
                      <div style={{ opacity: 0.5 }}>{systemInfo.projection}{systemInfo.fov ? ` . ${systemInfo.fov} deg` : ""}{systemInfo.orthoZ ? ` . z${systemInfo.orthoZ}` : ""}</div>
                      <div style={{ opacity: 0.5 }}>palette . {systemInfo.palette}</div>
                      {systemInfo.quatActive && (
                        <div style={{ opacity: 0.5 }}>quat . {systemInfo.quatAngle} . axis {systemInfo.quatAxis}</div>
                      )}
                      <div style={{ opacity: 0.35, marginTop: 4 }}>{systemInfo.fps} fps</div>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
            <header>
              <div style={{
                fontFamily: "Georgia, 'Times New Roman', serif", fontStyle: "italic",
                fontSize: 28, letterSpacing: "0.02em", color: P.grid,
                textShadow: `0 0 20px ${P.glow}66`,
              }}>
                Polyform
              </div>
              <div style={{ fontSize: 11, opacity: 0.5, marginTop: 4, lineHeight: 1.5 }}>
                a lattice synthesizer &middot; fold a grid through composed mathematics with countable dimensions
              </div>
            </header>

            <div style={{
              display: "flex", flexDirection: "column", gap: 16,
              background: "#0c0c0c", border: "1px solid #1a1a1a",
              padding: 16, borderRadius: 4,
            }}>
            <div>
              <div onClick={() => setShowHelp(h => !h)} style={{
                fontSize: 9, letterSpacing: "0.2em", opacity: 0.5,
                marginBottom: showHelp ? 8 : 0, textTransform: "uppercase",
                cursor: "pointer", userSelect: "none",
                display: "flex", justifyContent: "space-between", alignItems: "center",
              }}>
                <span>guide</span>
                <span style={{ fontSize: 8, opacity: 0.6 }}>{showHelp ? "\u25B2" : "\u25BC"}</span>
              </div>
              {showHelp && (
                <div style={{ fontSize: 10, opacity: 0.45, lineHeight: 1.7 }}>
                  <strong style={{ color: P.grid, fontWeight: "normal" }}>signal flow</strong> . the equation on the canvas shows the full reconstruction formula for the current object . every active operator appears in its nesting position . the equation is the structural identity of the form
                  <br/><br/>
                  <strong style={{ color: P.grid, fontWeight: "normal" }}>pipeline</strong> . sections below follow execution order: lattice defines the 2D domain, operations compose R2 maps on it, lift adds 3D displacement, quaternion rotates the result in SO(3), camera projects it, distortion warps the projection
                  <br/><br/>
                  <strong style={{ color: P.grid, fontWeight: "normal" }}>animation</strong> . toggle the anim button on any parameter to animate it . each animated param runs at <span style={{ color: P.grid }}>tempo x ratio</span> . rational ratios produce repeating polyrhythms . irrational/metallic ratios produce non-repeating coupling
                  <br/><br/>
                  <strong style={{ color: P.grid, fontWeight: "normal" }}>blend</strong> . every layer has a blend param (0 = identity, 1 = full effect) . toggling a layer on/off animates blend with cosine easing . reorder via fade: blend out, swap, blend in
                  <br/><br/>
                  <strong style={{ color: P.grid, fontWeight: "normal" }}>dim count</strong> . structural = parameter count (continuous DOFs) . temporal = animated parameter count (moving DOFs) . total = structural + temporal . operators = active nodes in the composition chain
                </div>
              )}
            </div>
            <Section label="display">
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap" }}>
                <Toggle active={showFormula} onClick={() => setShowFormula(!showFormula)} accent={P.grid}>signal flow</Toggle>
                <Toggle active={showDimCount} onClick={() => setShowDimCount(!showDimCount)} accent={P.grid}>dim count</Toggle>
                <Toggle active={showActiveLayers} onClick={() => setShowActiveLayers(!showActiveLayers)} accent={P.grid}>active layers</Toggle>
                {showActiveLayers && (
                  <Toggle active={showParamValues} onClick={() => setShowParamValues(!showParamValues)} accent={P.grid}>param values</Toggle>
                )}
                <Toggle active={false} onClick={exportPNG} accent={P.grid}>export png</Toggle>
                <Toggle active={neutralizeState !== "normal"} onClick={handleNeutralize} accent={P.grid}>{neutralizeState === "normal" ? "neutralize" : neutralizeState === "neutral" ? "restore" : neutralizeState === "fading-out" ? "neutralizing..." : "restoring..."}</Toggle>
                <Toggle active={recordPhase !== "idle"} onClick={handleRecord} accent={recordPhase !== "idle" ? "#ff4444" : P.grid}>{recordPhase === "idle" ? "record" : recordPhase === "recording" ? "stop recording" : recordPhase}</Toggle>
              </div>
            </Section>

            <Section label="preset">
              <div style={{ display: "flex", gap: 8 }}>
                <button onClick={handleSave} style={{
                  padding: "8px 16px", background: "transparent", color: P.grid,
                  border: `1px solid ${P.grid}66`, borderRadius: 2, cursor: "pointer",
                  fontSize: 11, fontFamily: "ui-monospace, monospace",
                  letterSpacing: "0.1em", textTransform: "uppercase",
                }}>save</button>
                <button onClick={handleLoad} style={{
                  padding: "8px 16px", background: "transparent", color: P.grid,
                  border: `1px solid ${P.grid}66`, borderRadius: 2, cursor: "pointer",
                  fontSize: 11, fontFamily: "ui-monospace, monospace",
                  letterSpacing: "0.1em", textTransform: "uppercase",
                }}>load</button>
              </div>
              {importWarnings && (
                <div style={{
                  marginTop: 8, padding: 8, background: "#1a1500",
                  border: "1px solid #554400", borderRadius: 2,
                  fontSize: 10, lineHeight: 1.5, color: "#ffcc00",
                }}>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 4 }}>
                    <span style={{ fontWeight: "bold" }}>import warnings</span>
                    <button onClick={() => setImportWarnings(null)} style={{
                      background: "transparent", border: "none", color: "#ffcc00",
                      cursor: "pointer", fontSize: 11,
                    }}>x</button>
                  </div>
                  {importWarnings.map((w, i) => <div key={i}>{w}</div>)}
                </div>
              )}
            </Section>

            <Section label="tempo">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                base rate . animated parameters run at <span style={{ color: P.grid }}>tempo x ratio</span>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 10 }}>
                <Toggle active={!paused} onClick={() => setPaused(p => !p)} accent={P.grid}>
                  {paused ? "play" : "pause"}
                </Toggle>
              </div>
              <RangeSlider label="bpm" value={tempo * 60}
                onChange={(v) => setTempo(v / 60)}
                onReset={() => setTempo(0.3)}
                min={0} max={200} step={1} accent={P.grid}
                display={Math.round(tempo * 60) + " bpm"} />
              <div style={{ marginTop: 8 }}>
                <div style={{ fontSize: 9, opacity: 0.5, marginBottom: 4, letterSpacing: "0.1em", textTransform: "uppercase" }}>
                  presets
                </div>
                <div style={{ display: "flex", flexWrap: "wrap", gap: 3 }}>
                  {BPM_PRESETS.map(bpm => (
                    <Pill key={bpm} active={Math.round(tempo * 60) === bpm}
                      onClick={() => smoothSetTempo(bpm / 60)}
                      accent={P.grid}>{bpm}</Pill>
                  ))}
                </div>
              </div>
            </Section>

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

            <Section label="lattice">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                the 2D domain . scale sets extent, density sets line count . everything else operates on this grid
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="lattice scale" sublabel="grid extent in domain . 1 DOF"
                  param={scaleP} setParam={setScaleP} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("scaleP", setScaleP)} />
                <ViewLayerCard label="lattice density" sublabel="line count (odd) . 1 DOF"
                  param={densityP} setParam={setDensityP} accent={P.grid} step={2} oddOnly
                  onToggle={() => toggleViewParam("densityP", setDensityP)} />
              </div>
            </Section>

            <Section label="operations (top applied first . drag to reorder)">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                R2 maps composed on the lattice . non-commutative . ordering changes the result . stack as many as you like
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {layers.map((layer, idx) => (
                  <PipelineCard
                    key={layer.id}
                    layer={layer}
                    index={idx}
                    total={layers.length}
                    onToggle={() => toggleLayerOn(layer.id)}
                    onUpdate={(u) => updateLayer(layer.id, u)}
                    onUpdateParam={(k, u) => updateLayerParam(layer.id, k, u)}
                    onUpdateBlend={(u) => updateLayerBlend(layer.id, u)}
                    onMoveUp={() => moveLayerFade(layer.id, -1)}
                    onMoveDown={() => moveLayerFade(layer.id, 1)}
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
                }}>+ add operation</button>
              </div>
            </Section>

            {showAddMenu && <AddMenu onSelect={addLayer} onClose={() => setShowAddMenu(false)} accent={P.grid} />}

            <Section label="lift (3D . sequential)">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                3D displacement applied to the post-operation lattice . each layer adds (dx, dy, dz) . ordering matters . sign of amp inverts
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {dispLayers.map((layer, idx) => (
                  <DispPipelineCard
                    key={layer.id}
                    layer={layer}
                    index={idx}
                    total={dispLayers.length}
                    onToggle={() => toggleDispLayerOn(layer.id)}
                    onUpdate={(u) => updateDispLayer(layer.id, u)}
                    onUpdateParam={(k, u) => updateDispLayerParam(layer.id, k, u)}
                    onUpdateBlend={(u) => updateDispLayerBlend(layer.id, u)}
                    onMoveUp={() => moveDispLayerFade(layer.id, -1)}
                    onMoveDown={() => moveDispLayerFade(layer.id, 1)}
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
                }}>+ add lift</button>
              </div>
            </Section>

            {showDispAddMenu && <DispAddMenu onSelect={addDispLayer} onClose={() => setShowDispAddMenu(false)} accent={P.grid} />}

            <Section label="quaternion . SO(3)">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                rotates the entire lattice result in 3D . applied after operations and lift, before camera . axis + angle parameterization
              </div>
              <QuaternionCard quat={quat} setQuat={setQuat} accent={P.grid} onToggle={() => toggleQuat()} />
            </Section>

            <Section label="camera . position">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                where the eye is . orbit sets spherical position around target . roll spins around the view axis
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="orbit x" sublabel="azimuth . 1 DOF"
                  param={orbitX} setParam={setOrbitX} accent={P.grid} step={0.001} periodic unit="rad"
                  onToggle={() => toggleViewParam("orbitX", setOrbitX)} />
                <ViewLayerCard label="orbit y" sublabel="elevation . 1 DOF"
                  param={orbitY} setParam={setOrbitY} accent={P.grid} step={0.001} periodic unit="rad"
                  onToggle={() => toggleViewParam("orbitY", setOrbitY)} />
                <ViewLayerCard label="distance" sublabel="camera distance to target . 1 DOF"
                  param={zoomP} setParam={setZoomP} accent={P.grid} step={0.05}
                  onToggle={() => toggleViewParam("zoomP", setZoomP)} />
                <ViewLayerCard label="target x" sublabel="look-at offset along world x . 1 DOF"
                  param={targetX} setParam={setTargetX} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("targetX", setTargetX)} />
                <ViewLayerCard label="target y" sublabel="look-at offset along world z . 1 DOF"
                  param={targetY} setParam={setTargetY} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("targetY", setTargetY)} />
                <ViewLayerCard label="roll" sublabel="rotation around view axis . 1 DOF"
                  param={roll} setParam={setRoll} accent={P.grid} step={0.001} periodic unit="rad"
                  onToggle={() => toggleViewParam("roll", setRoll)} />
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
                      param={focalLength} setParam={setFocalLength} accent={P.grid} step={0.5}
                      onToggle={() => toggleViewParam("focalLength", setFocalLength)} />
                    <ViewLayerCard label="sensor size" sublabel="mm . wider sensor = wider FOV at same focal length . 1 DOF"
                      param={sensorSize} setParam={setSensorSize} accent={P.grid} step={0.5}
                      onToggle={() => toggleViewParam("sensorSize", setSensorSize)} />
                  </>
                )}
                {projection === "orthographic" && (
                  <ViewLayerCard label="ortho zoom" sublabel="visible half-extent . 1 DOF"
                    param={orthoZoom} setParam={setOrthoZoom} accent={P.grid} step={0.05}
                    onToggle={() => toggleViewParam("orthoZoom", setOrthoZoom)} />
                )}
                <ViewLayerCard label="anamorphic" sublabel="horizontal squeeze . <1 narrows, >1 stretches . 1 DOF"
                  param={anamorphic} setParam={setAnamorphic} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("anamorphic", setAnamorphic)} />
                <ViewLayerCard label="lens shift x" sublabel="off-axis horizontal . tilt-shift . 1 DOF"
                  param={shiftX} setParam={setShiftX} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("shiftX", setShiftX)} />
                <ViewLayerCard label="lens shift y" sublabel="off-axis vertical . tilt-shift . 1 DOF"
                  param={shiftY} setParam={setShiftY} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("shiftY", setShiftY)} />
              </div>
            </Section>

            <Section label="camera . distortion">
              <div style={{ fontSize: 10, opacity: 0.55, marginBottom: 10, lineHeight: 1.6 }}>
                radial lens distortion . r' = r (1 + k1 r^2 + k2 r^4) . negative k = barrel (fisheye) . positive k = pincushion
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                <ViewLayerCard label="k1" sublabel="quadratic distortion . dominant term . 1 DOF"
                  param={distortK1} setParam={setDistortK1} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("distortK1", setDistortK1)} />
                <ViewLayerCard label="k2" sublabel="quartic distortion . higher-order shaping . 1 DOF"
                  param={distortK2} setParam={setDistortK2} accent={P.grid} step={0.01}
                  onToggle={() => toggleViewParam("distortK2", setDistortK2)} />
              </div>
            </Section>
          </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function DispPipelineCard({ layer, index, total, onToggle, onUpdate, onUpdateParam, onUpdateBlend,
                            onMoveUp, onMoveDown, onRemove, onDuplicate, accent }) {
  const def = DISPLACEMENTS[layer.type];
  const [expanded, setExpanded] = useState(false);
  if (!def) return null;
  const hasParams = def.params.length > 0;
  const anyAnim = (layer.blend && layer.blend.anim) || Object.values(layer.params).some(p => p.anim);

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
        <Switch on={layer.on} onClick={onToggle} accent={accent} />
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
        <button onClick={() => setExpanded(!expanded)} style={iconBtn()}>{expanded ? "-" : "+"}</button>
      </div>
      {layer.on && expanded && (
        <div style={{ marginTop: 10, display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <AnimToggle anim={layer.blend.anim} onClick={() => onUpdateBlend({ anim: !layer.blend.anim })} accent={accent} small />
            <div style={{ flex: 1 }}>
              <RangeSlider label="blend" value={layer.blend.value}
                min={0} max={1} step={0.01}
                onChange={(v) => onUpdateBlend({ value: v, anim: false })}
                onReset={() => onUpdateBlend({ value: 1, anim: false })}
                accent={accent} />
            </div>
          </div>
          {layer.blend.anim && (
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
              <AnimModeToggle mode={layer.blend.animMode || "sweep"}
                onClick={() => onUpdateBlend({ animMode: (layer.blend.animMode || "sweep") === "loop" ? "sweep" : "loop" })}
                accent={accent} />
              <div style={{ flex: 1 }}>
                <RatioRow ratio={layer.blend.ratio || 1} onChange={(r) => onUpdateBlend({ ratio: r })} accent={accent} compact />
              </div>
            </div>
          )}
          <BlendPhaseOffset blend={layer.blend} onUpdateBlend={onUpdateBlend} accent={accent} />
          {hasParams && def.params.map(pdef => (
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
          add lift
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

function BlendPhaseOffset({ blend, onUpdateBlend, accent }) {
  const bpo = blend.phaseOffset || { value: 0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
  if (!blend.anim) return null;
  return (
    <>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
        <AnimToggle anim={bpo.anim} onClick={() => onUpdateBlend({ phaseOffset: { ...bpo, anim: !bpo.anim } })} accent={accent} small />
        <div style={{ flex: 1 }}>
          <RangeSlider label="phase" value={bpo.value} min={0} max={1} step={0.01}
            onChange={(v) => onUpdateBlend({ phaseOffset: { ...bpo, value: v, anim: false } })}
            onReset={() => onUpdateBlend({ phaseOffset: { ...bpo, value: 0, anim: false } })}
            accent={accent} display={bpo.value.toFixed(2)} />
        </div>
      </div>
      {bpo.anim && (
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
          <AnimModeToggle mode={bpo.animMode || "sweep"}
            onClick={() => onUpdateBlend({ phaseOffset: { ...bpo, animMode: (bpo.animMode || "sweep") === "loop" ? "sweep" : "loop" } })}
            accent={accent} />
          <div style={{ flex: 1 }}>
            <RatioRow ratio={bpo.ratio || 1} onChange={(r) => onUpdateBlend({ phaseOffset: { ...bpo, ratio: r } })} accent={accent} compact />
          </div>
        </div>
      )}
    </>
  );
}

function PipelineCard({ layer, index, total, onToggle, onUpdate, onUpdateParam, onUpdateBlend,
                        onMoveUp, onMoveDown, onRemove, onDuplicate, accent }) {
  const def = TRANSFORMS[layer.type];
  const [expanded, setExpanded] = useState(false);
  if (!def) return null;
  const hasParams = def.params.length > 0;
  const anyAnim = (layer.blend && layer.blend.anim) || Object.values(layer.params).some(p => p.anim);

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
        <Switch on={layer.on} onClick={onToggle} accent={accent} />
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
        <button onClick={() => setExpanded(!expanded)} style={iconBtn()}>{expanded ? "-" : "+"}</button>
      </div>
      {layer.on && expanded && (
        <div style={{ marginTop: 10, display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <AnimToggle anim={layer.blend.anim} onClick={() => onUpdateBlend({ anim: !layer.blend.anim })} accent={accent} small />
            <div style={{ flex: 1 }}>
              <RangeSlider label="blend" value={layer.blend.value}
                min={0} max={1} step={0.01}
                onChange={(v) => onUpdateBlend({ value: v, anim: false })}
                onReset={() => onUpdateBlend({ value: 1, anim: false })}
                accent={accent} />
            </div>
          </div>
          {layer.blend.anim && (
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
              <AnimModeToggle mode={layer.blend.animMode || "sweep"}
                onClick={() => onUpdateBlend({ animMode: (layer.blend.animMode || "sweep") === "loop" ? "sweep" : "loop" })}
                accent={accent} />
              <div style={{ flex: 1 }}>
                <RatioRow ratio={layer.blend.ratio || 1} onChange={(r) => onUpdateBlend({ ratio: r })} accent={accent} compact />
              </div>
            </div>
          )}
          <BlendPhaseOffset blend={layer.blend} onUpdateBlend={onUpdateBlend} accent={accent} />
          {hasParams && def.params.map(pdef => (
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
  const po = param.phaseOffset || { value: 0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
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
        <>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
            <AnimModeToggle mode={animMode}
              onClick={() => onUpdate({ animMode: animMode === "loop" ? "sweep" : "loop" })}
              accent={accent} />
            <div style={{ flex: 1 }}>
              <RatioRow ratio={param.ratio} onChange={(r) => onUpdate({ ratio: r })} accent={accent} compact />
            </div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
            <AnimToggle anim={po.anim} onClick={() => onUpdate({ phaseOffset: { ...po, anim: !po.anim } })} accent={accent} small />
            <div style={{ flex: 1 }}>
              <RangeSlider label="phase" value={po.value} min={0} max={1} step={0.01}
                onChange={(v) => onUpdate({ phaseOffset: { ...po, value: v, anim: false } })}
                onReset={() => onUpdate({ phaseOffset: { ...po, value: 0, anim: false } })}
                accent={accent} display={po.value.toFixed(2)} />
            </div>
          </div>
          {po.anim && (
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
              <AnimModeToggle mode={po.animMode || "sweep"}
                onClick={() => onUpdate({ phaseOffset: { ...po, animMode: (po.animMode || "sweep") === "loop" ? "sweep" : "loop" } })}
                accent={accent} />
              <div style={{ flex: 1 }}>
                <RatioRow ratio={po.ratio || 1} onChange={(r) => onUpdate({ phaseOffset: { ...po, ratio: r } })} accent={accent} compact />
              </div>
            </div>
          )}
        </>
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
          add operation
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

function ViewLayerCard({ label, sublabel, param, setParam, accent, step, periodic, unit, integer, oddOnly, extra, onToggle }) {
  const quantize = (v) => {
    if (oddOnly) {
      const r = Math.round(v);
      return r % 2 === 0 ? r + 1 : r;
    }
    if (integer) return Math.round(v);
    return v;
  };
  const animMode = param.animMode || (param.periodic ? "loop" : "sweep");
  const po = param.phaseOffset || { value: 0, min: 0, max: 1, anim: false, animMode: "sweep", ratio: 1 };
  return (
    <div style={{
      border: `1px solid ${param.on ? accent + "55" : "#222"}`,
      borderRadius: 4, padding: 12,
      background: param.on ? accent + "08" : "transparent",
    }}>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: param.on ? 10 : 0 }}>
        <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
          <Switch on={param.on} onClick={onToggle || (() => setParam(p => ({ ...p, on: !p.on })))} accent={accent} />
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
              <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
                <AnimToggle anim={po.anim} onClick={() => setParam(p => ({ ...p, phaseOffset: { ...po, anim: !po.anim } }))} accent={accent} small />
                <div style={{ flex: 1 }}>
                  <RangeSlider label="phase" value={po.value} min={0} max={1} step={0.01}
                    onChange={(v) => setParam(p => ({ ...p, phaseOffset: { ...po, value: v, anim: false } }))}
                    onReset={() => setParam(p => ({ ...p, phaseOffset: { ...po, value: 0, anim: false } }))}
                    accent={accent} display={po.value.toFixed(2)} />
                </div>
              </div>
              {po.anim && (
                <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
                  <AnimModeToggle mode={po.animMode || "sweep"}
                    onClick={() => setParam(p => ({ ...p, phaseOffset: { ...po, animMode: (po.animMode || "sweep") === "loop" ? "sweep" : "loop" } }))}
                    accent={accent} />
                  <div style={{ flex: 1 }}>
                    <RatioRow ratio={po.ratio || 1} onChange={(r) => setParam(p => ({ ...p, phaseOffset: { ...po, ratio: r } }))} accent={accent} compact />
                  </div>
                </div>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}

function QuaternionCard({ quat, setQuat, accent, onToggle }) {
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
          <Switch on={quat.on} onClick={onToggle || (() => setQuat(q => ({ ...q, on: !q.on })))} accent={accent} />
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

function RatioGroup({ group, ratio, onChange, accent }) {
  const [open, setOpen] = React.useState(false);
  const active = group.presets.some(p => Math.abs(ratio - p.value) < 1e-4);
  return (
    <div style={{ marginBottom: 4 }}>
      <div onClick={() => setOpen(o => !o)} style={{
        fontSize: 8, opacity: active ? 0.8 : 0.4, marginBottom: open ? 3 : 0,
        letterSpacing: "0.1em", cursor: "pointer", userSelect: "none",
        display: "flex", justifyContent: "space-between", alignItems: "center",
        color: active ? accent : undefined,
      }}>
        <span>{group.label}</span>
        <span style={{ fontSize: 7, opacity: 0.5 }}>{open ? "\u25B2" : "\u25BC"}</span>
      </div>
      {open && (
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
      )}
    </div>
  );
}

function RatioRow({ ratio, onChange, accent, compact }) {
  const [open, setOpen] = React.useState(false);
  const [customInput, setCustomInput] = React.useState("");
  const handleCustom = (e) => {
    if (e.key === "Enter") {
      var v = parseFloat(customInput);
      if (!isNaN(v) && v > 0) { onChange(v); setCustomInput(""); }
    }
  };
  return (
    <div style={{ marginTop: compact ? 4 : 0 }}>
      <div onClick={() => setOpen(o => !o)} style={{
        display: "flex", justifyContent: "space-between", alignItems: "center",
        fontSize: 9, letterSpacing: "0.15em", opacity: 0.6,
        marginBottom: open ? 6 : 0, textTransform: "uppercase",
        cursor: "pointer", userSelect: "none",
      }}>
        <span>ratio to tempo</span>
        <span style={{ display: "flex", alignItems: "center", gap: 6 }}>
          <span style={{ color: accent, opacity: 0.9 }}>x{ratio.toFixed(4)}</span>
          <span style={{ fontSize: 8, opacity: 0.5 }}>{open ? "\u25B2" : "\u25BC"}</span>
        </span>
      </div>
      {open && (
        <>
          <div style={{ marginBottom: 6 }}>
            <input
              type="text" value={customInput}
              onClick={(e) => e.stopPropagation()}
              onChange={(e) => setCustomInput(e.target.value)}
              onKeyDown={handleCustom}
              placeholder="custom ratio (enter to set)"
              style={{
                width: "100%", boxSizing: "border-box",
                padding: "4px 6px", background: "#111", color: "#999",
                border: "1px solid #333", borderRadius: 2, fontSize: 9,
                fontFamily: "ui-monospace, monospace", outline: "none",
              }}
            />
          </div>
          {RATIO_GROUPS.map((group) => (
            <RatioGroup key={group.label} group={group} ratio={ratio} onChange={onChange} accent={accent} />
          ))}
        </>
      )}
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
        <EditableNumber value={value} accent={accent}
          displayText={display}
          onCommit={(v) => {
            var clamped = Math.max(min, Math.min(max, v));
            onChange(clamped);
          }} />
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

function EditableNumber({ value, onCommit, accent, displayText }) {
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
        fontSize: displayText !== undefined ? 10 : 9,
        opacity: displayText !== undefined ? 0.9 : 0.6,
        color: displayText !== undefined ? accent : "#aaa",
        cursor: "text", userSelect: "none",
        padding: "2px 4px",
        border: displayText !== undefined ? "none" : "1px dashed #333",
        background: "transparent",
        borderRadius: 2, minWidth: displayText !== undefined ? undefined : 36,
        textAlign: "center",
        fontFamily: "ui-monospace, monospace",
      }}
      title="click to type a value"
    >
      {displayText !== undefined ? displayText : formatNumber(value)}
    </button>
  );
}

function formatNumber(v) {
  if (v === Math.round(v)) return v.toString();
  return parseFloat(v.toFixed(4)).toString();
}

function Section({ label, children, defaultOpen }) {
  const [open, setOpen] = React.useState(defaultOpen !== false);
  return (
    <div>
      <div onClick={() => setOpen(o => !o)} style={{
        fontSize: 9, letterSpacing: "0.2em", opacity: 0.5,
        marginBottom: open ? 8 : 0, textTransform: "uppercase",
        cursor: "pointer", userSelect: "none",
        display: "flex", justifyContent: "space-between", alignItems: "center",
      }}>
        <span>{label}</span>
        <span style={{ fontSize: 8, opacity: 0.6 }}>{open ? "\u25B2" : "\u25BC"}</span>
      </div>
      {open && children}
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
