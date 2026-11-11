/*
 * Kin: a parametric character rig that draws itself as SVG.
 *
 * A pose is plain data (joint angles in degrees, a few offsets, an expression name).
 * `Kin.draw(pose)` is a pure function from pose to markup, so poses can be interpolated,
 * scrubbed and rewound, which is the same rule the Kinetic engine follows.
 *
 * Conventions: origin at the ground between the feet, y grows downward (SVG), angles are
 * measured from straight down and grow toward +x (screen right).
 */
(function (global) {
  "use strict";

  const C = {
    ink: "#17132b",
    haldi: "#f5b921",
    rani: "#e3266f",
    tota: "#3fbf5a",
    neel: "#2b3fae",
    madder: "#d43a2f",
    skin: "#b9703f",
    hair: "#221f4a",
    hairLight: "#4a4796",
    paper: "#f7f1e1",
    mouth: "#5a1630",
  };

  // Colour plates sit slightly off the ink plate, like cheap two-pass comic printing.
  const OFFSET = "translate(2.4 1.8)";
  const INK_W = 4.5;

  const rad = (d) => (d * Math.PI) / 180;
  const fk = (p, len, a) => [p[0] + len * Math.sin(rad(a)), p[1] + len * Math.cos(rad(a))];
  const n = (v) => Math.round(v * 100) / 100;
  const pt = (p) => `${n(p[0])},${n(p[1])}`;
  let uid = 0;

  function shape(d, fill, sw) {
    return (
      `<path d="${d}" fill="${fill}" transform="${OFFSET}"/>` +
      `<path d="${d}" fill="none" stroke="${C.ink}" stroke-width="${sw || INK_W}" stroke-linejoin="round" stroke-linecap="round"/>`
    );
  }

  function line(d, sw, color) {
    return `<path d="${d}" fill="none" stroke="${color || C.ink}" stroke-width="${sw || INK_W}" stroke-linejoin="round" stroke-linecap="round"/>`;
  }

  function limb(points, w, color) {
    const d = "M" + points.map(pt).join(" L");
    return (
      `<path d="${d}" fill="none" stroke="${C.ink}" stroke-width="${w + 2 * INK_W}" stroke-linejoin="round" stroke-linecap="round"/>` +
      `<path d="${d}" fill="none" stroke="${color}" stroke-width="${w}" stroke-linejoin="round" stroke-linecap="round" transform="${OFFSET}"/>`
    );
  }

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

  function halftone(d, x, y, w, h) {
    const id = "kin-clip-" + uid++;
    return (
      `<clipPath id="${id}"><path d="${d}"/></clipPath>` +
      `<rect x="${n(x)}" y="${n(y)}" width="${n(w)}" height="${n(h)}" fill="url(#kin-dots)" clip-path="url(#${id})"/>`
    );
  }

  /* ---------- props ---------- */

  function kite(cx, cy, hw, hh, rot, noTail) {
    const top = [0, -hh], right = [hw, -hh * 0.12], bottom = [0, hh], left = [-hw, -hh * 0.12];
    const whole = `M${pt(top)} L${pt(right)} L${pt(bottom)} L${pt(left)} Z`;
    const leftHalf = `M${pt(top)} L${pt(bottom)} L${pt(left)} Z`;
    const rightHalf = `M${pt(top)} L${pt(right)} L${pt(bottom)} Z`;
    const tail = `M${n(-hw * 0.16)},${n(hh * 0.8)} L0,${n(hh * 1.22)} L${n(hw * 0.16)},${n(hh * 0.8)} Z`;
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) rotate(${n(rot)})">` +
      (noTail ? "" : shape(tail, C.haldi)) +
      `<path d="${leftHalf}" fill="${C.rani}" transform="${OFFSET}"/>` +
      `<path d="${rightHalf}" fill="${C.tota}" transform="${OFFSET}"/>` +
      halftone(rightHalf, 0, -hh, hw, hh * 2) +
      line(whole) +
      line(`M${pt(top)} L${pt(bottom)}`, 3) +
      line(`M${pt(left)} Q0,${n(-hh * 0.82)} ${pt(right)}`, 3) +
      `</g>`
    );
  }

  function charkhi(cx, cy, s) {
    // The spool a kite flyer holds: a drum of thread between two discs, with a handle each side.
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      limb([[-58, 0], [58, 0]], 7, C.haldi) +
      shape(roundRect(-26, -20, 52, 40, 6), C.rani) +
      [-14, -4, 6, 16].map((y) => line(`M-26,${y} L26,${y - 6}`, 2.2, C.paper)).join("") +
      shape(roundRect(-36, -30, 12, 60, 5), C.neel) +
      shape(roundRect(24, -30, 12, 60, 5), C.neel) +
      `</g>`
    );
  }

  function chappal(cx, cy, s) {
    // Rubber Hawai chappal seen from above.
    const sole = "M-22,-52 C-40,-40 -34,10 -26,44 C-20,66 20,66 26,44 C34,10 40,-40 22,-52 C10,-60 -10,-60 -22,-52 Z";
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      shape(sole, C.paper) +
      limb([[-27, 16], [0, -30], [27, 16]], 6, C.neel) +
      `<circle cx="0" cy="-32" r="5" fill="${C.neel}" stroke="${C.ink}" stroke-width="3"/>` +
      `</g>`
    );
  }

  function gamchaSwatch(cx, cy, s) {
    const cloth = "M-60,-30 C-30,-40 30,-22 60,-32 L62,28 C30,38 -30,20 -60,30 Z";
    const id = "kin-clip-" + uid++;
    let checks = "";
    for (let i = -50; i <= 50; i += 20) checks += `<path d="M${i},-44 L${i},44" stroke="${C.paper}" stroke-width="5"/>`;
    for (let j = -22; j <= 22; j += 22) checks += `<path d="M-70,${j} L70,${j}" stroke="${C.paper}" stroke-width="5" opacity="0.75"/>`;
    return (
      `<g transform="translate(${n(cx)} ${n(cy)}) scale(${s})">` +
      `<path d="${cloth}" fill="${C.madder}" transform="${OFFSET}"/>` +
      `<clipPath id="${id}"><path d="${cloth}"/></clipPath><g clip-path="url(#${id})">${checks}</g>` +
      line(cloth) +
      [-52, -38, -24].map((x, i) => line(`M${x},${28 - i * 2} l-3,14`, 3)).join("") +
      `</g>`
    );
  }

  /* ---------- expressions ---------- */

  const EXPR = {
    grin: { eyeL: "open", eyeR: "open", browL: [0, -6], browR: [0, 6], mouth: "grin", blush: 1 },
    open: { eyeL: "open", eyeR: "open", browL: [-3, -10], browR: [-3, 10], mouth: "open", blush: 1 },
    joy: { eyeL: "happy", eyeR: "happy", browL: [-4, -8], browR: [-4, 8], mouth: "open", blush: 1 },
    smirk: { eyeL: "open", eyeR: "happy", browL: [-7, -14], browR: [2, 4], mouth: "smirk", blush: 0 },
    wow: { eyeL: "wide", eyeR: "wide", browL: [-9, -4], browR: [-9, 4], mouth: "o", blush: 0 },
    set: { eyeL: "open", eyeR: "open", browL: [5, 18], browR: [5, -18], mouth: "flat", blush: 0 },
    oops: { eyeL: "closed", eyeR: "closed", browL: [-2, -16], browR: [-2, 16], mouth: "small", blush: 1, sweat: 1 },
    sulk: { eyeL: "open", eyeR: "open", browL: [0, -20], browR: [0, 20], mouth: "pout", blush: 0, look: [0, 4] },
    curious: { eyeL: "wide", eyeR: "open", browL: [-8, -6], browR: [-1, 10], mouth: "small", blush: 0 },
  };

  function eye(kind, x, y, rx, look, blink) {
    if (kind === "happy") return line(`M${n(x - 9)},${n(y + 3)} Q${n(x)},${n(y - 11)} ${n(x + 9)},${n(y + 3)}`, 4.5);
    if (kind === "closed") return line(`M${n(x - 9)},${n(y - 1)} Q${n(x)},${n(y + 8)} ${n(x + 9)},${n(y - 1)}`, 4.5);
    const wide = kind === "wide";
    const ry = (wide ? 15 : 12.5) * (1 - 0.9 * blink);
    const r = wide ? 5 : 6.5;
    const ex = wide ? rx + 1.5 : rx;
    if (ry < 3) return line(`M${n(x - 9)},${n(y)} L${n(x + 9)},${n(y)}`, 4.5);
    const id = "kin-clip-" + uid++;
    const white = ellipse(x, y, ex, ry);
    return (
      `<path d="${white}" fill="${C.paper}"/>` +
      `<clipPath id="${id}"><path d="${white}"/></clipPath>` +
      `<g clip-path="url(#${id})"><circle cx="${n(x + look[0])}" cy="${n(y + look[1])}" r="${r}" fill="${C.ink}"/>` +
      `<circle cx="${n(x + look[0] + 2.2)}" cy="${n(y + look[1] - 2.4)}" r="1.9" fill="${C.paper}"/></g>` +
      line(white, 3.6)
    );
  }

  function mouth(kind, mx, my) {
    switch (kind) {
      case "grin": {
        const d = `M${n(mx - 17)},${n(my - 4)} Q${n(mx)},${n(my + 17)} ${n(mx + 17)},${n(my - 4)} Q${n(mx)},${n(my + 2)} ${n(mx - 17)},${n(my - 4)} Z`;
        return `<path d="${d}" fill="${C.paper}"/>` + line(d, 3.6) + `<rect x="${n(mx - 3)}" y="${n(my - 1)}" width="6" height="6.5" rx="1.5" fill="${C.mouth}"/>`;
      }
      case "open": {
        const d = `M${n(mx - 16)},${n(my - 5)} Q${n(mx)},${n(my - 9)} ${n(mx + 16)},${n(my - 5)} Q${n(mx + 13)},${n(my + 17)} ${n(mx)},${n(my + 17)} Q${n(mx - 13)},${n(my + 17)} ${n(mx - 16)},${n(my - 5)} Z`;
        const id = "kin-clip-" + uid++;
        return (
          `<path d="${d}" fill="${C.mouth}"/><clipPath id="${id}"><path d="${d}"/></clipPath>` +
          `<g clip-path="url(#${id})"><ellipse cx="${n(mx)}" cy="${n(my + 16)}" rx="9" ry="7" fill="${C.rani}"/>` +
          `<rect x="${n(mx - 16)}" y="${n(my - 10)}" width="32" height="9" fill="${C.paper}"/>` +
          `<rect x="${n(mx - 3)}" y="${n(my - 10)}" width="6" height="9" fill="${C.mouth}"/></g>` +
          line(d, 3.6)
        );
      }
      case "o":
        return `<path d="${ellipse(mx, my + 3, 7, 9)}" fill="${C.mouth}"/>` + line(ellipse(mx, my + 3, 7, 9), 3.6);
      case "smirk":
        return line(`M${n(mx - 11)},${n(my + 1)} Q${n(mx + 3)},${n(my + 10)} ${n(mx + 15)},${n(my - 6)}`, 4.2);
      case "flat": {
        const d = roundRect(mx - 14, my - 5, 28, 12, 4.5);
        return `<path d="${d}" fill="${C.paper}"/>` + line(d, 3.6) + line(`M${n(mx - 14)},${n(my + 1)} H${n(mx + 14)}`, 2.2) + `<rect x="${n(mx - 3)}" y="${n(my - 4)}" width="6" height="5" fill="${C.mouth}"/>`;
      }
      case "pout":
        return line(`M${n(mx - 9)},${n(my + 6)} Q${n(mx)},${n(my - 4)} ${n(mx + 9)},${n(my + 6)}`, 4.2);
      default:
        return line(`M${n(mx - 8)},${n(my)} Q${n(mx)},${n(my + 7)} ${n(mx + 8)},${n(my)}`, 4.2);
    }
  }

  // Head drawn in its own space: origin at the base of the neck, head centre at (0, -50).
  function head(pose) {
    const back = pose.view === "back";
    const turn = back ? 0 : pose.head.turn;
    const tx = turn * 13;
    const e = EXPR[pose.expr] || EXPR.grin;
    const look = e.look || [turn * 2.5, 0];
    let s = "";

    s += shape(ellipse(-49 - turn * 5, -46, 10, 12), C.skin) + shape(ellipse(49 - turn * 5, -46, 10, 12), C.skin);
    const skull = roundRect(-48, -94, 96, 88, 40);
    s += shape(skull, C.skin);

    if (back) {
      const hairBack =
        "M-53,-40 C-58,-78 -34,-104 -6,-102 L14,-124 L16,-100 C40,-96 58,-74 53,-40 " +
        "L44,-22 L34,-32 L22,-16 L10,-30 L-2,-14 L-14,-30 L-26,-16 L-38,-30 L-46,-20 Z";
      s += shape(hairBack, C.hair);
      s += line("M-30,-84 Q-12,-96 4,-92", 4, C.hairLight);
      return s;
    }

    s += halftone(skull, 14 + tx, -94, 40, 88);

    if (e.blush) {
      for (const side of [-1, 1]) {
        const bx = side * 31 + tx, by = -27;
        s += `<g fill="${C.rani}" opacity="0.6"><circle cx="${n(bx - 5)}" cy="${by}" r="2.6"/><circle cx="${n(bx + 2)}" cy="${by - 3}" r="2.6"/><circle cx="${n(bx + 3)}" cy="${by + 4}" r="2.6"/></g>`;
      }
    }

    const eyeY = -45;
    s += eye(e.eyeL, -19 + tx, eyeY, 10.5 * (1 + 0.2 * turn), look, pose.blink);
    s += eye(e.eyeR, 19 + tx, eyeY, 10.5 * (1 - 0.22 * turn), look, pose.blink);

    const brow = (x, b, dir) =>
      `<g transform="translate(${n(x)} ${n(-66 + b[0])}) rotate(${n(b[1])})">` + line(`M-10,${dir * 1.5} Q0,-3 10,${-dir * 1.5}`, 5.5) + `</g>`;
    s += brow(-19 + tx, e.browL, 1) + brow(19 + tx, e.browR, -1);

    s += line(`M${n(tx * 1.4 - 2)},-38 q6,4 0,8`, 3.4);
    s += mouth(e.mouth, tx * 1.2, -21);

    if (e.sweat) {
      const d = "M40,-88 q9,12 0,17 q-9,-5 0,-17 Z";
      s += `<path d="${d}" fill="#bfe6f2"/>` + line(d, 3);
    }

    const hair =
      "M-51,-42 C-60,-76 -40,-106 -10,-104 L16,-128 L16,-102 C42,-100 60,-76 51,-42 " +
      "L41,-66 L31,-52 L18,-70 L5,-54 L-10,-70 L-24,-54 L-38,-67 Z";
    s += `<g transform="translate(${n(tx * 0.25)} 0)">` + shape(hair, C.hair) + line("M-32,-86 Q-14,-98 2,-94", 4, C.hairLight) + `</g>`;
    return s;
  }

  /* ---------- body ---------- */

  const BASE = {
    lift: 0, drop: 0, lean: 0, squash: 1, scarf: 0, blink: 0,
    head: { rot: 0, turn: 0 },
    armL: { sh: -14, el: -6, point: 0 },
    armR: { sh: 14, el: 6, point: 0 },
    legL: { hip: -7, knee: 0, foot: 0 },
    legR: { hip: 7, knee: 0, foot: 0 },
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
    const hip = [side * 15, hipY];
    const knee = fk(hip, 38, spec.hip);
    const shinA = spec.hip + spec.knee;
    const ankle = fk(knee, 38, shinA);
    const short = [hip[0] + (knee[0] - hip[0]) * 0.62, hip[1] + (knee[1] - hip[1]) * 0.62];
    const dir = turn > 0.35 ? 1 : side;
    const foot =
      `<g transform="translate(${pt(ankle)}) rotate(${n(-shinA * spec.foot)})">` +
      shape(roundRect(dir > 0 ? -9 : -25, 3, 34, 7, 3.5), C.paper, 3.6) +
      shape(ellipse(7 * dir, -1, 15, 7.5), C.skin, 3.8) +
      limb([[16 * dir, 2], [2 * dir, -8]], 3.2, C.neel) +
      `</g>`;
    return limb([hip, knee, ankle], 13, C.skin) + limb([hip, short], 21, C.neel) + foot;
  }

  function arm(side, spec, shoulderY) {
    const sh = [side * 33, shoulderY];
    const el = fk(sh, 38, spec.sh);
    const foreA = spec.sh + spec.el;
    const hand = fk(el, 36, foreA);
    const cuff = [sh[0] + (el[0] - sh[0]) * 0.7, sh[1] + (el[1] - sh[1]) * 0.7];
    let s = limb([sh, el, hand], 13, C.skin) + limb([sh, cuff], 19, C.haldi);
    if (spec.point > 0.5) s += limb([hand, fk(hand, 19, foreA)], 5.5, C.skin);
    s += shape(ellipse(hand[0], hand[1], 10, 10), C.skin, 4);
    return { markup: s, hand };
  }

  function gamcha(neckY, sway, turn) {
    const band = `M-25,${neckY - 7} Q0,${neckY + 9} 25,${neckY - 7} Q28,${neckY + 4} 0,${neckY + 15} Q-28,${neckY + 4} -25,${neckY - 7} Z`;
    const x0 = -13 - turn * 6;
    const tail = (dx, len, lag) => {
      const pts = [[x0 + dx, neckY + 6], [x0 + dx - 6 + sway * 16 * lag, neckY + 6 + len * 0.55], [x0 + dx - 3 + sway * 34 * lag, neckY + 6 + len]];
      const d = "M" + pts.map(pt).join(" L");
      return limb(pts, 13, C.madder) + `<path d="${d}" fill="none" stroke="${C.paper}" stroke-width="3" stroke-dasharray="3 8" transform="${OFFSET}"/>`;
    };
    return tail(7, 50, 0.8) + tail(-4, 64, 1) + shape(band, C.madder);
  }

  function draw(input) {
    const pose = merge(BASE, input || {});
    const back = pose.view === "back";
    const turn = back ? 0 : pose.head.turn;
    const hipY = -76 + pose.drop;
    const shoulderY = -158 + pose.drop;
    const neckY = -170 + pose.drop;

    const torso = `M-35,${shoulderY - 8} Q0,${shoulderY - 20} 35,${shoulderY - 8} L43,${hipY + 16} Q0,${hipY + 27} -43,${hipY + 16} Z`;
    const px = turn * 11;
    let torsoMarkup = shape(torso, C.haldi) + halftone(torso, 10 + px, shoulderY - 20, 40, 130);
    if (!back) {
      torsoMarkup +=
        line(`M${n(px)},${shoulderY - 12} L${n(px)},${shoulderY + 30}`, 3) +
        `<circle cx="${n(px)}" cy="${shoulderY + 6}" r="2.6" fill="${C.ink}"/><circle cx="${n(px)}" cy="${shoulderY + 20}" r="2.6" fill="${C.ink}"/>`;
    }
    torsoMarkup += line(`M${n(43 - 3)},${hipY + 17} l-2,-22`, 3);

    const aL = arm(-1, pose.armL, shoulderY);
    const aR = arm(1, pose.armR, shoulderY);
    const legs = leg(-1, pose.legL, hipY, turn) + leg(1, pose.legR, hipY, turn);
    const backKite = pose.kite === "back" ? kite(-turn * 20, -144 + pose.drop, 80, 92, pose.kiteRot, !back) : "";
    const neck = shape(roundRect(-9, neckY - 12, 18, 26, 4), C.skin);
    const headMarkup = `<g transform="translate(0 ${neckY}) rotate(${n(pose.head.rot)})">${head(pose)}</g>`;
    const scarf = gamcha(neckY, pose.scarf, turn);
    const farArmBehind = turn > 0.3;

    const sq = pose.squash;
    const upperXf = `translate(0 ${hipY}) rotate(${n(pose.lean)}) scale(${n(1 / Math.sqrt(sq))} ${n(sq)}) translate(0 ${-hipY})`;

    let upper = "";
    if (back) {
      upper = neck + aL.markup + aR.markup + torsoMarkup + scarf + backKite + headMarkup;
    } else {
      upper =
        backKite + (farArmBehind ? aL.markup : "") + neck + torsoMarkup +
        (pose.legsFront ? legs : "") + scarf + headMarkup + (farArmBehind ? "" : aL.markup) + aR.markup;
    }

    let fly = "";
    if (pose.kite === "fly") {
      // Hand position in world space: apply the upper-body lean about the hips.
      const a = rad(pose.lean), hx = aR.hand[0], hy = aR.hand[1] - hipY;
      const hand = [hx * Math.cos(a) - hy * Math.sin(a), hipY + hx * Math.sin(a) + hy * Math.cos(a)];
      const k = pose.kiteAt;
      fly =
        line(`M${pt(hand)} Q${n(hand[0] + 10)},${n((hand[1] + k[1]) / 2 + 40)} ${n(k[0] - 4)},${n(k[1] + 30)}`, 1.8) +
        kite(k[0], k[1], 44, 50, pose.kiteRot + 12);
    }

    const shadowScale = Math.max(0.45, 1 - pose.lift / 60);
    const shadow = `<ellipse cx="0" cy="9" rx="${n(58 * shadowScale)}" ry="${n(8 * shadowScale)}" fill="${C.ink}" opacity="0.28"/>`;

    return (
      shadow +
      `<g transform="translate(0 ${n(-pose.lift)})">` +
      (pose.legsFront && !back ? "" : legs) +
      `<g transform="${upperXf}">${upper}</g>` +
      fly +
      `</g>`
    );
  }

  const DEFS =
    `<svg width="0" height="0" style="position:absolute" aria-hidden="true" focusable="false"><defs>` +
    `<pattern id="kin-dots" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(28)">` +
    `<circle cx="3.5" cy="3.5" r="1.7" fill="${C.ink}" opacity="0.3"/></pattern></defs></svg>`;

  function svg(pose, opts) {
    const o = Object.assign({ box: [-150, -330, 300, 360], label: "Kin" }, opts || {});
    return `<svg viewBox="${o.box.join(" ")}" role="img" aria-label="${o.label}" xmlns="http://www.w3.org/2000/svg">${draw(pose)}</svg>`;
  }

  function headSvg(expr, turn, label) {
    const pose = merge(BASE, { expr, head: { rot: 0, turn: turn || 0 } });
    return `<svg viewBox="-80 -140 160 150" role="img" aria-label="${label || expr}" xmlns="http://www.w3.org/2000/svg">${head(pose)}</svg>`;
  }

  const POSES = {
    hero: { armL: { sh: -38, el: 103 }, armR: { sh: 38, el: -103 }, legL: { hip: -9 }, legR: { hip: 9 }, expr: "grin" },
    point: {
      lean: 5, head: { rot: 4, turn: 0.5 }, armL: { sh: -38, el: 103 }, armR: { sh: 98, el: 6, point: 1 },
      legL: { hip: -11 }, legR: { hip: 14 }, expr: "open", scarf: -0.3,
    },
    run: {
      lean: 16, lift: 16, head: { rot: -4, turn: 0.8 }, armL: { sh: 52, el: 72 }, armR: { sh: -56, el: 76 },
      legL: { hip: -38, knee: -70, foot: 1 }, legR: { hip: 58, knee: -78, foot: 1 }, expr: "open", scarf: -1,
    },
    fly: {
      lift: 6, lean: -4, head: { rot: -8, turn: 0.35 }, armL: { sh: -104, el: -62 }, armR: { sh: 162, el: 6 },
      legL: { hip: -14 }, legR: { hip: 10 }, expr: "joy", kite: "fly", scarf: 0.4,
    },
    squat: {
      drop: 50, head: { rot: 7, turn: 0.3 }, armL: { sh: -30, el: 40 }, armR: { sh: 30, el: -40 },
      legL: { hip: -72, knee: 142 }, legR: { hip: 72, knee: -142 }, legsFront: true, expr: "curious",
    },
  };

  global.Kin = { C, BASE, POSES, EXPR, DEFS, draw, svg, headSvg, merge, lerpPose, kite, charkhi, chappal, gamchaSwatch };
})(window);
