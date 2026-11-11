// The journey. GSAP ScrollTrigger maps scroll position to a master time `t` in
// [0, CHAPTERS]; everything on stage is drawn from `t` alone, so scrolling back rewinds
// the street exactly. The only wall-clock inputs are scrub smoothing and Kin's blink, and
// neither changes what a given `t` looks like.

import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import "../../../design/kin/kin.js";
import { clamp, inOutCubic, lerp, linear, mix, outCubic, resample, rng, smooth, span, spring, track } from "./math.js";
import { CAM_MAX, GROUND, PARALLAX, brassShapes, buildWorld } from "./street.js";
import { wordAt } from "./glyphmorph.js";
import { CRANK, SADDLE_Y, rickshaw } from "./rickshaw.js";

const Kin = window.Kin;
const CHAPTERS = 7;
// The street sits above the bottom of the frame so captions have the road to themselves.
const RISE = 150;
const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

const journey = $("#journey");
const reduced = matchMedia("(prefers-reduced-motion: reduce)");

/* ---------- build the street ---------- */

const world = buildWorld();
$("#far").innerHTML = world.far;
$("#mid").innerHTML = world.mid;
$("#near-back").innerHTML = world.near;
$("#fore").innerHTML = world.fore;
$("#glow-near").innerHTML = world.glow;

const layers = ["far", "mid", "near", "fore"].map((name) => ({ el: $("#" + name), p: PARALLAX[name], items: [] }));
for (const layer of layers) {
  layer.items = $$(".cull", layer.el).map((el) => ({ el, a: +el.dataset.a, b: +el.dataset.b, on: true }));
}
const glowNear = $("#glow-near");
const shutters = $$(".shutter").map((el) => ({ el, cull: el.closest(".cull"), x: +el.dataset.x, ox: el.dataset.ox, oy: el.dataset.oy, last: -1 }));

/* ---------- Kin's path ---------- */

// Chapter two: Kin walks to the parked rickshaw, rides it down the lane, and steps off.
const RIDE = { mount: 0.86, go: 1.02, stop: 1.95, off: 2.06, from: 900, to: 2900 };
const saddleX = (t) => lerp(RIDE.from, RIDE.to, inOutCubic(span(t, RIDE.go, RIDE.stop)));
const riding = (t) => smooth(span(t, RIDE.go - 0.07, RIDE.go)) * (1 - smooth(span(t, RIDE.stop, RIDE.stop + 0.06)));

const stops = { brass: world.brassX - 270, board: world.boardX - 330, steps: 10050, roof: 10110 };
const kinKeys = [
  [0, 610], [RIDE.mount, 610], [RIDE.go, RIDE.from, smooth], [RIDE.stop, RIDE.to, inOutCubic], [RIDE.off, RIDE.to + 110, smooth], [3, 4500, smooth], [3.35, stops.brass, outCubic], [3.9, stops.brass],
  [4.3, stops.board, inOutCubic], [4.85, stops.board], [5.3, stops.steps, inOutCubic], [6.05, stops.steps],
  [6.3, stops.roof, smooth], [7, stops.roof],
];
const kinX = (t) => track(kinKeys, t);
// Kin climbs the mosque steps (six risers, drawn at 0.8 scale) on the way to the top.
const kinClimb = (x) => 105.6 * smooth(span(x, 9800, 9990));
const camExtra = [[0, 0], [4.9, 0], [5.4, 300], [6.1, 300], [6.5, 120], [7, 120]];

// When does Kin pass world position x? Used to trigger the swinging signs.
function timeAt(x, lo, hi) {
  for (let i = 0; i < 30; i++) {
    const m = (lo + hi) / 2;
    if (kinX(m) < x) lo = m; else hi = m;
  }
  return lo;
}
const swings = $$(".swing").map((el, i) => ({ el: el.firstElementChild, t0: timeAt(+el.dataset.x - 40, 2, 3), dir: i % 2 ? -1 : 1 }));

const base = (pose) => Kin.merge(Kin.BASE, pose);
const POSE = {
  idle: base(Kin.merge(Kin.POSES.hero, { head: { rot: -2, turn: 0.1 } })),
  look: base(Kin.merge(Kin.POSES.squat, { expr: "curious", head: { rot: 4, turn: 0.6 } })),
  point: base(Kin.merge(Kin.POSES.point, { head: { rot: 2, turn: 0.6 } })),
  watch: base(Kin.merge(Kin.POSES.squat, { expr: "wow", head: { rot: -12, turn: 0.55 } })),
  fly: base(Kin.POSES.fly),
};

