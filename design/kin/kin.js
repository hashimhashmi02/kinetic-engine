/*
 * Kin: a parametric paper cut-out puppet that draws itself as SVG.
 *
 * A pose is plain data (joint angles in degrees, a few offsets, an expression name).
 * `Kin.draw(pose)` is a pure function from pose to markup, so poses can be interpolated,
 * scrubbed and rewound, which is the same rule the Kinetic engine follows.
 *
 * Look: every part is a piece of cut paper. Pieces carry paper grain and cast a soft
 * shadow on the piece below (the `kin-paper` filter). No outlines; edges come from the
 * shadows. The palette is muted, with one accent: the red of the kite. The scarf is white and grey.
 *
 * Conventions: origin at the ground between the feet, y grows downward (SVG), angles are
 * measured from straight down and grow toward +x (screen right).
 */
(function (global) {
  "use strict";

  const C = {
    accent: "#d8402f",
    accentDeep: "#a92c22",
    bone: "#e2d6bf",
    boneShade: "#c9bba0",
    indigo: "#39436b",
    skin: "#d9a273",
    skinShade: "#bd8558",
    hair: "#1c1924",
    hairLight: "#2d2a3a",
    mouth: "#2a1420",
    thread: "#efe7d6",
    kurta: "#5f7590",
    kurtaShade: "#4c6079",
    scarf: "#f1ede4",
    scarfGrey: "#9c9ca6",
  };

  const TONES = {
    dawn: { sky: "#62748a", disc: "#f0dcc2", far: "#4b5c72", near: "#2c3848", ground: "#1f2733" },
    haze: { sky: "#b48d58", disc: "#efdcb4", far: "#93703f", near: "#5c4229", ground: "#3b2a1b" },
    dusk: { sky: "#4d3b58", disc: "#e9a66a", far: "#3b2e47", near: "#251c2e", ground: "#1a141f" },
    night: { sky: "#202943", disc: "#d9d3c0", far: "#182036", near: "#101627", ground: "#0b101b" },
  };

  const rad = (d) => (d * Math.PI) / 180;
  const fk = (p, len, a) => [p[0] + len * Math.sin(rad(a)), p[1] + len * Math.cos(rad(a))];
  const n = (v) => Math.round(v * 100) / 100;
  const pt = (p) => `${n(p[0])},${n(p[1])}`;

  /** One piece of paper: grain plus a shadow on whatever lies beneath. */
  const L = (inner) => `<g filter="url(#kin-paper)">${inner}</g>`;
  const fill = (d, color) => `<path d="${d}" fill="${color}"/>`;
  const strip = (points, w, color) =>
    `<path d="M${points.map(pt).join(" L")}" fill="none" stroke="${color}" stroke-width="${w}" stroke-linejoin="round" stroke-linecap="round"/>`;
  const curve = (d, w, color) =>
    `<path d="${d}" fill="none" stroke="${color}" stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round"/>`;

  function roundRect(x, y, w, h, r) {
    return (
      `M${n(x + r)},${n(y)} H${n(x + w - r)} Q${n(x + w)},${n(y)} ${n(x + w)},${n(y + r)} V${n(y + h - r)} ` +
      `Q${n(x + w)},${n(y + h)} ${n(x + w - r)},${n(y + h)} H${n(x + r)} Q${n(x)},${n(y + h)} ${n(x)},${n(y + h - r)} ` +
      `V${n(y + r)} Q${n(x)},${n(y)} ${n(x + r)},${n(y)} Z`
    );
  }

  function ellipse(cx, cy, rx, ry) {
    return `M${n(cx - rx)},${n(cy)} A${n(rx)},${n(ry)} 0 1 0 ${n(cx + rx)},${n(cy)} A${n(rx)},${n(ry)} 0 1 0 ${n(cx - rx)},${n(cy)} Z`;
  }

  /* ---------- props ---------- */

  function kite(cx, cy, hw, hh, rot, noTail) {
    const top = [0, -hh], right = [hw, -hh * 0.12], bottom = [0, hh], left = [-hw, -hh * 0.12];
    const whole = `M${pt(top)} L${pt(right)} L${pt(bottom)} L${pt(left)} Z`;
    const fold = `M${pt(top)} L${pt(right)} L${pt(bottom)} Z`;
    const tail = `M${n(-hw * 0.2)},${n(hh * 0.78)} L0,${n(hh * 1.26)} L${n(hw * 0.2)},${n(hh * 0.78)} Z`;
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) rotate(${n(rot)})">` +
      (noTail ? "" : L(fill(tail, C.bone))) +
      L(fill(whole, C.accent) + fill(fold, C.accentDeep)) +
      L(strip([top, bottom], 2.4, C.bone) + curve(`M${pt(left)} Q0,${n(-hh * 0.86)} ${pt(right)}`, 2.4, C.bone)) +
      `</g>`
    );
  }

  function charkhi(cx, cy, s) {
    // The spool a kite flyer holds: a drum of thread between two discs, a handle each side.
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      L(strip([[-58, 0], [58, 0]], 7, C.boneShade)) +
      L(fill(roundRect(-26, -20, 52, 40, 5), C.accent) +
        [-12, -2, 8, 18].map((y) => strip([[-26, y], [26, y - 6]], 1.6, C.thread)).join("")) +
      L(fill(roundRect(-37, -30, 12, 60, 4), C.indigo) + fill(roundRect(25, -30, 12, 60, 4), C.indigo)) +
      `</g>`
    );
  }

  function chappal(cx, cy, s) {
    // Rubber Hawai chappal seen from above.
    const sole = "M-22,-52 C-40,-40 -34,10 -26,44 C-20,66 20,66 26,44 C34,10 40,-40 22,-52 C10,-60 -10,-60 -22,-52 Z";
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      L(fill(sole, C.bone)) +
      L(strip([[-27, 16], [0, -30], [27, 16]], 7, C.indigo) + `<circle cx="0" cy="-32" r="5.5" fill="${C.indigo}"/>`) +
      `</g>`
    );
  }

  function scarfSwatch(cx, cy, s) {
    const cloth = "M-62,-26 C-30,-36 30,-18 62,-28 L64,24 C30,34 -30,16 -62,26 Z";
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      L([-56, -44, -32].map((x) => strip([[x, 22], [x - 2, 40]], 2.4, C.scarf)).join("")) +
      L(fill(cloth, C.scarf) + strip([[-44, -29], [-44, 23]], 7, C.scarfGrey) + strip([[-30, -30], [-30, 21]], 3, C.scarfGrey) + strip([[44, -24], [44, 28]], 7, C.scarfGrey)) +
      `</g>`
    );
  }

  /* ---------- backdrop: a paper-theatre rooftop view of the old city ---------- */

  function scene(box, toneName) {
    const t = TONES[toneName] || TONES.dusk;
    const [x, y, w, h] = box;
    const fx = (f) => n(x + f * w);
    // Far layer: the old-city skyline with three domes and two minarets, as plain silhouette.
    const base = -118, dx = x + w * 0.66, u = w / 300;
    const dome = (cx, r) =>
      `M${n(cx - r)},${base - 6} C${n(cx - r * 1.25)},${n(base - 6 - r * 1.1)} ${n(cx - r * 0.3)},${n(base - 6 - r * 1.5)} ${n(cx)},${n(base - 6 - r * 1.95)} ` +
      `C${n(cx + r * 0.3)},${n(base - 6 - r * 1.5)} ${n(cx + r * 1.25)},${n(base - 6 - r * 1.1)} ${n(cx + r)},${base - 6} Z`;
    const minaret = (cx) =>
      fill(roundRect(cx - 3.5 * u, base - 96 * u, 7 * u, 100 * u, 1), t.far) + fill(ellipse(cx, base - 99 * u, 5.5 * u, 6 * u), t.far);
    const far =
      fill(`M${fx(0)},${base} L${fx(0)},${base - 20} L${fx(0.1)},${base - 20} L${fx(0.1)},${base - 34} L${fx(0.24)},${base - 34} L${fx(0.24)},${base - 14} L${fx(0.4)},${base - 14} L${fx(0.4)},${base - 6} L${fx(1)},${base - 6} L${fx(1)},${n(y + h)} L${fx(0)},${n(y + h)} Z`, t.far) +
      fill(dome(dx, 22 * u), t.far) + fill(dome(dx - 38 * u, 13 * u), t.far) + fill(dome(dx + 38 * u, 13 * u), t.far) +
      minaret(dx - 64 * u) + minaret(dx + 64 * u);
    // Near layer: flat roofs, a water tank, an aerial, and sagging wires.
    const steps = [[0, 62], [0.13, 62], [0.13, 88], [0.3, 88], [0.3, 50], [0.47, 50], [0.47, 74], [0.6, 74], [0.6, 40], [0.8, 40], [0.8, 68], [1, 68]];
    const near =
      fill(`M${fx(0)},${n(y + h)} ` + steps.map(([f, hh]) => `L${fx(f)},${-hh}`).join(" ") + ` L${fx(1)},${n(y + h)} Z`, t.near) +
      fill(roundRect(x + w * 0.16, -106, w * 0.07, 18, 3), t.near) +
      strip([[x + w * 0.9, -68], [x + w * 0.9, -104]], 1.6, t.near) + strip([[x + w * 0.87, -98], [x + w * 0.93, -98]], 1.6, t.near);
    const wires =
      curve(`M${fx(0)},-150 Q${fx(0.4)},-112 ${fx(1)},-162`, 1, t.near) +
      curve(`M${fx(0)},-176 Q${fx(0.55)},-128 ${fx(1)},-140`, 1, t.near);
    return (
      L(`<rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}" fill="${t.sky}"/>`) +
      L(`<circle cx="${fx(0.24)}" cy="${n(y + h * 0.3)}" r="${n(w * 0.085)}" fill="${t.disc}"/>`) +
      L(far) + wires + L(near) +
      L(`<rect x="${n(x - 4)}" y="-4" width="${n(w + 8)}" height="${n(Math.max(1, y + h + 4))}" fill="${t.ground}"/>`)
    );
  }

  function card(box, color) {
    return L(`<rect x="${box[0]}" y="${box[1]}" width="${box[2]}" height="${box[3]}" fill="${color}"/>`);
  }

  /* ---------- expressions ---------- */

  const EXPR = {
    grin: { eyeL: "dot", eyeR: "dot", browL: [0, -5], browR: [0, 5], mouth: "smile", blush: 1 },
    open: { eyeL: "dot", eyeR: "dot", browL: [-3, -9], browR: [-3, 9], mouth: "open", blush: 1 },
    joy: { eyeL: "happy", eyeR: "happy", browL: [-4, -8], browR: [-4, 8], mouth: "open", blush: 1 },
    smirk: { eyeL: "dot", eyeR: "happy", browL: [-6, -13], browR: [1, 4], mouth: "smirk", blush: 0 },
    wow: { eyeL: "wide", eyeR: "wide", browL: [-8, -4], browR: [-8, 4], mouth: "o", blush: 0 },
    set: { eyeL: "dot", eyeR: "dot", browL: [4, 17], browR: [4, -17], mouth: "flat", blush: 0 },
    oops: { eyeL: "closed", eyeR: "closed", browL: [-2, -15], browR: [-2, 15], mouth: "small", blush: 1, sweat: 1 },
    sulk: { eyeL: "dot", eyeR: "dot", browL: [1, -18], browR: [1, 18], mouth: "pout", blush: 0, look: [0, 3] },
    curious: { eyeL: "wide", eyeR: "dot", browL: [-7, -6], browR: [-1, 9], mouth: "small", blush: 0 },
  };

  function eye(kind, x, y, look, blink) {
    if (kind === "happy") return curve(`M${n(x - 6)},${n(y + 2)} Q${n(x)},${n(y - 7)} ${n(x + 6)},${n(y + 2)}`, 3.2, C.hair);
    if (kind === "closed") return curve(`M${n(x - 6)},${n(y - 1)} Q${n(x)},${n(y + 5)} ${n(x + 6)},${n(y - 1)}`, 3.2, C.hair);
    const r = kind === "wide" ? 6.2 : 4.8;
    const ry = r * (1 - 0.9 * blink);
    if (ry < 1.2) return strip([[x - 5, y], [x + 5, y]], 3, C.hair);
    return (
      `<ellipse cx="${n(x + look[0])}" cy="${n(y + look[1])}" rx="${r}" ry="${n(ry)}" fill="${C.hair}"/>` +
      `<circle cx="${n(x + look[0] + r * 0.32)}" cy="${n(y + look[1] - r * 0.36)}" r="${n(r * 0.26)}" fill="${C.bone}"/>`
    );
  }

  function mouth(kind, mx, my) {
    switch (kind) {
      case "smile":
        return curve(`M${n(mx - 9)},${n(my - 1)} Q${n(mx)},${n(my + 8)} ${n(mx + 9)},${n(my - 1)}`, 3, C.mouth);
      case "open": {
        const d = `M${n(mx - 11)},${n(my - 3)} Q${n(mx)},${n(my - 6)} ${n(mx + 11)},${n(my - 3)} Q${n(mx + 9)},${n(my + 12)} ${n(mx)},${n(my + 12)} Q${n(mx - 9)},${n(my + 12)} ${n(mx - 11)},${n(my - 3)} Z`;
        return fill(d, C.mouth) + fill(roundRect(mx - 7, my - 4.5, 14, 4.5, 1.5), C.bone) + `<ellipse cx="${n(mx)}" cy="${n(my + 9.5)}" rx="5" ry="2.6" fill="${C.accent}"/>`;
      }
      case "o":
        return fill(ellipse(mx, my + 3, 4.6, 6), C.mouth);
      case "smirk":
        return curve(`M${n(mx - 7)},${n(my + 1)} Q${n(mx + 2)},${n(my + 6)} ${n(mx + 10)},${n(my - 4)}`, 3, C.mouth);
      case "flat":
        return strip([[mx - 8, my + 1], [mx + 8, my + 1]], 3.2, C.mouth);
      case "pout":
        return curve(`M${n(mx - 7)},${n(my + 5)} Q${n(mx)},${n(my - 3)} ${n(mx + 7)},${n(my + 5)}`, 3, C.mouth);
      default:
        return curve(`M${n(mx - 5)},${n(my)} Q${n(mx)},${n(my + 4)} ${n(mx + 5)},${n(my)}`, 3, C.mouth);
    }
  }

  // Head drawn in its own space: origin at the base of the neck, head centre near (0, -50).
  function head(pose) {
    const back = pose.view === "back";
    const turn = back ? 0 : pose.head.turn;
    const tx = turn * 12;
    const e = EXPR[pose.expr] || EXPR.grin;
    const look = e.look || [turn * 1.6, 0];

    const face = "M-45,-52 C-45,-88 -26,-99 0,-99 C26,-99 45,-88 45,-52 C45,-22 27,-4 0,-4 C-27,-4 -45,-22 -45,-52 Z";
    const ears = fill(ellipse(-45 - turn * 4, -46, 8, 10), C.skinShade) + fill(ellipse(45 - turn * 4, -46, 8, 10), C.skinShade);

    if (back) {
      const cap =
        "M-51,-44 C-57,-96 -29,-113 2,-111 L11,-127 L17,-108 C41,-104 57,-90 51,-44 " +
        "C47,-30 40,-22 33,-27 C27,-17 19,-17 13,-25 C8,-15 -2,-15 -8,-25 C-14,-16 -23,-17 -28,-27 C-36,-20 -47,-30 -51,-44 Z";
      return L(ears) + L(fill(face, C.skin)) + L(fill(cap, C.hair) + curve("M-30,-90 Q-12,-102 6,-98", 3, C.hairLight));
    }

    const hairBack = "M-53,-46 C-58,-98 -28,-113 2,-111 C36,-111 58,-94 53,-46 C52,-36 47,-32 45,-40 L-45,-40 C-47,-32 -52,-36 -53,-46 Z";
    let f = fill(face, C.skin);
    // Shadow-side cheek: a second sheet of darker paper tucked under the hairline.
    f += fill(`M${n(24 + tx * 0.6)},-96 C40,-88 45,-70 45,-52 C45,-24 29,-6 4,-4 C24,-22 30,-56 ${n(24 + tx * 0.6)},-96 Z`, C.skinShade);
    if (e.blush) {
      f += `<circle cx="${n(-27 + tx)}" cy="-28" r="7" fill="${C.accent}" opacity="0.28"/>`;
      f += `<circle cx="${n(27 + tx)}" cy="-28" r="7" fill="${C.accent}" opacity="0.22"/>`;
    }
    f += eye(e.eyeL, -17 + tx, -45, look, pose.blink) + eye(e.eyeR, 17 + tx, -45, look, pose.blink);
    const brow = (x, b, dir) =>
      `<g transform="translate(${n(x)} ${n(-60 + b[0])}) rotate(${n(b[1])})">` + curve(`M-7,${dir} Q0,-2 7,${-dir}`, 3, C.hair) + `</g>`;
    f += brow(-17 + tx, e.browL, 1) + brow(17 + tx, e.browR, -1);
    f += curve(`M${n(tx * 1.3 - 1)},-38 q4,3 0,6`, 2.4, C.skinShade);
    f += mouth(e.mouth, tx * 1.2, -22);
    if (e.sweat) f += fill("M38,-90 q7,10 0,14 q-7,-4 0,-14 Z", "#9db9c9");

    const fringe =
      "M-47,-54 C-50,-93 -25,-105 0,-104 L11,-126 L17,-103 C37,-99 51,-85 47,-52 " +
      "C42,-68 32,-77 19,-80 C21,-70 15,-63 8,-61 C9,-72 0,-80 -11,-80 C-25,-78 -40,-69 -47,-54 Z";
    return (
      L(fill(hairBack, C.hair)) + L(ears) + L(f) +
      `<g transform="translate(${n(tx * 0.25)} 0)">` + L(fill(fringe, C.hair) + curve("M-30,-90 Q-12,-102 4,-98", 3, C.hairLight)) + `</g>`
    );
  }

  /* ---------- body ---------- */

  const BASE = {
    lift: 0, drop: 0, lean: 0, squash: 1, scarf: 0, blink: 0,
    head: { rot: 0, turn: 0 },
    armL: { sh: -12, el: -5 },
    armR: { sh: 12, el: 5 },
    legL: { hip: -6, knee: 0, foot: 0 },
    legR: { hip: 6, knee: 0, foot: 0 },
    kite: "back", kiteRot: 10, kiteAt: [95, -395],
    legsFront: false, expr: "grin", view: "front",
  };

  function merge(a, b) {
    const out = Array.isArray(a) ? a.slice() : Object.assign({}, a);
    for (const k in b) {
      const v = b[k];
      out[k] = v && typeof v === "object" && !Array.isArray(v) && a && typeof a[k] === "object" ? merge(a[k], v) : v;
    }
    return out;
  }

  function lerpPose(a, b, t) {
    if (typeof a === "number" && typeof b === "number") return a + (b - a) * t;
    if (Array.isArray(a) && Array.isArray(b)) return a.map((v, i) => lerpPose(v, b[i], t));
    if (a && b && typeof a === "object" && typeof b === "object") {
      const out = {};
      for (const k in a) out[k] = k in b ? lerpPose(a[k], b[k], t) : a[k];
      return out;
    }
    return t < 0.5 ? a : b;
  }

  function leg(side, spec, hipY, turn) {
    const hip = [side * 12, hipY];
    const knee = fk(hip, 37, spec.hip);
    const shinA = spec.hip + spec.knee;
    const ankle = fk(knee, 37, shinA);
    const cuff = [hip[0] + (knee[0] - hip[0]) * 0.8, hip[1] + (knee[1] - hip[1]) * 0.8];
    const dir = turn > 0.35 ? 1 : side;
    const foot =
      `<g transform="translate(${pt(ankle)}) rotate(${n(-shinA * spec.foot)})">` +
      fill(roundRect(dir > 0 ? -7 : -21, 3, 28, 5, 2.5), C.boneShade) +
      fill(ellipse(6 * dir, 0, 12, 5.5), C.skinShade) +
      strip([[13 * dir, 2], [2 * dir, -5]], 2.6, C.indigo) +
      `</g>`;
    return strip([hip, knee, ankle], 9, C.skin) + strip([hip, cuff], 17, C.indigo) + foot;
  }

  function arm(side, spec, shoulderY) {
    const sh = [side * 26, shoulderY];
    const el = fk(sh, 36, spec.sh);
    const hand = fk(el, 34, spec.sh + spec.el);
    const cuff = [el[0] + (hand[0] - el[0]) * 0.3, el[1] + (hand[1] - el[1]) * 0.3];
    return {
      markup: strip([el, hand], 8.5, C.skin) + strip([sh, el, cuff], 14, C.kurtaShade) + `<circle cx="${n(hand[0])}" cy="${n(hand[1])}" r="7.5" fill="${C.skin}"/>`,
      hand,
    };
  }

  function scarf(neckY, sway, turn) {
    const band = `M-25,${neckY - 9} Q0,${neckY + 7} 25,${neckY - 9} Q29,${neckY + 5} 0,${neckY + 16} Q-29,${neckY + 5} -25,${neckY - 9} Z`;
    const x0 = -12 - turn * 6;
    const pts = (dx, len, lag) => [[x0 + dx, neckY + 6], [x0 + dx - 5 + sway * 15 * lag, neckY + 6 + len * 0.55], [x0 + dx - 2 + sway * 32 * lag, neckY + 6 + len]];
    const along = (p, f) => { const [a, b, c] = p; return f < 0.55 ? [a[0] + (b[0] - a[0]) * (f / 0.55), a[1] + (b[1] - a[1]) * (f / 0.55)] : [b[0] + (c[0] - b[0]) * ((f - 0.55) / 0.45), b[1] + (c[1] - b[1]) * ((f - 0.55) / 0.45)]; };
    const front = pts(-4, 62, 1);
    // Two grey bands near the end of the front tail.
    const bands = [0.7, 0.86].map((f) => { const p = along(front, f); return strip([[p[0] - 6, p[1]], [p[0] + 6, p[1]]], 4.5, C.scarfGrey); }).join("");
    return L(strip(pts(7, 46, 0.8), 12, C.scarfGrey)) + L(strip(front, 12, C.scarf) + bands) + L(fill(band, C.scarf) + curve(`M-17,${neckY + 2} Q0,${neckY + 13} 17,${neckY + 2}`, 2, C.scarfGrey));
  }

  function draw(input) {
    const pose = merge(BASE, input || {});
    const back = pose.view === "back";
    const turn = back ? 0 : pose.head.turn;
    const hipY = -74 + pose.drop;
    const shoulderY = -150 + pose.drop;
    const neckY = -162 + pose.drop;
    const hem = hipY + 30;

    const px = turn * 9;
    let kurta = fill(`M-26,${shoulderY - 6} Q0,${shoulderY - 15} 26,${shoulderY - 6} L37,${hem} Q0,${hem + 9} -37,${hem} Z`, C.kurta);
    if (!back) {
      kurta +=
        strip([[px, shoulderY - 8], [px, shoulderY + 34]], 5, C.kurtaShade) +
        `<circle cx="${n(px)}" cy="${shoulderY + 10}" r="1.7" fill="${C.bone}"/><circle cx="${n(px)}" cy="${shoulderY + 24}" r="1.7" fill="${C.bone}"/>`;
    }
    kurta += strip([[31, hem + 2], [29, hem - 20]], 2, C.kurtaShade);

    const aL = arm(-1, pose.armL, shoulderY);
    const aR = arm(1, pose.armR, shoulderY);
    const legs = L(leg(-1, pose.legL, hipY, turn)) + L(leg(1, pose.legR, hipY, turn));
    const backKite = pose.kite === "back" ? kite(-turn * 18, -138 + pose.drop, 74, 86, pose.kiteRot, !back) : "";
    const neck = L(fill(roundRect(-7, neckY - 12, 14, 24, 3), C.skinShade));
    const headMarkup = `<g transform="translate(0 ${neckY}) rotate(${n(pose.head.rot)})">${head(pose)}</g>`;
    const scarfMarkup = scarf(neckY, pose.scarf, turn);
    const farArmBehind = turn > 0.3;

    const sq = pose.squash;
    const upperXf = `translate(0 ${hipY}) rotate(${n(pose.lean)}) scale(${n(1 / Math.sqrt(sq))} ${n(sq)}) translate(0 ${-hipY})`;

    let upper;
    if (back) {
      upper = neck + L(aL.markup) + L(aR.markup) + L(kurta) + scarfMarkup + backKite + headMarkup;
    } else {
      upper =
        backKite + (farArmBehind ? L(aL.markup) : "") + neck + L(kurta) + (pose.legsFront ? legs : "") +
        scarfMarkup + headMarkup + (farArmBehind ? "" : L(aL.markup)) + L(aR.markup);
    }

    let fly = "";
    if (pose.kite === "fly") {
      // Hand position in world space: apply the upper-body lean about the hips.
      const a = rad(pose.lean), hx = aR.hand[0], hy = aR.hand[1] - hipY;
      const hand = [hx * Math.cos(a) - hy * Math.sin(a), hipY + hx * Math.sin(a) + hy * Math.cos(a)];
      const k = pose.kiteAt;
      fly =
        curve(`M${pt(hand)} Q${n(hand[0] + 10)},${n((hand[1] + k[1]) / 2 + 40)} ${n(k[0] - 4)},${n(k[1] + 30)}`, 1.1, C.thread) +
        kite(k[0], k[1], 42, 48, pose.kiteRot + 12);
    }

    const s = Math.max(0.45, 1 - pose.lift / 60);
    const shadow = `<ellipse cx="0" cy="5" rx="${n(48 * s)}" ry="${n(6 * s)}" fill="#000" opacity="0.32"/>`;

    return (
      shadow +
      `<g transform="translate(0 ${n(-pose.lift)})">` +
      (pose.legsFront && !back ? "" : legs) +
      `<g transform="${upperXf}">${upper}</g>` +
      fly +
      `</g>`
    );
  }

  // Paper grain multiplied into each piece, then a soft shadow cast onto the piece below.
  const DEFS =
    `<svg width="0" height="0" style="position:absolute" aria-hidden="true" focusable="false"><defs>` +
    `<filter id="kin-paper" x="-25%" y="-25%" width="150%" height="150%" color-interpolation-filters="sRGB">` +
    `<feTurbulence type="fractalNoise" baseFrequency="0.85" numOctaves="3" seed="11" result="noise"/>` +
    `<feColorMatrix in="noise" type="matrix" values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 -0.75 0.47" result="specks"/>` +
    `<feComposite in="specks" in2="SourceAlpha" operator="in" result="grain"/>` +
    `<feMerge result="sheet"><feMergeNode in="SourceGraphic"/><feMergeNode in="grain"/></feMerge>` +
    `<feDropShadow in="sheet" dx="1.2" dy="2.6" stdDeviation="2" flood-color="#07050a" flood-opacity="0.5"/>` +
    `</filter></defs></svg>`;

  function svg(pose, opts) {
    const o = Object.assign({ box: [-150, -330, 300, 360], label: "Kin", tone: "dusk" }, opts || {});
    return (
      `<svg viewBox="${o.box.join(" ")}" role="img" aria-label="${o.label}" xmlns="http://www.w3.org/2000/svg">` +
      (o.tone ? scene(o.box, o.tone) : "") + draw(pose) + `</svg>`
    );
  }

  function headSvg(expr, turn, label, color) {
    const pose = merge(BASE, { expr, head: { rot: 0, turn: turn || 0 } });
    const box = [-80, -140, 160, 150];
    return `<svg viewBox="${box.join(" ")}" role="img" aria-label="${label || expr}" xmlns="http://www.w3.org/2000/svg">${card(box, color || TONES.dusk.far)}${head(pose)}</svg>`;
  }

  const POSES = {
    hero: { head: { rot: -3, turn: 0.15 }, armL: { sh: -34, el: 98 }, armR: { sh: 16, el: 4 }, legL: { hip: -8 }, legR: { hip: 8 }, expr: "grin", scarf: 0.3 },
    point: {
      lean: 5, head: { rot: 4, turn: 0.5 }, armL: { sh: -34, el: 98 }, armR: { sh: 98, el: 6 },
      legL: { hip: -10 }, legR: { hip: 13 }, expr: "open", scarf: -0.3,
    },
    run: {
      lean: 16, lift: 16, head: { rot: -4, turn: 0.8 }, armL: { sh: 52, el: 72 }, armR: { sh: -56, el: 76 },
      legL: { hip: -38, knee: -70, foot: 1 }, legR: { hip: 58, knee: -78, foot: 1 }, expr: "open", scarf: -1,
    },
    fly: {
      lift: 4, lean: -4, head: { rot: -9, turn: 0.35 }, armL: { sh: -104, el: -62 }, armR: { sh: 162, el: 6 },
      legL: { hip: -13 }, legR: { hip: 9 }, expr: "joy", kite: "fly", scarf: 0.4,
    },
    squat: {
      drop: 50, head: { rot: 7, turn: 0.3 }, armL: { sh: -30, el: 40 }, armR: { sh: 30, el: -40 },
      legL: { hip: -72, knee: 142 }, legR: { hip: 72, knee: -142 }, legsFront: true, expr: "curious",
    },
  };

  global.Kin = { C, TONES, BASE, POSES, EXPR, DEFS, draw, svg, headSvg, scene, card, merge, lerpPose, kite, charkhi, chappal, scarfSwatch };
})(window);
