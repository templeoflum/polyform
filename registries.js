// =============================================================================
// Polyform registries + pipeline math
//
// Pure JS, no React/Three.js/browser dependencies.
// Extracted from polydimensional-coupling-v7.jsx for reuse and testing.
// =============================================================================

// ============================================================
// TRANSFORM REGISTRY
// ============================================================

export const TRANSFORMS = {
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

export const CATEGORIES = ["linear", "complex", "polar", "wave", "non-holo", "utility"];

// ============================================================
// DISPLACEMENT REGISTRY (Pipeline 2: 3D)
// fn signature: (x, y, p, x0, y0, jSigned) => [dx, dy, dz]
// (x, y) are post-XY-pipeline coordinates, (x0, y0) original
// ============================================================

export const DISPLACEMENTS = {
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

export const DISP_CATEGORIES = ["height", "wave", "3d"];

export const PHI = (1 + Math.sqrt(5)) / 2;
export const SILVER = 1 + Math.sqrt(2);
export const BRONZE = (3 + Math.sqrt(13)) / 2;
export const PLASTIC = 1.32471795724474602596;

export const PALETTES = {
  acid:  { bg: "#000", grid: "#7fff5a", warp: "#caff3a", glow: "#7fff5a" },
  cyan:  { bg: "#000", grid: "#5be3d4", warp: "#bff7ee", glow: "#5be3d4" },
  blush: { bg: "#000", grid: "#ff5e8a", warp: "#ffb3c8", glow: "#ff5e8a" },
  amber: { bg: "#0a0500", grid: "#ffb347", warp: "#fff1c2", glow: "#ffb347" },
  void:  { bg: "#0a0a12", grid: "#b39dff", warp: "#e8defc", glow: "#b39dff" },
};

export const RATIO_GROUPS = [
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

export const BPM_PRESETS = [1, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 200];

// ============================================================
// STABLE LAYER IDS
// ============================================================

export function generateLayerId() {
  return Math.random().toString(36).slice(2, 10).padEnd(8, "0");
}

// ============================================================
// REGISTRY (for preset reconciliation)
// ============================================================

export const REGISTRY = {
  operations: Object.fromEntries(
    Object.entries(TRANSFORMS).map(([key, def]) => [key, { key, params: def.params }])
  ),
  lift: Object.fromEntries(
    Object.entries(DISPLACEMENTS).map(([key, def]) => [key, { key, params: def.params }])
  ),
};

// ============================================================
// LAYER FACTORY + PIPELINE
// ============================================================

export function makeLayer(type) {
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

export function makeDispLayer(type) {
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

export function applyPipeline(x0, y0, layers) {
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

export function applyDispPipeline(x, y, x0, y0, dispLayers, jSignedFn) {
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

// Pre-flattened pipeline variants for hot path (avoid per-point allocation)
export function preFlattenLayers(layers) {
  const result = [];
  for (let i = 0; i < layers.length; i++) {
    const layer = layers[i];
    if (!layer.on) continue;
    const def = TRANSFORMS[layer.type];
    if (!def) continue;
    const flat = {};
    for (const k in layer.params) flat[k] = layer.params[k].value;
    result.push({ def, flat, blend: layer.blend ? layer.blend.value : 1 });
  }
  return result;
}

export function preFlattenDispLayers(dispLayers) {
  const result = [];
  for (let i = 0; i < dispLayers.length; i++) {
    const layer = dispLayers[i];
    if (!layer.on) continue;
    const def = DISPLACEMENTS[layer.type];
    if (!def) continue;
    const flat = {};
    for (const k in layer.params) flat[k] = layer.params[k].value;
    result.push({ def, flat, blend: layer.blend ? layer.blend.value : 1, needsJacobian: !!def.needsJacobian });
  }
  return result;
}

export function applyPipelineFlat(x0, y0, flatLayers) {
  let x = x0, y = y0;
  for (let i = 0; i < flatLayers.length; i++) {
    const fl = flatLayers[i];
    const out = fl.def.needsOrigin ? fl.def.fn(x, y, fl.flat, x0, y0) : fl.def.fn(x, y, fl.flat);
    const b = fl.blend;
    x = x + (out[0] - x) * b;
    y = y + (out[1] - y) * b;
  }
  return [x, y];
}

export function applyDispPipelineFlat(x, y, x0, y0, flatDispLayers, jSignedFn) {
  let cx = x, cy = y, cz = 0;
  let jSigned = null;
  for (let i = 0; i < flatDispLayers.length; i++) {
    const fl = flatDispLayers[i];
    if (fl.needsJacobian && jSigned === null) jSigned = jSignedFn();
    const out = fl.def.fn(cx, cy, fl.flat, x0, y0, jSigned ?? 0);
    const b = fl.blend;
    cx += out[0] * b;
    cy += out[1] * b;
    cz += out[2] * b;
  }
  return [cx - x, cy - y, cz];
}

// Map operator labels to mathematical notation
export const FORMULA_SYM = {
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

export function compositionString(layers, dispLayers, pipeline) {
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

export function countDOFs(layers, dispLayers, view) {
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