function walk(phase, amp) {
  const s = Math.sin(phase), c = Math.cos(phase);
  return base({
    lean: 5 + 9 * amp, lift: 3 + 9 * amp * Math.abs(c), head: { rot: -2, turn: 0.78 }, scarf: -0.5 - 0.5 * amp,
    armL: { sh: 34 * s * amp, el: 24 + 40 * amp }, armR: { sh: -34 * s * amp, el: 24 + 40 * amp },
    legL: { hip: -30 * s * amp, knee: -(8 + 52 * amp * Math.max(0, c)), foot: 0.6 },
    legR: { hip: 30 * s * amp, knee: -(8 + 52 * amp * Math.max(0, -c)), foot: 0.6 },
    expr: amp > 1 ? "open" : "grin",
  });
}

// Two-bone leg: thigh and shin angles (rig convention) that put the foot on a target.
function legTo(hx, hy, tx, ty) {
  const LEN = 37;
  const d = clamp(Math.hypot(tx - hx, ty - hy), 8, LEN * 2 - 0.5);
  const toward = Math.atan2(tx - hx, ty - hy), bend = Math.acos(d / (LEN * 2));
  const deg = 180 / Math.PI;
  return { hip: (toward + bend) * deg, knee: -2 * bend * deg, foot: 0.4 };
}

const KIN_SCALE = 0.92;
function ridePose(crank) {
  const lift = -SADDLE_Y / KIN_SCALE - 74;
  const pedal = (side) => {
    const a = crank + (side < 0 ? Math.PI : 0);
    return legTo(side * 12, -74 - lift, (CRANK.x + Math.cos(a) * CRANK.r) / KIN_SCALE, (CRANK.y + Math.sin(a) * CRANK.r) / KIN_SCALE);
  };
  return base({
    lift, lean: 13, head: { rot: -7, turn: 0.8 }, scarf: -1, expr: "joy",
    armL: { sh: 70, el: 16 }, armR: { sh: 78, el: 10 }, legL: pedal(-1), legR: pedal(1),
  });
}

function kinPose(t, x) {
  const speed = Math.abs(kinX(t + 0.01) - kinX(t - 0.01)) / 0.02;
  const amp = clamp(speed / 1900, 0, 1.25);
  const still = t < 3.3 ? POSE.idle : t < 4.1 ? POSE.look : t < 5.05 ? POSE.point : t < 6.15 ? POSE.watch : POSE.fly;
  let pose = Kin.lerpPose(still, walk(x / 52, Math.max(amp, 0.25)), smooth(clamp(amp * 3)));
  // Chapter one opens on a held frame: Kin caught mid-stride, then released.
  if (t < 0.34) pose = Kin.lerpPose(walk(1.25, 1), pose, smooth(span(t, 0.14, 0.34)));
  const ride = riding(t);
  if (ride > 0) pose = Kin.lerpPose(pose, ridePose((saddleX(t) - RIDE.from) / 40), ride);
  if (t > 6.15) pose.kiteAt = [lerp(70, 150, smooth(span(t, 6.3, 7))), lerp(-320, -455, outCubic(span(t, 6.3, 6.9)))];
  return pose;
}

/* ---------- chapter props ---------- */

const ghosts = [];
let ghostMarkup = "";
for (let i = 0; i <= 10; i++) {
  const gx = lerp(RIDE.from, RIDE.to, inOutCubic(i / 10));
  ghosts.push(gx);
  ghostMarkup += `<path d="M0,-9 L9,0 L0,9 L-9,0Z" transform="translate(${gx.toFixed(1)} ${GROUND + 30})" fill="#e2533f"/>`;
}
$("#ghosts").innerHTML = ghostMarkup;
const ghostEls = $$("#ghosts path");

const brass = brassShapes();
const brassPts = brass.shapes.map((shape) => resample(shape, 72));
const brassKeys = [[3.3, 0], [3.5, 1, inOutCubic], [3.58, 1], [3.74, 2, inOutCubic], [3.82, 2], [3.98, 3, inOutCubic]];
const brassEl = [$("#brass"), $("#brass-shadow"), $("#brass-shine")];

const boardKeys = [[4.3, 0], [4.52, 1, linear], [4.58, 1], [4.8, 2, linear]];
const wordEls = [$("#word-ink"), $("#word-holes")];
let lastWord = -1;

