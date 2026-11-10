# Landing page: "Kin learns to move"

The page is a short film you scroll through. One character, one story, seven chapters. Each chapter demonstrates one engine feature by having the character discover it.

## Principles

- **Scroll is the playhead.** The page behaves like Kinetic's timeline: scrolling scrubs, scrolling back rewinds, nothing is a one-shot trigger that cannot be reversed.
- **The character carries the story.** No floating abstract shapes for decoration. Every object on screen is something Kin touches, pushes, rides or reacts to.
- **Show the code that made the motion.** Each chapter pairs the action with the three or four lines of Kinetic that would produce it.
- **Quiet page, loud character.** Paper-like background, one ink colour, one accent. The motion is the decoration.

## Kin

A small figure drawn in a single confident ink line: round head, bean body, noodle limbs, two dot eyes. No mouth; emotion comes from eyes, posture and timing.

Rig (inline SVG, one `<g>` per part, transform origins at joints):

```
kin
  shadow
  body            squash/stretch on scaleX/scaleY, origin at feet
    leg-l, leg-r  each a 2-segment path, IK-ish via pose presets
    arm-l, arm-r
    head
      eye-l, eye-r   scaleY for blink, translate for look-at
      brow-l, brow-r (appear only for surprise / effort)
  accent          scarf: the one coloured element, trails with a lag
```

Poses are plain objects of part transforms (`idle`, `notice`, `push`, `crouch`, `jump`, `land`, `juggle`, `reach`, `pull`, `cheer`). `poseTo(name, opts)` returns a GSAP timeline, so poses compose into chapter timelines. Limb paths morph between pose shapes by interpolating point lists with matching point counts.

Always-on idle layer, independent of scroll: breathing, blink every few seconds, scarf sway, eyes following the pointer within a small radius.

Animation principles to honour: anticipation before every big move, squash and stretch on impacts, follow-through on the scarf, arcs not straight lines, slow in and out.

## Structure

A fixed stage holds Kin. Chapters are tall scroll sections; each owns a GSAP timeline scrubbed by ScrollTrigger. Copy and code sit beside the stage and change per chapter.

A timeline bar is fixed to the bottom: a track with seven chapter markers, keyframe diamonds, and a playhead whose position is scroll progress. Clicking a marker scrolls to that chapter.

| # | Chapter | What happens | Engine idea | Code shown |
|---|---|---|---|---|
| 1 | Still | Blank paper. A pencil line draws Kin. Kin is frozen, then blinks and looks at the visitor. Headline writes on. | Scene and nodes | `s.add(...)` |
| 2 | First tween | A keyframe diamond drops in. Kin pokes it, gets carried from A to B on an eased slide, looks back amazed. Ghost frames show the easing spacing. | Tween and easing | `s.play(kin.move_to(..), 0.6, ease::out_cubic)` |
| 3 | Springs | Kin finds a coil, steps on it, launches, overshoots, settles. Two sliders (stiffness, damping) let the visitor retune the jump; a live curve plots the closed-form response. | Springs | `spring().stiffness(k).damping(c)` |
| 4 | Morph | Kin tosses a circle; mid-air it becomes a square, then a star. Kin catches the star and rides it across like a board. | Path morphing | `shape.morph_to(Star::new(5))` |
| 5 | Text | Letters of "Hello" rain down. Kin dodges, stacks them, shoves the stack and the glyphs flow into "World". | Text as paths | `title.text_to("World")` |
| 6 | Timeline | Kin notices the bar at the bottom, climbs down, grabs the playhead and drags it. The whole earlier scene replays backwards and forwards in a small inset. The visitor can drag it too. | Pure function of t | `timeline.evaluate(t)` |
| 7 | Export | Kin snaps a clapperboard. Frames peel off into a film strip that rolls into an MP4 card. Kin bows. Call to action. | Export | `kinetic render scene -o film.mp4` |

After the story: a compact section with install command, quickstart snippet, feature list, and links to GitHub and docs. Footer.

## Visual direction

- Background: warm off-white paper with faint grain. Dark theme: deep ink blue-black with chalk-white line.
- Ink: near-black. Accent: one saturated colour used for the scarf, keyframes, playhead and links.
- Type: one characterful display face for headlines, one clean mono for code, one readable sans for body. Self-hosted, subset.
- Line quality: consistent stroke width, round caps and joins, slight hand wobble via path variation (not filters).
- Decide exact tokens with the `impeccable` and `frontend-design` skills at build time and record them in `site/src/styles/tokens.css`.

## Technical notes

- Astro, static output, no client framework. One TypeScript module per chapter exporting `build(stage): gsap.core.Timeline`.
- GSAP core + ScrollTrigger. Animate only `transform` and `opacity`; path morphs set `d` on a handful of small paths.
- `prefers-reduced-motion`: no scrubbing or pinning. Each chapter shows a still illustration of its key pose with the same copy and code.
- Small screens: stage on top, copy below, shorter timelines, no pinning where it hurts scroll feel.
- Keyboard: chapter markers are buttons; sliders are native inputs; focus is visible.
- Budget: under 150 KB JS gzipped, no layout shift, 60 fps on a mid-range laptop.
- Base path configurable for GitHub Pages (`/kinetic-engine/`).

## Done when

- All seven chapters play forwards and backwards with scroll without visual glitches.
- Kin never teleports between chapters: each chapter's end pose is the next one's start pose.
- Reduced-motion and 375 px layouts are checked in a browser.
- Lighthouse performance and accessibility are at least 90.
- The page is live on GitHub Pages and linked from the README.
