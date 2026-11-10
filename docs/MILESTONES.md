# Milestones

A milestone is done when every box is ticked, `verify` passes, and the tag is pushed. Work top to bottom; each box is roughly one to three commits.

## M0 — Bootstrap (`v0.0.0`)

- [x] Cargo workspace with seven crate stubs
- [x] Licenses, README, line-ending rules
- [x] `CLAUDE.md` and `docs/`
- [x] `.claude/` settings, hooks, skills, agents
- [x] CI: fmt, clippy, test on push
- [x] Setup scripts (plugins, tool checks)
- [x] CI green on GitHub

## M1 — First pixels (`v0.1.0`)

- [ ] `kinetic-core`: `LinearRgba`, sRGB conversion, colour constants
- [ ] `kinetic-core`: minimal `Lerp` trait for scalar, vec2, colour
- [ ] `kinetic-core`: `ease::linear`, `in_out_cubic`, `out_back`
- [ ] `kinetic-vector`: circle and rect to path, lyon fill tessellation to a vertex/index mesh
- [ ] `kinetic-render`: wgpu device setup, offscreen target, MSAA, solid-colour pipeline
- [ ] `kinetic-render`: render a mesh list to RGBA bytes (headless)
- [ ] Golden test harness: software adapter, tolerance compare, `cargo xtask bless`
- [ ] First golden: circle and rect on a dark background
- [ ] `kinetic-editor`: winit window presenting the renderer output
- [ ] `kinetic-editor`: egui overlay with a time slider and play/pause
- [ ] One hard-coded tween (circle position, eased) driven by the slider
- [ ] `kinetic preview` opens the window

## M2 — Timeline and animation (`v0.2.0`)

- [ ] `Value`, `PropertyId`, `NodeId`, `NodeDef`
- [ ] `Track`, `Segment`, `Curve`; `Timeline::evaluate(t) -> FrameState`
- [ ] `Scene` builder: `add`, cursor, `play`, `wait`
- [ ] `Handle<T>` and actions: `move_to`, `scale_to`, `rotate_to`, `fade_to`, `fill_to`
- [ ] `parallel`, `sequence`, `stagger`
- [ ] Full easing set (sine, quad, cubic, quart, expo, back, elastic, bounce; in, out, in-out)
- [ ] `CubicBezier` easing
- [ ] Closed-form spring: three damping regimes, initial velocity, settle time
- [ ] Property tests: endpoints, continuity, evaluate purity and order independence
- [ ] `serde` round trip for `Timeline`
- [ ] Editor previews a `Timeline` built from an in-process scene function
- [ ] `scenes/hello` example

## M3 — Hot reload (`v0.3.0`)

- [ ] `kinetic_scene!` macro and C entry point returning serialised timeline bytes
- [ ] Wire-format version check
- [ ] Loader: copy library to temp, `libloading`, deserialise, unload safely
- [ ] File watcher with debounce, background `cargo build`
- [ ] Build error overlay; previous timeline keeps playing
- [ ] Playhead and play state preserved across reload
- [ ] `kinetic preview scenes/hello` end to end; reload under about two seconds
- [ ] `kinetic new <name>` scaffolds a scene crate

## M4 — Paths and morphing (`v0.4.0`)

- [ ] `PathData`, shape lowering for polygon, star, line, rounded rect, custom bezier
- [ ] Stroke tessellation: width, caps, joins
- [ ] `trim` property (draw-on effect by arc length)
- [ ] Arc-length resampling
- [ ] Winding normalisation and start-point alignment
- [ ] Subpath pairing; grow from / shrink to point
- [ ] `morph_to` action with prepared pairs stored at build time
- [ ] Mesh cache keyed by geometry hash
- [ ] Goldens: circle to rect, star to blob, ring to disc at t = 0, 0.25, 0.5, 0.75, 1
- [ ] `scenes/morph` example

## M5 — Text (`v0.5.0`)

- [ ] Bundled font (OFL) and font database
- [ ] Shaping and layout via cosmic-text; alignment, size, line height
- [ ] Glyph outlines to paths; text node as path group
- [ ] Per-glyph handles for stagger animations
- [ ] `text_to` glyph morph with surplus glyph handling
- [ ] AA quality check at 1x and 2x against a reference rasteriser
- [ ] Goldens: static paragraph, "Hello" to "World" midpoints
- [ ] `scenes/text` example

## M6 — Export (`v0.6.0`)

- [ ] Texture readback with row padding handled
- [ ] Frame iterator: fixed timestep from frame index
- [ ] ffmpeg child process, stdin pipe, error capture
- [ ] `kinetic render` with `--fps`, `--size`, `--crf`, `--range`
- [ ] PNG sequence output
- [ ] `kinetic doctor` (ffmpeg, GPU adapter, toolchain)
- [ ] Determinism test: frame hashes equal across two runs
- [ ] Progress bar and ETA

## M7 — Editor (`v0.7.0`)

- [ ] Layout: viewport, timeline panel, inspector panel
- [ ] Timeline track view with segments, zoom and pan
- [ ] Playback: play, pause, loop, speed, frame step, keyboard shortcuts
- [ ] Inspector: selected node's properties at the playhead
- [ ] Click to select in viewport; outline of selection
- [ ] Viewport pan and zoom, fit to frame, safe-area guides
- [ ] Export dialog calling `kinetic-export`
- [ ] Persist window layout and last scene

## M8 — Effects and scene features (`v0.8.0`)

- [ ] Groups with inherited transform and opacity
- [ ] Image nodes (PNG, JPEG) with texture cache
- [ ] Camera: pan, zoom, rotate as animatable properties
- [ ] Post stack infrastructure (ping-pong targets)
- [ ] Gaussian blur, glow
- [ ] Motion blur by sub-frame accumulation
- [ ] Gradients (linear, radial) as fills
- [ ] Effect goldens
- [ ] 1080p60 preview holds 60 fps on example scenes (measure, then optimise)

## M9 — Landing page (`v0.9.0`)

See `LANDING.md`.

- [ ] Astro project in `site/`, tokens, typography, base layout
- [ ] Kin SVG rig: parts, pivots, pose system, idle loop
- [ ] Scroll timeline bar with chapter markers
- [ ] Chapter 1 Still
- [ ] Chapter 2 First tween
- [ ] Chapter 3 Springs (interactive)
- [ ] Chapter 4 Morph
- [ ] Chapter 5 Text
- [ ] Chapter 6 Timeline (draggable playhead)
- [ ] Chapter 7 Export and CTA
- [ ] Reduced-motion and mobile paths
- [ ] Accessibility and performance pass (Lighthouse >= 90)
- [ ] GitHub Pages deploy workflow

## M10 — The film and 1.0 (`v1.0.0`)

- [ ] Kin rig as a Kinetic scene (paths and groups)
- [ ] `scenes/kin-film`: a 30 to 45 second short
- [ ] Render and embed the film on the landing page
- [ ] README with GIFs, quickstart, API tour
- [ ] Examples gallery
- [ ] API docs pass (`cargo doc` clean, examples compile)
- [ ] Release notes, `v1.0.0` tag and GitHub release