const birdRng = rng(77);
const birds = [];
let birdMarkup = "";
for (let i = 0; i < 26; i++) {
  const step = world.steps[birdRng.int(0, world.steps.length - 1)];
  birds.push({
    x: birdRng.range(step.x0, step.x1), y: step.y - 5, dx: birdRng.range(-420, 520), dy: birdRng.range(260, 690),
    delay: birdRng.range(0, 0.26), flip: birdRng() < 0.5 ? -1 : 1,
  });
  birdMarkup += `<g class="bird"><path class="wing" fill="#7f8090"/><ellipse rx="10" ry="5.5" fill="#9a9aa6"/><circle cx="9" cy="-3" r="4" fill="#9a9aa6"/></g>`;
}
$("#birds").innerHTML = birdMarkup;
const birdEls = $$(".bird").map((el) => ({ g: el, wing: el.firstElementChild }));

// Paper flecks drifting across the frame. Positions depend on t only.
const dustRng = rng(5);
const dust = [];
let dustMarkup = "";
for (let i = 0; i < 34; i++) {
  dust.push({ x: dustRng.range(0, 1600), y: dustRng.range(40, 720), v: dustRng.range(260, 620), s: dustRng.range(0.5, 1.5), ph: dustRng.range(0, 6) });
  dustMarkup += `<path d="M-4,0 L0,-2.5 L4,0 L0,2.5Z" fill="${i % 5 ? "#efe7d6" : "#d8402f"}" opacity="${i % 5 ? 0.5 : 0.8}"/>`;
}
$("#dust").innerHTML = dustMarkup;
const dustEls = $$("#dust path");

// Flying things are drawn by Three.js when WebGL is available. Without it, the SVG
// pigeons and flecks above stand in.
// Three.js is loaded after first paint so the street appears without waiting for it.
let sky = null;
import("./sky3d.js")
  .then(({ createSky }) => {
    sky = createSky($("#sky3d"), world.steps);
    $("#birds").style.display = "none";
    $("#dust").style.display = "none";
    dirty = true;
  })
  .catch(() => ($("#sky3d").style.display = "none"));

/* ---------- light across the day ---------- */

const SKY_TOP = [[0, "#1b2140"], [1, "#39487a"], [1.6, "#7ea0c6"], [3, "#8cb6d8"], [4.5, "#a7bccb"], [5.4, "#c98860"], [6.2, "#4a3957"], [7, "#1c1f36"]];
const SKY_LOW = [[0, "#3a3f66"], [1, "#e6a886"], [1.6, "#f0d8b8"], [3, "#dde8ee"], [4.5, "#efdcb0"], [5.4, "#f1b47a"], [6.2, "#c8776a"], [7, "#3a3056"]];
const LIGHT = [[0, "#8a96cf"], [1, "#e6b6a0"], [1.7, "#fff2de"], [3, "#ffffff"], [4.5, "#ffe8bc"], [5.4, "#f4b48a"], [6.2, "#a67799"], [7, "#585d9a"]];
const NIGHT = [[0, 1], [0.9, 1], [1.5, 0], [5.6, 0], [6.3, 1], [7, 1]];

const el = {
  skyTop: $("#sky-top"), skyLow: $("#sky-low"), far: $("#far-shape"), sun: $("#sun"), moon: $("#moon"),
  light: $(".light"), glow: $("#glow"), kin: $("#kin"), rick: $("#rick"), stage: $(".stage"),
  caps: $$(".cap"), head: $("#playhead"), fill: $("#bar-fill"), marks: $$(".mark"),
  springPanel: $("#spring-panel"), render: $("#render"), renderFill: $("#render-fill"), renderPct: $("#render-pct"),
  box: $$(".letterbox"), brassName: $("#brass-name"), bar: $(".bar"),
};

/* ---------- interactive spring ---------- */

const springUi = { k: $("#stiffness"), c: $("#damping"), curve: $("#spring-curve"), code: $("#spring-code"), read: $("#spring-read") };
let sp = spring(180, 8);
let dirty = true;
function readSpring() {
  const k = +springUi.k.value, c = +springUi.c.value;
  sp = spring(k, c);
  let d = "";
  for (let i = 0; i <= 90; i++) {
    const time = (i / 90) * 1.6;
    d += `${i ? "L" : "M"}${(i / 90) * 240},${(96 - sp.step(time) * 60).toFixed(1)}`;
  }
  springUi.curve.setAttribute("d", d);
  springUi.code.textContent = `spring().stiffness(${k.toFixed(1)}).damping(${c.toFixed(1)})`;
  springUi.read.textContent = sp.zeta < 1 ? "underdamped: it overshoots, then settles" : sp.zeta > 1.02 ? "overdamped: it creeps in, no bounce" : "critically damped: fastest with no overshoot";
  dirty = true;
}
springUi.k.addEventListener("input", readSpring);
springUi.c.addEventListener("input", readSpring);
readSpring();

