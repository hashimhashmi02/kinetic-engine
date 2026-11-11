// Small pure helpers shared by the world. Nothing here keeps state.

export const clamp = (v, lo = 0, hi = 1) => Math.min(hi, Math.max(lo, v));
export const lerp = (a, b, u) => a + (b - a) * u;
export const smooth = (u) => u * u * (3 - 2 * u);
export const inOutCubic = (u) => (u < 0.5 ? 4 * u * u * u : 1 - Math.pow(2 - 2 * u, 3) / 2);
export const outCubic = (u) => 1 - Math.pow(1 - u, 3);
export const linear = (u) => u;

/** Progress of `t` through the window [a, b], clamped to [0, 1]. */
export const span = (t, a, b) => clamp((t - a) / (b - a));

/** Seeded generator, so the same street is built on every load. */
export function rng(seed) {
  let s = seed >>> 0;
  const next = () => {
    s = (s + 0x6d2b79f5) >>> 0;
    let r = Math.imul(s ^ (s >>> 15), 1 | s);
    r = (r + Math.imul(r ^ (r >>> 7), 61 | r)) ^ r;
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
  next.range = (a, b) => a + (b - a) * next();
  next.int = (a, b) => Math.floor(next.range(a, b + 1));
  next.pick = (list) => list[Math.floor(next() * list.length)];
  return next;
}

const hex = (c) => [1, 3, 5].map((i) => parseInt(c.slice(i, i + 2), 16));

/** Mixes two `#rrggbb` colours. */
export function mix(a, b, u) {
  const [x, y] = [hex(a), hex(b)];
  return "#" + x.map((v, i) => Math.round(lerp(v, y[i], u)).toString(16).padStart(2, "0")).join("");
}

/** Value of a keyframed track `[[t, value], ...]` at `t`, using `blend(a, b, u)`. */
export function track(keys, t, blend = lerp) {
  if (t <= keys[0][0]) return keys[0][1];
  for (let i = 1; i < keys.length; i++) {
    if (t <= keys[i][0]) {
      const [t0, a] = keys[i - 1];
      const [t1, b, ease = smooth] = keys[i];
      return blend(a, b, ease((t - t0) / (t1 - t0)));
    }
  }
  return keys[keys.length - 1][1];
}

/**
 * Closed-form spring, the same solution the engine uses (docs/ARCHITECTURE.md).
 * `step` is the response from 0 to 1; `impulse` is the response to a single push,
 * normalised so its first peak is about 1 for a lightly damped spring.
 */
export function spring(stiffness, damping) {
  const w0 = Math.sqrt(stiffness);
  const z = damping / (2 * w0);
  if (Math.abs(z - 1) < 1e-4) {
    return {
      zeta: z,
      step: (t) => 1 - Math.exp(-w0 * t) * (1 + w0 * t),
      impulse: (t) => Math.E * w0 * t * Math.exp(-w0 * t),
    };
  }
  if (z < 1) {
    const wd = w0 * Math.sqrt(1 - z * z);
    return {
      zeta: z,
      step: (t) => 1 - Math.exp(-z * w0 * t) * (Math.cos(wd * t) + ((z * w0) / wd) * Math.sin(wd * t)),
      impulse: (t) => Math.exp(-z * w0 * t) * Math.sin(wd * t),
    };
  }
  const s = Math.sqrt(z * z - 1);
  const r1 = -w0 * (z - s), r2 = -w0 * (z + s);
  return {
    zeta: z,
    step: (t) => 1 + (r2 * Math.exp(r1 * t) - r1 * Math.exp(r2 * t)) / (r1 - r2),
    impulse: (t) => ((Math.exp(r1 * t) - Math.exp(r2 * t)) / (r1 - r2)) * w0 * 2,
  };
}

/** Resamples a closed polyline to `count` points evenly spaced by arc length. */
export function resample(points, count) {
  const loop = points.concat([points[0]]);
  const lengths = [0];
  for (let i = 1; i < loop.length; i++) {
    lengths.push(lengths[i - 1] + Math.hypot(loop[i][0] - loop[i - 1][0], loop[i][1] - loop[i - 1][1]));
  }
  const total = lengths[lengths.length - 1];
  const out = [];
  let seg = 1;
  for (let k = 0; k < count; k++) {
    const d = (k / count) * total;
    while (lengths[seg] < d) seg++;
    const u = (d - lengths[seg - 1]) / (lengths[seg] - lengths[seg - 1] || 1);
    out.push([lerp(loop[seg - 1][0], loop[seg][0], u), lerp(loop[seg - 1][1], loop[seg][1], u)]);
  }
  return out;
}
