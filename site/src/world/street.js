// Builds the paper street as SVG markup. Pure: the same seed gives the same street.
// World units: the stage is 1600 x 900, the ground line is at y = GROUND, and the near
// layer (parallax 1) is WORLD units long. Other layers are shorter or longer by their
// parallax factor.

import { mix, rng } from "./math.js";

export const GROUND = 760;
export const VIEW = 1600;
export const WORLD = 11200;
export const CAM_MAX = WORLD - VIEW;
export const PARALLAX = { far: 0.12, mid: 0.45, near: 1, fore: 1.3 };

const INK = "#1b1620";
const BONE = "#e2d6bf";
const FACADES = ["#b98a6a", "#8d6e63", "#a3927a", "#6f7f8c", "#9a6b5c", "#7c8a7a", "#c2a47c", "#5f6f86", "#a87b52", "#8a7a94"];
const SHUTTERS = ["#5b6470", "#6e5a4e", "#4f6a66", "#7a6a52", "#5a5f7a"];
const SIGNS = ["#e2d6bf", "#d9c48f", "#9fb3a8", "#c98f6b", "#b7c0d0"];
const GOODS = ["#c9973f", "#a8504a", "#5f8f8a", "#d9c48f", "#7a5f8f", "#e2d6bf"];
const BRASS = "#c9973f";

// Shop words. Each is a common noun: tea, kites, sweets, cloth, spices, books, brass, paper.
const WORDS = [
  ["चाय", "deva"], ["CHAI", "latin"], ["پتنگ", "urdu"], ["पतंग", "deva"], ["मिठाई", "deva"], ["CLOTH", "latin"],
  ["मसाले", "deva"], ["کتابیں", "urdu"], ["किताबें", "deva"], ["SPICES", "latin"], ["कपड़ा", "deva"], ["چائے", "urdu"],
];

const R = (x, y, w, h) => `M${x},${y}h${w}v${h}h${-w}Z`;
const arch = (x, y, w, h) => `M${x},${y + h}V${y + w / 2}A${w / 2},${w / 2} 0 0 1 ${x + w},${y + w / 2}V${y + h}Z`;
const shade = (d, dx = 7, dy = 9, o = 0.3) => `<path d="${d}" fill="#0b0810" opacity="${o}" transform="translate(${dx} ${dy})"/>`;
/** A sheet of paper: a soft-edged offset shadow, then the sheet. */
const P = (d, fill, dx, dy) => shade(d, dx, dy) + `<path d="${d}" fill="${fill}"/>`;
const F = (d, fill) => `<path d="${d}" fill="${fill}"/>`;
const cull = (a, b, inner) => `<g class="cull" data-a="${Math.round(a)}" data-b="${Math.round(b)}">${inner}</g>`;
const label = ([word, script], x, y, size, fill = INK) =>
  `<text class="t-${script}" x="${x}" y="${y}" font-size="${script === "urdu" ? size * 0.82 : size}" text-anchor="middle" dominant-baseline="central" fill="${fill}">${word}</text>`;

