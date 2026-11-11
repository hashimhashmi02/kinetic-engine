// Morphs one shaped word into another, outline to outline.
//
// The outlines come from HarfBuzz (scripts/glyphs.mjs), so conjuncts and joining forms are
// the real shaped glyphs. The method is the one planned for the engine: resample every
// contour to the same number of points, give them the same winding, pair contours left to
// right, rotate each pair's start point to minimise travel, then interpolate. Contours
// with no partner grow from, or shrink to, a point.

import words from "./words.json";
import { clamp, inOutCubic, lerp, resample } from "./math.js";

const POINTS = 44;

const area = (c) => c.reduce((sum, p, i) => sum + p[0] * c[(i + 1) % c.length][1] - c[(i + 1) % c.length][0] * p[1], 0) / 2;
const centroid = (c) => [c.reduce((s, p) => s + p[0], 0) / c.length, c.reduce((s, p) => s + p[1], 0) / c.length];

/** Splits a word into outer contours and holes, each resampled, clockwise, sorted by x. */
function prepare(word) {
  const sized = word.contours.map((c) => ({ c, a: area(c) }));
  const outerSign = Math.sign(sized.reduce((big, s) => (Math.abs(s.a) > Math.abs(big.a) ? s : big)).a);
  const build = (list) =>
    list
      .map(({ c, a }) => {
        const pts = resample(c, POINTS);
        if (a < 0) pts.reverse();
        return { pts, at: centroid(pts) };
      })
      .sort((p, q) => p.at[0] - q.at[0]);
  return {
    outers: build(sized.filter((s) => Math.sign(s.a) === outerSign)),
    holes: build(sized.filter((s) => Math.sign(s.a) !== outerSign)),
  };
}

function align(a, b) {
  let best = 0, bestCost = Infinity;
  for (let shift = 0; shift < POINTS; shift++) {
    let cost = 0;
    for (let i = 0; i < POINTS; i++) {
      const q = b[(i + shift) % POINTS];
      cost += (a[i][0] - q[0]) ** 2 + (a[i][1] - q[1]) ** 2;
    }
    if (cost < bestCost) [best, bestCost] = [shift, cost];
  }
  return a.map((_, i) => b[(i + best) % POINTS]);
}

const dot = (at) => Array.from({ length: POINTS }, () => at);

function pair(from, to) {
  const swap = from.length > to.length;
  const [small, large] = swap ? [to, from] : [from, to];
  const partner = new Map();
  small.forEach((_, i) => partner.set(small.length === 1 ? Math.floor(large.length / 2) : Math.round((i * (large.length - 1)) / (small.length - 1)), i));
  return large.map((big, k) => {
    let other;
    if (partner.has(k)) {
      other = small[partner.get(k)].pts;
    } else if (small.length) {
      // No partner: collapse onto the nearest contour of the other word.
      const near = small.reduce((best, s) => (Math.abs(s.at[0] - big.at[0]) < Math.abs(best.at[0] - big.at[0]) ? s : best));
      other = dot(near.at);
    } else {
      other = dot(big.at);
    }
    return swap ? { a: big.pts, b: align(big.pts, other) } : { a: align(big.pts, other), b: big.pts };
  });
}

const prepared = words.map(prepare);
const steps = prepared.slice(1).map((next, i) => ({ outers: pair(prepared[i].outers, next.outers), holes: pair(prepared[i].holes, next.holes) }));

function draw(pairs, u) {
  let d = "";
  pairs.forEach(({ a, b }, k) => {
    // Contours leave one after another, left to right.
    const local = inOutCubic(clamp(u * 1.5 - (0.5 * k) / Math.max(1, pairs.length - 1)));
    for (let i = 0; i < POINTS; i++) d += `${i ? "L" : "M"}${lerp(a[i][0], b[i][0], local).toFixed(1)},${lerp(a[i][1], b[i][1], local).toFixed(1)}`;
    d += "Z";
  });
  return d;
}

/** Outlines at morph position `m`: 0 is the first word, 1 the second, 2 the third. */
export function wordAt(m) {
  const i = Math.min(steps.length - 1, Math.floor(m));
  const u = clamp(m - i);
  return { ink: draw(steps[i].outers, u), holes: draw(steps[i].holes, u) };
}

export const wordCount = words.length;