/* ---------- render one frame from t ---------- */

let lastSaddle = -1;

function window4(t, a, b, c, d) {
  return smooth(span(t, a, b)) * (1 - smooth(span(t, c, d)));
}

function render(t, blink) {
  const x = kinX(t);
  // Kin stands left of centre on wide screens and near the centre on narrow ones, where
  // only the middle of the 1600-unit frame is visible.
  const visible = Math.min(1600, (900 * innerWidth) / innerHeight);
  const narrow = clamp((1600 - visible) / 900);
  const cam = clamp(x - lerp(640, 790, narrow) + track(camExtra, t) * lerp(1, 0.5, narrow), 0, CAM_MAX);

  for (const layer of layers) {
    const off = cam * layer.p;
    layer.el.setAttribute("transform", `translate(${(-off).toFixed(1)} ${-RISE})`);
    for (const item of layer.items) {
      const on = item.b > off - 200 && item.a < off + 1800;
      if (on !== item.on) {
        item.el.style.display = on ? "" : "none";
        item.on = on;
      }
    }
  }
  glowNear.setAttribute("transform", `translate(${(-cam).toFixed(1)} ${-RISE})`);

  // Light.
  const low = track(SKY_LOW, t, mix), night = track(NIGHT, t);
  el.skyTop.setAttribute("stop-color", track(SKY_TOP, t, mix));
  el.skyLow.setAttribute("stop-color", low);
  el.far.setAttribute("fill", mix(low, "#37324d", 0.58));
  el.light.style.backgroundColor = track(LIGHT, t, mix);
  el.glow.style.opacity = night.toFixed(3);
  const day = span(t, 1, 6);
  el.sun.setAttribute("cx", lerp(240, 1380, day).toFixed(1));
  el.sun.setAttribute("cy", (600 - Math.sin(day * Math.PI) * 500).toFixed(1));
  el.sun.setAttribute("fill", mix("#f3a66a", "#fff3d6", Math.sin(day * Math.PI)));
  el.moon.setAttribute("opacity", (night * 0.9).toFixed(3));
  el.moon.setAttribute("cx", t < 3 ? 1240 : 330);

  // Shutters roll up as Kin reaches them.
  for (const s of shutters) {
    if (s.cull.style.display === "none") continue;
    const open = s.x < 900
      ? smooth(span(t, 1.04 + ((900 - s.x) / 900) * 0.2, 1.3 + ((900 - s.x) / 900) * 0.2))
      : smooth(span(x, s.x - 420, s.x - 60));
    if (open !== s.last) {
      s.el.setAttribute("transform", `translate(${s.ox} ${s.oy}) scale(1 ${(1 - open * 0.93).toFixed(3)})`);
      s.last = open;
    }
  }

  const ghostFade = 1 - smooth(span(t, 2.1, 2.4));
  ghostEls.forEach((g, k) => g.setAttribute("opacity", (smooth(span(x, ghosts[k] - 30, ghosts[k] + 30)) * (t > 1 ? ghostFade : 0)).toFixed(3)));

  // Signboards: a push as Kin passes, then the closed-form spring does the rest.
  for (const s of swings) {
    const a = t > s.t0 ? 19 * s.dir * sp.impulse((t - s.t0) * 4.2) : 0;
    s.el.setAttribute("transform", `rotate(${a.toFixed(2)})`);
  }

  // Brass: thali to lota to ghanti to diya.
  const m = track(brassKeys, t);
  const i = Math.min(2, Math.floor(m)), u = m - i;
  let d = "", shine = "";
  for (let k = 0; k < 72; k++) {
    const px = lerp(brassPts[i][k][0], brassPts[i + 1][k][0], u), py = lerp(brassPts[i][k][1], brassPts[i + 1][k][1], u);
    d += `${k ? "L" : "M"}${px.toFixed(1)},${py.toFixed(1)}`;
    if (k >= 5 && k <= 17) shine += `${k > 5 ? "L" : "M"}${(px * 0.74).toFixed(1)},${(py * 0.74).toFixed(1)}`;
  }
  brassEl[0].setAttribute("d", d + "Z");
  brassEl[1].setAttribute("d", d + "Z");
  brassEl[2].setAttribute("d", shine);
  el.brassName.textContent = brass.names[Math.round(m)];

  // Signboard: the shaped outlines of one word morph into the next.
  const b = track(boardKeys, t);
  if (b !== lastWord) {
    const word = wordAt(b);
    wordEls[0].setAttribute("d", word.ink);
    wordEls[1].setAttribute("d", word.holes);
    lastWord = b;
  }

  // Pigeons: each one's flight is a function of t, so they land where they left.
  if (!sky) birds.forEach((bird, k) => {
    const e = outCubic(span(t, 5.42 + bird.delay, 5.98 + bird.delay));
    const bx = bird.x + bird.dx * e, by = bird.y - bird.dy * e - Math.sin(e * Math.PI) * 50;
    const flap = e > 0 && e < 1 ? Math.sin(e * 46 + k) : e >= 1 ? Math.sin(k * 2.1 + e) : 0.22;
    birdEls[k].g.setAttribute("transform", `translate(${bx.toFixed(1)} ${by.toFixed(1)}) scale(${bird.flip} 1)`);
    birdEls[k].wing.setAttribute("d", `M-8,0 L-1,${(-17 * flap).toFixed(1)} L8,0Z`);
  });

  if (sky) sky.render(t, cam, RISE);
  else dust.forEach((f, k) => {
    const fx = ((((f.x - t * f.v) % 1700) + 1700) % 1700) - 50;
    const fy = f.y + Math.sin(t * 2.3 + f.ph) * 46;
    dustEls[k].setAttribute("transform", `translate(${fx.toFixed(1)} ${fy.toFixed(1)}) rotate(${(t * 140 * f.s + f.ph * 60).toFixed(0)}) scale(${f.s})`);
  });

  // Rickshaw: parked, ridden, parked again.
  const sx = saddleX(t);
  el.rick.setAttribute("transform", `translate(${sx.toFixed(1)} ${GROUND})`);
  if (sx !== lastSaddle) {
    el.rick.innerHTML = rickshaw(((sx - RIDE.from) / 60) * (180 / Math.PI), (sx - RIDE.from) / 40);
    lastSaddle = sx;
  }

  // Kin.
  const pose = kinPose(t, x);
  pose.blink = blink;
  el.kin.setAttribute("transform", `translate(${x.toFixed(1)} ${(GROUND - kinClimb(x)).toFixed(1)}) scale(${KIN_SCALE})`);
  el.kin.innerHTML = Kin.draw(pose);

  // Captions and panels.
  el.caps.forEach((cap, k) => {
    const o = k === 0 ? 1 - smooth(span(t, 0.8, 0.97)) : window4(t, k + 0.06, k + 0.22, k + 0.84, k + 0.97) + (k === CHAPTERS - 1 ? smooth(span(t, k + 0.84, k + 0.97)) : 0);
    cap.style.opacity = o.toFixed(3);
    cap.style.transform = `translateY(${((1 - o) * 14).toFixed(1)}px)`;
    cap.style.visibility = o < 0.01 ? "hidden" : "visible";
  });
  const sp3 = window4(t, 2.1, 2.24, 2.86, 2.97);
  el.springPanel.style.opacity = sp3.toFixed(3);
  el.springPanel.style.visibility = sp3 < 0.01 ? "hidden" : "visible";

  const film = smooth(span(t, 6.2, 6.5));
  el.box.forEach((bar) => (bar.style.transform = `scaleY(${film.toFixed(3)})`));
  const done = span(t, 6.42, 6.9);
  el.render.style.opacity = smooth(span(t, 6.34, 6.46)).toFixed(3);
  el.renderFill.style.transform = `scaleX(${done.toFixed(3)})`;
  el.renderPct.textContent = done >= 1 ? "film.mp4 written" : `frame ${Math.round(done * 420)} of 420`;

  // Timeline bar.
  const p = t / CHAPTERS;
  el.head.style.left = `${(p * 100).toFixed(2)}%`;
  el.head.setAttribute("aria-valuenow", t.toFixed(2));
  el.fill.style.transform = `scaleX(${p.toFixed(4)})`;
  const now = Math.min(CHAPTERS - 1, Math.floor(t));
  el.marks.forEach((mark, k) => mark.setAttribute("aria-current", k === now ? "step" : "false"));
}