function bay(r, x, w, h, kind, out) {
  const top = GROUND - h;
  const col = r.pick(FACADES);
  const dark = mix(col, "#1a1420", 0.4);
  const lite = mix(col, "#f3e6cf", 0.3);
  const shopH = 204;
  let s = P(R(x, top, w, h), col);
  s += F(R(x - 6, top - 14, w + 12, 18), dark) + F(R(x, top + 4, w, 5), lite);

  // Upper floors: arched windows, some with a balcony below.
  const count = w > 285 ? 3 : 2;
  const ww = 46;
  const gap = (w - count * ww) / (count + 1);
  const rows = h - shopH > 300 ? 2 : 1;
  for (let row = 0; row < rows; row++) {
    const wy = top + 44 + row * 150;
    for (let c = 0; c < count; c++) {
      const wx = x + gap + c * (ww + gap);
      s += F(arch(wx - 7, wy - 7, ww + 14, 104), lite) + F(arch(wx, wy, ww, 97), "#241c28");
      if (r() < 0.45) s += F(R(wx, wy + 40, ww / 2 - 2, 57), dark);
      if (r() < 0.5) out.glow.push(`<path d="${arch(wx, wy, ww, 97)}" fill="#f6c56b" opacity="0.75"/>`);
    }
    if (r() < 0.6) {
      const by = wy + 100;
      s += P(R(x + 8, by, w - 16, 9), dark, 4, 6);
      for (let k = x + 16; k < x + w - 12; k += 15) s += F(R(k, by - 22, 3, 22), dark);
      s += F(R(x + 8, by - 26, w - 16, 5), dark);
    }
  }

  // Shop: dark opening, goods on shelves, rolling shutter, signboard.
  const ox = x + 18, ow = w - 36, oy = GROUND - shopH + 6, oh = shopH - 6;
  s += F(R(ox, oy, ow, oh), "#211a22");
  if (kind === "brass") {
    for (let k = 0; k < 9; k++) {
      const cx = ox + 30 + ((k * 43) % (ow - 60)), cy = oy + 44 + (k % 3) * 52, rr = 13 + (k % 4) * 3;
      s += `<circle cx="${cx}" cy="${cy}" r="${rr}" fill="${mix(BRASS, "#3a2a14", (k % 3) * 0.18)}"/>`;
    }
  } else if (kind === "paper") {
    for (let k = 0; k < 6; k++) {
      const px = ox + 14 + k * ((ow - 28) / 6), ph = 50 + ((k * 37) % 90);
      for (let j = 0; j < ph; j += 9) s += F(R(px, oy + oh - j - 8, (ow - 28) / 6 - 8, 7), j % 18 ? "#d8cbb0" : "#efe7d6");
    }
  } else {
    for (let j = 0; j < 3; j++) {
      const sy = oy + 50 + j * 52;
      s += F(R(ox + 8, sy, ow - 16, 5), "#3a2f3a");
      for (let k = ox + 14; k < ox + ow - 30; k += r.int(22, 34)) {
        const gh = r.int(16, 34);
        s += F(R(k, sy - gh, r.int(12, 20), gh), r.pick(GOODS));
      }
    }
  }
  const shut = r.pick(SHUTTERS);
  s +=
    `<g class="shutter" data-x="${Math.round(x + w / 2)}" data-ox="${ox}" data-oy="${oy}" transform="translate(${ox} ${oy})">` +
    `<rect width="${ow}" height="${oh}" fill="${shut}"/><rect width="${ow}" height="${oh}" fill="url(#slats)"/>` +
    `<rect y="${oh - 12}" width="${ow}" height="12" fill="${mix(shut, "#101018", 0.45)}"/></g>`;

  const sy = GROUND - shopH - 46;
  if (kind !== "paper") {
    const signCol = r.pick(SIGNS);
    s += P(R(x + 12, sy, w - 24, 44), signCol, 4, 6);
    const word = kind === "brass" ? ["पीतल", "deva"] : r.pick(WORDS);
    s += label(word, x + w / 2, sy + 23, 27);
  }
  if (!kind && r() < 0.42) {
    const ac = mix(r.pick(GOODS), "#6a6070", 0.45);
    s += P(`M${x + 6},${oy - 2}H${x + w - 6}L${x + w + 16},${oy + 44}H${x - 16}Z`, ac, 5, 8);
  }
  s += F(R(x - 4, GROUND - 10, w + 8, 10), dark);
  out.glow.push(`<circle cx="${x + w / 2}" cy="${oy - 6}" r="150" fill="url(#lamp)"/>`);

  if (kind === "brass") {
    // The brassworker's bench, with the piece that morphs standing on it.
    const cx = x + w / 2;
    s += P(R(cx - 120, GROUND - 78, 240, 16), "#6b4a34") + F(R(cx - 108, GROUND - 62, 14, 62), "#523726") + F(R(cx + 94, GROUND - 62, 14, 62), "#523726");
    s += `<g transform="translate(${cx} ${GROUND - 168})"><path id="brass-shadow" fill="#0b0810" opacity="0.3" transform="translate(7 9)"/><path id="brass" fill="${BRASS}"/><path id="brass-shine" fill="none" stroke="#f1d58a" stroke-width="5" stroke-linecap="round" opacity="0.8"/></g>`;
    out.brassX = cx;
  }
  if (kind === "paper") {
    // The sign painter's board, repainted from one script to the next.
    const cx = x + w / 2, cy = sy - 34;
    const word = (j, text, cls, size) =>
      `<g clip-path="url(#word-clip-${j})"><rect x="-176" y="-74" width="352" height="148" fill="${BONE}"/>` +
      `<text class="t-${cls}" y="${cls === "urdu" ? -8 : 4}" font-size="${size}" text-anchor="middle" dominant-baseline="central" fill="#7c2a22">${text}</text></g>`;
    s +=
      `<g id="board" transform="translate(${cx} ${cy})">` + P(R(-190, -88, 380, 176), "#3d3040") + F(R(-176, -74, 352, 148), BONE) +
      word(0, "नमस्ते", "deva", 96) + word(1, "آداب", "urdu", 78) + word(2, "Hello", "latin", 100) + `</g>`;
    out.boardX = cx;
  }
  return cull(x - 30, x + w + 30, s);
}

