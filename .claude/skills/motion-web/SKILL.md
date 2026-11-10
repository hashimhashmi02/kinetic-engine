---
name: motion-web
description: Reference for building Kinetic's landing page - a scroll-driven Three.js journey through Old Delhi with the character Kin. Covers master timeline, camera rail, procedural street kit, character animation, lighting script, fallbacks and performance. Use when working in site/ or on anything in docs/LANDING.md.
---

# Motion on the web: Kin in Purani Dilli

Read `docs/LANDING.md` first; it is the spec. Load the `impeccable` and `frontend-design` skills for visual decisions. This file covers how the motion is built.

Check the installed versions of `three`, `gsap` and `astro` in `site/package.json` and read their current docs before writing against them. Do not write API calls from memory.

## One rule: everything is a function of `t`

```ts
interface Animated { update(t: number): void }   // t = master time in seconds
```

- ScrollTrigger maps scroll progress to `t` (with a little scrub smoothing). A single render loop calls `update(t)` on every active object, then renders.
- No object keeps state that depends on the previous frame. Things that need lag (scarf, swinging signs after a pass) compute it from `t` and `t - dt`, or from a closed-form response to a trigger time.
- Test for it: jump `t` to random values in random order and compare screenshots against the same `t` reached by smooth scrolling. They must match.
- Idle life (breathing, flicker, pointer parallax) runs on wall-clock time on separate properties and is switched off for reduced motion.

## Scroll and chapters

- `timeline.ts` lists chapters `{ id, start, end, scrollWeight }`. Scroll height per chapter comes from `scrollWeight`, so pacing is tuned in one place.
- One pinned full-viewport canvas; one tall spacer element drives ScrollTrigger. Captions are DOM elements whose opacity and offset are also functions of `t`.
- The timeline bar reads the same `t`. Chapter markers are buttons that scroll to a chapter start.
- Build inside `gsap.matchMedia()` so desktop, phone and reduced-motion setups are separate and cleanly reverted.

## Camera rail

- Two Catmull-Rom curves: position and look-at target. Sample by a per-chapter eased parameter so the camera slows into stops and never moves at constant speed through a turn.
- Keep the look-at target ahead of Kin, not on Kin, so the character leads the frame.
- Pointer parallax: a few centimetres of offset and a degree or two of rotation, low-pass filtered. It is added after the rail and never changes `t`.
- Author the rail with a debug view (top-down, curve handles visible) toggled by a query parameter.

## Street kit

- Modules: ground strip, building bays of a few widths and heights, shutter, balcony, jharokha, signboard, pole, awning, stall, crate stack. Each is a low-poly mesh with vertex colours.
- A seeded generator places modules along the route. Same seed, same street, every load.
- Repeats use `InstancedMesh`. Aim for tens of draw calls per chapter, not hundreds.
- Cables: catenary curves between anchors, drawn as thin tubes or lines; sag varies with the seed. They are the signature of the place, so give them real density.
- Signboard text: render to a canvas texture at build or load time using fonts with correct shaping for Devanagari and Urdu. Never hand-place glyphs.
- Stream by distance: add a chapter's group when the camera is one chapter away, dispose geometry and textures when two away.

## Kin

- Simple hierarchy or a small skeleton. Actions are either short authored clips sampled at a time derived from `t`, or procedural poses.
- Blend actions with weights that are functions of `t`. Cross-fade over fixed windows defined in the chapter data.
- Feet: for walking and pedalling, drive the cycle phase from distance travelled along the path, so feet do not slide when the easing changes speed.
- Squash and stretch on the root, preserving volume. Eyes are separate meshes: blink by scale, look by small rotation.

## Character animation principles (apply every time)

1. Anticipation before every big action.
2. Squash and stretch on launches and landings, held for only a few frames.
3. Follow-through and overlap: head, arms and scarf arrive late and overshoot.
4. Arcs, not straight lines.
5. Timing gives weight: fast out, slow settle.
6. Eyes lead, then head, then body.
7. Holds: let a pose read before the next action.

## Light script

- Tracks on the master timeline: sun direction, sun colour, sky gradient stops, fog colour and density, lamp emissive strength, exposure.
- Fog does most of the depth work; match fog colour to the horizon colour at every `t`.
- Bloom only from dusk onward and only on capable devices. Keep tone mapping constant across chapters so colours do not shift at boundaries.

## Springs on the page

Port the closed-form spring from `kinetic-core` (see the `animation-math` skill) to TypeScript and use it for swinging signs and the interactive dials. Add a unit test that compares the port with reference values exported from the Rust tests.

## Fallbacks

- Detect WebGL support, `prefers-reduced-motion`, and low device capability (memory, core count, a short frame-time probe).
- Fallback path: one pre-rendered still per chapter with the same captions and code, normal document scroll, no pinning.
- Phones on the 3D path: pixel ratio capped lower, fewer instances, no post-processing, shorter scroll distance.

## Performance

- Cap pixel ratio at 2. Render only when `t`, the pointer or an idle animation changed.
- Merge static geometry per chapter; share materials; compress textures; Draco-compress glTF.
- Pause the loop when the tab is hidden or the canvas is off screen.
- Watch draw calls, triangles and texture memory with the renderer's info counters in the debug view.

## Verifying

Use the browser pane: scroll forwards and backwards through every chapter, drag the timeline bar, then repeat at 375 px wide, with reduced motion emulated, and with WebGL disabled. Look for pops at chapter boundaries, feet sliding, anything that fails to rewind, and captions that are hard to read over the scene.