/* ---------- scroll to time ---------- */

function scrollRange() {
  return Math.max(1, journey.offsetHeight - innerHeight);
}
function targetTime() {
  return clamp((scrollY - journey.offsetTop) / scrollRange()) * CHAPTERS;
}
function seek(progress, smoothScroll) {
  scrollTo({ top: journey.offsetTop + clamp(progress) * scrollRange(), behavior: smoothScroll && !reduced.matches ? "smooth" : "auto" });
}

// GSAP's ScrollTrigger scrubs one number, clock.t, from the scroll position. The ticker
// redraws the stage whenever that number (or Kin's blink) changes.
gsap.registerPlugin(ScrollTrigger);
const clock = { t: targetTime() };
const range = { trigger: journey, start: "top top", end: "bottom bottom" };
gsap.matchMedia().add({ motion: "(prefers-reduced-motion: no-preference)", still: "(prefers-reduced-motion: reduce)" }, (context) => {
  if (context.conditions.motion) {
    gsap.fromTo(clock, { t: 0 }, { t: CHAPTERS, ease: "none", scrollTrigger: { ...range, scrub: 0.6 } });
  } else {
    // No scrubbing: hold each chapter's key frame and cut between them.
    ScrollTrigger.create({
      ...range,
      onUpdate: (self) => (clock.t = Math.min(CHAPTERS - 0.35, Math.floor(Math.min(self.progress * CHAPTERS, CHAPTERS - 0.01)) + 0.65)),
    });
    clock.t = 0.65;
  }
});