function hangingSign(r, x, i) {
  const col = r.pick(SIGNS), word = WORDS[(i * 5 + 2) % WORDS.length];
  const w = 124, h = 70, drop = 58;
  return (
    F(R(x - 4, 250, 8, 14), INK) + F(R(x - 70, 246, 140, 7), INK) +
    `<g class="swing" data-x="${x}" transform="translate(${x} 253)"><g>` +
    F(R(-w / 2 + 14, 0, 2.5, drop), INK) + F(R(w / 2 - 16, 0, 2.5, drop), INK) +
    P(R(-w / 2, drop, w, h), col, 5, 7) + F(R(-w / 2 + 7, drop + 7, w - 14, h - 14), mix(col, "#fff6e0", 0.35)) +
    label(word, 0, drop + h / 2 + 1, 30) + `</g></g>`
  );
}

function masjid(out) {
  // Jama Masjid from the street: steps, sandstone wall, the great gate, three domes, two
  // minarets. Exterior only.
  const stone = "#9a4a3b", stoneDark = "#733428", marble = "#e9e2d2";
  const cx = 10300;
  // Drawn at full size, then scaled about the foot of the steps so the domes clear the frame.
  const SCALE = 0.8;
  let s = "";
  const minaret = (mx) => {
    let m = P(R(mx - 17, 196, 34, 440), stone);
    for (let y = 230; y < 620; y += 64) m += F(R(mx - 17, y, 34, 9), marble);
    m += P(R(mx - 25, 186, 50, 14), marble, 3, 5) + F(`M${mx - 19},186 C${mx - 22},160 ${mx - 6},150 ${mx},128 C${mx + 6},150 ${mx + 22},160 ${mx + 19},186Z`, marble) + F(R(mx - 1.5, 106, 3, 24), stoneDark);
    return m;
  };
  const dome = (dx, base, rr) => {
    const d = `M${dx - rr},${base} C${dx - rr * 1.3},${base - rr * 1.1} ${dx - rr * 0.35},${base - rr * 1.5} ${dx},${base - rr * 2} C${dx + rr * 0.35},${base - rr * 1.5} ${dx + rr * 1.3},${base - rr * 1.1} ${dx + rr},${base}Z`;
    let m = P(d, marble);
    for (const k of [-0.5, 0, 0.5]) m += `<path d="M${dx + k * rr},${base} Q${dx + k * rr * 1.25},${base - rr} ${dx},${base - rr * 1.9}" fill="none" stroke="#3a3340" stroke-width="3.5"/>`;
    return m + F(R(dx - 1.5, base - rr * 2 - 26, 3, 28), stoneDark) + F(R(dx - rr * 0.8, base - 4, rr * 1.6, 10), stone);
  };
  s += minaret(cx - 432) + minaret(cx + 432);
  s += dome(cx - 250, 404, 62) + dome(cx + 250, 404, 62) + dome(cx, 336, 96);
  s += P(R(cx - 410, 400, 820, 232), stone);
  for (let k = 0; k < 5; k++) {
    for (const side of [-1, 1]) {
      const ax = cx + side * (150 + k * 52) - 18;
      s += F(arch(ax - 5, 455, 46, 150), marble) + F(arch(ax, 462, 36, 143), "#2a1f26");
    }
  }
  s += P(R(cx - 124, 318, 248, 314), stone) + F(R(cx - 124, 318, 248, 12), marble);
  s += F(arch(cx - 86, 368, 172, 264), marble) + F(arch(cx - 70, 388, 140, 244), "#2a1f26");
  s += F(R(cx - 138, 300, 18, 332), stoneDark) + F(R(cx + 120, 300, 18, 332), stoneDark);
  out.steps = [];
  for (let k = 0; k < 6; k++) {
    const y = GROUND - 22 * (k + 1), half = 600 - k * 30;
    s += P(R(cx - half, y, half * 2, 22 * (k + 1)), mix(stone, "#d8b8a0", 0.12 + k * 0.05), 5, 6);
    // Recorded in street coordinates, after the scale applied to the whole building below.
    out.steps.push({ y: GROUND - (GROUND - y) * SCALE, x0: cx - (half - 30) * SCALE, x1: cx + (half - 30) * SCALE });
  }
  out.glow.push(`<path d="${arch(cx - 56, 462, 112, 195)}" fill="#f6c56b" opacity="0.55"/>`, `<circle cx="${cx}" cy="540" r="300" fill="url(#lamp)"/>`);
  return cull(cx - 560, cx + 560, `<g transform="translate(${cx} ${GROUND}) scale(${SCALE}) translate(${-cx} ${-GROUND})">${s}</g>`);
}

