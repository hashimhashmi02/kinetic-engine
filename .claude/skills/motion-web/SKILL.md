---
name: motion-web
description: Reference for building Kinetic's landing page - the Kin SVG character rig, pose system, GSAP ScrollTrigger scrubbing, character animation principles, reduced motion and performance. Use when working in site/ or on anything in docs/LANDING.md.
---

# Motion on the web: Kin's page

Read `docs/LANDING.md` first; it is the spec. Load the `impeccable` and `frontend-design` skills for visual decisions. This file covers how the motion is built.

## Rig

- One inline SVG. Each body part is a `<g data-part="...">`; children are positioned so the group's local origin is the joint. Set `transform-box: fill-box` and an explicit `transform-origin`, or use GSAP's `svgOrigin`, and check it in Firefox and Safari as well as Chromium.
- Limbs are single stroked paths with a fixed number of points per limb. A pose stores each limb's point list, so limbs bend by interpolating points; no skeleton solver is needed.
- Squash and stretch on the body group, origin at the feet, volume roughly preserved: `scaleX = 1 / sqrt(scaleY)`.
- Eyes: blink is `scaleY` to 0.1 and back in about 120 ms; look-at is a clamped translate.
- The scarf is a short chain of points following the neck with a lag (lerp towards the previous point each tick). It runs on the idle ticker, not the scroll timeline.

## Poses

```ts
type Pose = { body: Xf; head: Xf; armL: Pt[]; armR: Pt[]; legL: Pt[]; legR: Pt[]; eyes?: EyeState };
function poseTo(rig: Rig, pose: Pose, vars?: gsap.TweenVars): gsap.core.Timeline;
```

- Poses are data in one file. Chapters compose `poseTo` calls; they never set part transforms directly.
- A chapter's last pose and position equal the next chapter's first. Assert this in a dev-only check.

## Scroll scrubbing

- One master approach: each chapter section gets `ScrollTrigger.create({ trigger, start: "top top", end: "+=N%", pin: stage, scrub: 0.5, animation: timeline })`. A little scrub smoothing hides wheel steps.
- Timelines must be fully reversible: use `fromTo` or explicit start states, no `onComplete` side effects that are not undone in `onReverseComplete`. Prefer state derived from timeline progress over callbacks.
- The bottom timeline bar reads total scroll progress from a single ScrollTrigger over the whole story and sets a CSS variable.
- Two layers of time: scroll-driven (story) and clock-driven (idle). Idle tweens touch different properties or wrapper groups so they never fight the scrubbed ones.
- Call `ScrollTrigger.refresh()` after fonts load and on resize; build inside `gsap.matchMedia()` so breakpoints and reduced motion get separate, cleanly reverted setups.

## Character animation principles (apply every time)

1. Anticipation: a small move the opposite way before any big action (crouch before jump, lean back before push).
2. Squash and stretch on launches and landings, held for only a few frames.
3. Follow-through and overlap: head, arms and scarf arrive after the body and overshoot slightly.
4. Arcs: move hands and the body along curves; use a motion path or separate x and y easings.
5. Timing gives weight: fast out, slow settle. Avoid uniform durations across parts.
6. Eyes lead: the eyes move a beat before the head, the head before the body.
7. Holds: let a pose read for a moment before the next action.

## Reduced motion and small screens

- `prefers-reduced-motion: reduce`: no pinning, no scrubbing, no idle loop. Each chapter renders its key pose as a still with copy and code. Interactive controls still work but apply changes instantly.
- Under about 768 px: stage on top, copy below, shorter pin distances, or unpinned reveal timelines if pinning feels sticky on touch.

## Performance

- Animate `transform` and `opacity`. Path `d` updates are fine for a few small paths per frame; avoid animating filters, large blurs or stroke-dasharray on long paths every frame.
- `will-change: transform` only on parts that move during the active chapter.
- Pause the idle ticker when the tab is hidden or the stage is off screen.
- Ship GSAP core and ScrollTrigger only. Check the bundle with `pnpm --dir site build`.

## Verifying

Use the browser pane: scroll forwards and backwards through every chapter, then at 375 px wide, then with reduced motion emulated. Look for teleports at chapter boundaries, limbs detaching at joints, and anything that fails to reverse.