// "Let it play": a tween scrolls the page at a steady pace until the visitor takes over.
const autoBtn = $("#auto");
let autoTween = null;
function setAuto(on) {
  autoTween?.kill();
  autoTween = null;
  if (on && !reduced.matches) {
    const from = { y: scrollY }, end = journey.offsetTop + scrollRange();
    autoTween = gsap.to(from, {
      y: end, ease: "none", duration: Math.max(0.5, ((end - scrollY) / scrollRange()) * 75),
      onUpdate: () => scrollTo(0, from.y), onComplete: () => setAuto(false),
    });
  }
  autoBtn.setAttribute("aria-pressed", String(Boolean(autoTween)));
  autoBtn.textContent = autoTween ? "Pause" : "Let it play";
}
autoBtn.addEventListener("click", () => setAuto(!autoTween));
for (const type of ["wheel", "touchstart", "keydown", "pointerdown"]) {
  addEventListener(type, (event) => {
    if (autoTween && !(event.target instanceof Element && event.target.closest("#auto"))) setAuto(false);
  }, { passive: true });
}

let current = clock.t;
let lastT = -1, lastBlink = -1;
gsap.ticker.add((time) => {
  current = clock.t;
  const blink = !reduced.matches && current > 0.34 && (time * 1000) % 4300 < 130 ? 1 : 0;
  if (current !== lastT || blink !== lastBlink || dirty) {
    render(current, blink);
    lastT = current;
    lastBlink = blink;
    dirty = false;
  }
  el.bar.classList.toggle("is-off", scrollY > journey.offsetTop + scrollRange() + innerHeight * 0.4);
});

/* ---------- timeline bar: click a chapter, or drag the playhead ---------- */

el.marks.forEach((mark, k) => mark.addEventListener("click", () => seek((k + 0.62) / CHAPTERS, true)));

const trackEl = $("#bar-track");
let dragging = false;
const dragTo = (event) => {
  const box = trackEl.getBoundingClientRect();
  seek((event.clientX - box.left) / box.width, false);
};
trackEl.addEventListener("pointerdown", (event) => {
  if (event.target.closest(".mark")) return;
  dragging = true;
  trackEl.setPointerCapture(event.pointerId);
  dragTo(event);
});
trackEl.addEventListener("pointermove", (event) => dragging && dragTo(event));
trackEl.addEventListener("pointerup", () => (dragging = false));
trackEl.addEventListener("pointercancel", () => (dragging = false));
el.head.addEventListener("keydown", (event) => {
  const step = event.key === "ArrowRight" ? 1 : event.key === "ArrowLeft" ? -1 : 0;
  if (!step) return;
  event.preventDefault();
  seek(current / CHAPTERS + step * 0.02, false);
});
reduced.addEventListener("change", () => (dirty = true));
addEventListener("resize", () => {
  sky?.resize();
  dirty = true;
});