export function buildWorld(seed = 1947) {
  const r = rng(seed);
  const out = { glow: [], signs: [] };

  // Far: one continuous skyline silhouette.
  let far = `M-20,900 L-20,600`;
  for (let x = -20; x < VIEW + CAM_MAX * PARALLAX.far + 80; ) {
    const w = r.int(40, 120), y = 640 - r.int(20, 150);
    far += ` L${x},${y} L${x + w},${y}`;
    if (r() < 0.14) far += ` L${x + w},${y - 70} L${x + w + 8},${y - 70} L${x + w + 8},${y}`;
    x += w;
  }
  out.far = `<path id="far-shape" d="${far} L4000,640 L4000,1100 L-20,1100Z"/>`;

  // Mid: plainer, taller, hazier buildings.
  let mid = "";
  for (let x = -60; x < VIEW + CAM_MAX * PARALLAX.mid + 100; ) {
    const w = r.int(170, 300), h = r.int(380, 640), top = GROUND - h;
    const col = mix(r.pick(FACADES), "#6c6680", 0.55);
    let s = P(R(x, top, w, h + 240), col, 6, 8);
    for (let wy = top + 40; wy < GROUND - 120; wy += 86) {
      for (let wx = x + 26; wx < x + w - 40; wx += 62) s += F(arch(wx, wy, 26, 48), mix(col, "#15101a", 0.6));
    }
    if (r() < 0.35) s += F(R(x + w * 0.3, top - 30, w * 0.25, 30), mix(col, "#15101a", 0.3));
    mid += cull(x - 20, x + w + 20, s);
    x += w + r.int(-4, 26);
  }
  out.mid = mid;

  // Near: the shopfronts Kin walks past.
  let near = "";
  let special = { brass: false, paper: false };
  for (let x = -40; x < 9360; ) {
    let kind = null;
    if (!special.brass && x >= 5560) kind = "brass";
    else if (!special.paper && x >= 7180) kind = "paper";
    if (kind) special[kind] = true;
    const w = kind ? 430 : r.int(230, 330);
    const h = kind === "paper" ? 600 : r.int(410, 590);
    near += bay(r, x, w, h, kind, out);
    x += w + r.int(-4, 5);
  }
  let hang = "";
  for (let i = 0, x = 3040; x < 4460; x += 205, i++) {
    hang += hangingSign(r, x, i);
    out.signs.push(x);
  }
  near += cull(2900, 4600, hang);
  near += masjid(out);
  near += `<path d="${R(-200, GROUND, WORLD + 400, 420)}" fill="#3a3038"/><path d="${R(-200, GROUND, WORLD + 400, 7)}" fill="#52464e"/>`;
  out.near = near;

  // Fore: poles and the tangle of cables overhead, closer than the street.
  let fore = "";
  const foreLen = VIEW + CAM_MAX * PARALLAX.fore;
  for (let px = 260, k = 0; px < foreLen + 900; px += 860, k++) {
    let s = F(R(px - 9, 96, 18, 1000), INK) + F(R(px - 70, 128, 140, 9), INK) + F(R(px - 50, 168, 100, 7), INK);
    const nx = px + 860;
    for (let j = 0; j < 5; j++) {
      const y0 = 120 + j * 13 + r.int(-6, 6), sag = r.int(40, 150);
      s += `<path d="M${px},${y0} Q${px + 430 + r.int(-90, 90)},${y0 + sag} ${nx},${120 + j * 13}" fill="none" stroke="${INK}" stroke-width="${j % 2 ? 2 : 3}"/>`;
    }
    if (k % 2 === 0) {
      for (let j = 1; j < 16; j++) {
        const u = j / 16, bx = px + u * 860, by = 190 + Math.sin(u * Math.PI) * 74;
        s += `<path d="M${bx - 11},${by} L${bx + 11},${by} L${bx},${by + 24}Z" fill="${["#a8504a", "#d9c48f", "#5f8f8a", "#e2d6bf"][j % 4]}"/>`;
      }
      s += `<path d="M${px},190 Q${px + 430},338 ${nx},190" fill="none" stroke="${INK}" stroke-width="1.5"/>`;
    }
    fore += cull(px - 80, nx + 20, s);
  }
  out.fore = fore;
  out.glow = out.glow.join("");
  return out;
}

/** Outlines the brass piece morphs through. Each starts at top centre and runs clockwise. */
export function brassShapes() {
  const mirror = (profile) => profile.concat(profile.slice().reverse().map(([x, y]) => [-x, y]));
  const disc = [];
  for (let i = 0; i < 28; i++) disc.push([Math.sin((i / 28) * Math.PI * 2) * 82, -Math.cos((i / 28) * Math.PI * 2) * 82]);
  return {
    names: ["thali", "lota", "ghanti", "diya"],
    shapes: [
      disc,
      mirror([[0, -84], [30, -84], [36, -74], [22, -58], [24, -44], [56, -22], [72, 14], [62, 50], [34, 76], [0, 80]]),
      mirror([[0, -88], [9, -86], [13, -70], [30, -52], [42, -10], [50, 36], [74, 60], [74, 72], [20, 74], [16, 88], [0, 90]]),
      mirror([[0, -92], [7, -70], [3, -48], [10, -40], [52, -34], [82, -22], [60, 12], [26, 30], [30, 60], [58, 74], [58, 84], [0, 84]]),
    ],
  };
}
