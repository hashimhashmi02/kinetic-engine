# Architecture

## Crates

```
kinetic-core      scene graph, properties, timeline, easing, springs,
                  paths and morphs, text shaping with a bundled font     (pure)
   ^
kinetic-vector    kurbo paths, lyon tessellation, resampling, morph
   ^
kinetic-text      empty for now: shaping moved into core (see Decisions)

kinetic-render    wgpu renderer, MSAA, post stack        (core, vector, text)
kinetic-export    offscreen frames to ffmpeg             (render)
kinetic-reload    cargo builds, scene library loading, file watching   (core)
kinetic-editor    winit + egui preview                   (render, reload)
kinetic-cli       preview / render / new / doctor        (editor, export)
```

Arrows point from dependent to dependency. No reverse edges.

## The frame pipeline

```
scene fn  --build-->  Timeline  --evaluate(t)-->  FrameState  --tessellate-->  DrawList  --wgpu-->  pixels
(user code, once)     (data)                      (data)                       (meshes)
```

1. **Build.** The user's `fn scene(s: &mut Scene)` runs once. Each `s.add` creates a node with initial property values. Each `s.play` and friends append segments to per-property tracks and advance a cursor.
2. **Evaluate.** `Timeline::evaluate(t)` walks each track, finds the segment containing `t`, and interpolates. The result is a `FrameState`: a flat, ordered list of nodes with resolved properties. This step is pure.
3. **Tessellate.** Each node's path (after morphing) is filled and stroked into triangles by lyon. Meshes are cached by a hash of the geometry-affecting properties, so a node that only moves is not re-tessellated.
4. **Draw.** One pass into a 4x MSAA target, resolve, then the post stack, then present or read back.

## Core data model

```rust
pub struct Timeline {
    pub duration: f64,
    pub nodes: Vec<NodeDef>,          // creation order = default z order
    pub tracks: Vec<Track>,           // sorted by (node, property)
}

pub struct Track {
    pub node: NodeId,
    pub property: PropertyId,
    pub initial: Value,
    pub segments: Vec<Segment>,       // non-overlapping, sorted by start
}

pub struct Segment {
    pub start: f64,
    pub duration: f64,
    pub from: Value,                  // resolved at build time
    pub to: Value,
    pub curve: Curve,                 // Easing(..) | Spring { .. }
}

pub enum Value { Scalar(f64), Vec2(DVec2), Color(LinearRgba), Path(PathData), Text(TextData) }
```

Rules that keep evaluation pure:

- `from` is resolved when the segment is created (the track's value at that moment), so a segment never depends on evaluating an earlier one.
- Before the first segment the track holds `initial`; between segments and after the last it holds the previous segment's `to`.
- Tracks and nodes live in `Vec`s with stable order. No hash map iteration.
- `Timeline` is `serde`-serialisable. That is the hot reload wire format and the basis of core tests.

## Scene API

```rust
let circle: Handle<Circle> = s.add(Circle::new(80.0).fill(palette::blue()));
s.play(circle.scale_to(1.5), (0.6, Ease::OutBack));
s.parallel([a.move_to(p), b.fade_to(0.0)], 1.0);
s.stagger(items.iter().map(|i| i.fade_to(1.0)), 0.4, 0.05); // timing, then delay
s.play(circle.move_to((300.0, 0.0)), spring().stiffness(180.0).damping(12.0));
s.wait(0.5);
```

- `Handle<T>` is a typed, `Copy` node id. Methods on it return an `Action` (target property plus destination value); they do not mutate anything.
- `play` takes an action and a *timing*: either a duration with an optional easing, or a spring (whose duration is its settle time). `impl Into<Timing>` covers `0.6`, `(0.6, ease::out_back)` and `spring()`.
- `parallel` starts all actions at the cursor and advances by the longest. `sequence` chains. `stagger` offsets starts by a fixed delay.

## Easing and springs

Easings are plain `fn(f64) -> f64` on `[0, 1]`, plus `CubicBezier(x1, y1, x2, y2)` solved by Newton iteration with a bisection fallback.

A spring from 0 to 1 with mass `m`, stiffness `k`, damping `c` and initial velocity `v0` has, with `w0 = sqrt(k/m)` and `z = c / (2 sqrt(k m))`:

- **Underdamped (z < 1):** `wd = w0 sqrt(1 - z^2)`;
  `x(t) = 1 - e^(-z w0 t) (cos(wd t) + ((z w0 - v0) / wd) sin(wd t))`
- **Critically damped (z = 1):** `x(t) = 1 - e^(-w0 t) (1 + (w0 - v0) t)`
- **Overdamped (z > 1):** with `r1,2 = -w0 (z -+ sqrt(z^2 - 1))`,
  `x(t) = 1 + (r2 + v0) / (r1 - r2) * e^(r1 t) - (r1 + v0) / (r1 - r2) * e^(r2 t)`

Settle time is the smallest `t` after which the envelope `e^(-z w0 t)` (or the slow exponential when overdamped) stays under a tolerance (default 0.001). That gives a spring a definite duration so it composes with everything else. At the end of the segment the value snaps to exactly `to`.

## Vectors and morphing

- `PathData` (move, line, quad, cubic, close) lives in `kinetic-core`, so scenes and morphs need no geometry crate; `kinetic-vector` converts it to `kurbo::BezPath` for tessellation. Shapes (`Circle`, `Rect`, `RoundedRect`, `Polygon`, `Star`, `Line`, `PathShape`) all lower to a path. Every closed built-in shape winds counter-clockwise and starts at its top.
- **The shape is a property.** `PropertyId::Shape` carries a `Value::Shape`, so a morph is an ordinary segment on an ordinary track: interrupting, chaining, easing and springs work as for any other property.
- **Morph** between shapes A and B, done once in `Track::push` (`kinetic_core::morph::prepare`):
  1. Flatten each shape to contours (Wang's formula, fixed tolerance). An open contour is walked out and back so it can be treated as closed. Contours of zero length are dropped.
  2. Pair contours largest first by a cost of centroid distance plus size difference, preferring partners of the same winding. An unpaired contour grows from, or shrinks to, a point at its own centroid, so a hole closes where it is.
  3. Resample each pair by arc length to the same point count (from the longer perimeter, every 2 units, clamped to 32..1024).
  4. Make the winding equal, then rotate B's start index to minimise the sum of squared distances.
  5. Store both as `Shape::Outline` of the same structure. Evaluating is a point-wise lerp, which also extrapolates for springs that overshoot.
- **Strokes.** Outline colour, width, cap, join and `trim` are node properties. lyon tessellates strokes with a miter limit of 4. The width grows with the node's scale by `sqrt(|sx * sy|)`. Fill is drawn first, outline on top; a `Line` has no fill.
- **Trim** cuts the transformed path to `[start, end]` of its total arc length, subpaths in order, cutting curves exactly with kurbo's `inv_arclen`. Whole subpaths inside the range stay closed.
- **Mesh cache.** `MeshCache` (in `kinetic-render`) keys each fill or outline mesh by a hash of shape, scale, rotation, stroke style, trim and tolerance, confirms hits by comparing the full key, and drops meshes the last frame did not use. A node that only moves, fades or changes colour is not re-tessellated. Output is identical to `Frame::from_state`.

## Text

- **Fonts.** `kinetic_core::text` bundles Poppins Regular and Bold (SIL OFL 1.1, `crates/kinetic-core/fonts`). A `Font` is `'static` bytes plus a face index, so a scene can bring its own with `include_bytes!`; `FontBook` picks a face by family and closest weight. No font is ever read from disk, so every machine sets the same glyphs.
- **Shaping and layout.** rustybuzz shapes each line (kerning, ligatures, Devanagari conjuncts and marks). Lines break only at `\n`. Each line box is `line_height * size` tall with the font's ascent and descent centred in it; the block of line boxes is centred on the anchor and each line is aligned left, centre or right of it. Letter spacing goes after every glyph that advances the pen. Mixed-direction runs on one line are not reordered.
- **Glyphs as nodes.** `Scene::add_text` adds one `Shape::Path` node per glyph that draws something (spaces are skipped). Each glyph's path is drawn about its pivot, the middle of its advance on the baseline, so glyphs scale and turn about their own foot and stay on the line. A text is therefore a group of path nodes and renders through the vector pipeline like any shape.
- **Text as a group.** A `TextHandle` describes changes to all of a text's glyphs: `fade_to`, `fill_to`, `stroke_to`, `stroke_width_to`, `move_to` (glyphs keep their offsets from the anchor) and `text_to`. These are resolved into per-glyph changes when the action is scheduled, against the text's state at that point of the build. `Scene::glyphs` hands out `Handle<Glyph>`s for animating glyphs one by one.
- **`text_to`.** The new text is shaped in the same style at the current anchor and matched glyph for glyph in reading order: each matched glyph morphs into its new glyph (prepared like any morph) and slides to its new place. Leftover old glyphs shrink to scale 0 about their baseline pivot. Extra new glyphs become new nodes, created at scale 0 and drawn above everything added earlier, that grow to full size. Afterwards the text's glyph list is the new one.
- **Quality.** `kinetic-render/tests/text_quality.rs` compares rendered glyphs with an exact-coverage rasteriser at 1x and 2x: ink matches within 1% and edge pixels are off by about 0.08 on average, the limit of 4x MSAA's five coverage levels.

## Renderer

- Surface or offscreen target, `Rgba8UnormSrgb` output, 4x MSAA colour attachment.
- One pipeline for solid-colour triangles with per-node transform and colour in a uniform buffer array. Blending is premultiplied alpha in linear space.
- Post stack: an ordered list of fullscreen passes ping-ponging between two textures. Blur is separable Gaussian; glow is threshold, blur, add. Motion blur averages `n` sub-frame evaluations across the shutter interval, which is cheap because any `t` can be evaluated directly.
- The renderer takes a `FrameState` and a target; it knows nothing about time.

## Hot reload

Built in M3. A scene is a crate whose library is a `cdylib` (plus `rlib`, so its tests and other crates can still link it), depending only on `kinetic-core`.

- **Exports.** `kinetic_scene!(scene)` generates three C functions: `kinetic_scene_abi_version() -> u32`, `kinetic_scene_build(*mut SceneBytes) -> u32` and `kinetic_scene_free(SceneBytes)`. Build runs the scene inside `catch_unwind`, so a panic comes back as a status code and message instead of unwinding into the editor. The library frees its own allocation.
- **Wire format.** `KNTC`, a little-endian format version, then the timeline as JSON with exact float parsing (`kinetic_core::wire`). The editor checks the ABI version before calling anything and the format version before decoding, and names both numbers when they differ.
- **Building** (`kinetic-reload`). `cargo build --lib --message-format=json --manifest-path <scene>/Cargo.toml` into its own target directory under the system temp folder, so a scene build never waits on, or invalidates, the build of the editor running it. The cdylib comes from the artifact messages; on failure the compiler's rendered errors come back as text.
- **Loading.** The built library is copied to a uniquely named scratch file and the copy is loaded (Windows locks loaded libraries; cargo must be able to overwrite the original). The bytes are copied out, freed by the library, the library is unloaded and the copy deleted. Nothing returned points into the library.
- **Watching.** A debounced (150 ms) watcher on `src/` and `Cargo.toml` queues rebuilds; one worker thread drains the queue, so saves during a build cause one more build. Events (`Building`, `Loaded`, `Failed`) reach the editor through the winit event loop.
- **Editor.** Building shows a spinner; a failed build shows the compiler output while the previous timeline keeps playing; a reload shows how long it took. The playhead and play state are kept, clamped if the timeline got shorter.
- **Measured:** first build of `scenes/hello` about 22 s (compiling dependencies), then 0.3 to 0.7 s from save to new timeline.
- `kinetic new <dir>` writes a starter scene crate: a relative path dependency when inside a checkout of this repository (joining the workspace under `scenes/`), otherwise a git dependency and its own `[workspace]`.

## Export

- `kinetic render <scene> -o out.mp4 --fps 60 --size 1920x1080`.
- For each frame `i`: `t = i / fps`, evaluate, render offscreen, read back with row padding removed, write RGBA to ffmpeg's stdin.
- ffmpeg: `-f rawvideo -pix_fmt rgba -s WxH -r FPS -i - -c:v libx264 -pix_fmt yuv420p -crf 16`.
- `--frames dir/` writes a PNG sequence instead and needs no ffmpeg.

## Testing

| Layer | Method |
|---|---|
| Easing, springs | Unit tests for endpoints; property tests for continuity and bounds |
| Timeline | Exact `FrameState` comparison; serialisation round trip; evaluate-order independence |
| Morph | Invariants (point count, winding, endpoints equal sources); goldens for hard pairs |
| Render | Golden PNGs on a software adapter, small per-pixel tolerance, diff image written on failure |
| Export | Frame hash equality across two runs; `ffprobe` on the output |

## Decisions

Record choices here as they are made, newest first, one line each with the reason.

- `text_to` adds a node for each glyph the new text needs beyond the old ones instead of reserving nodes up front; the node count of a scene is only known once it is built, and new glyphs drawing above earlier nodes is acceptable for text.
- A text is a group of glyph nodes, not one node with a combined path, so glyphs can be staggered; group scale and rotation wait for real groups (M8), and a text-level move is applied as per-glyph moves.
- Glyphs pivot at the middle of their advance on the baseline: a row of glyphs scaled or turned one by one stays on its line, and `text_to` grows and shrinks glyphs at a point on the baseline as planned.
- One `Action` can carry several changes (`Action::all`), so groups play, run in parallel and stagger as one step.
- Text is shaped in `kinetic-core` with rustybuzz and a bundled font, not in `kinetic-text` with cosmic-text. Scenes depend on core only and text has to be shaped while the scene builds (per-glyph nodes, morphs prepared at build time); rustybuzz is pure computation, while cosmic-text keeps a stateful font system and pulls in file and memory-map IO. Fonts are `'static` bytes, so scenes bundle their own with `include_bytes!`.
- The bundled font is Poppins Regular and Bold (SIL OFL 1.1): geometric, good for motion, and it covers Devanagari as well as Latin.
- The mesh cache confirms every hash hit by comparing the full geometry, so a 64-bit collision costs a re-tessellation, never a wrong frame. It keeps only the last frame's meshes, which bounds memory while scrubbing.
- `trim` applies to the outline only, not the fill; a trimmed fill has no clear meaning for closed shapes and draw-on effects are drawn with outlines.
- Outline width scales with `sqrt(|sx * sy|)` of the node's scale: uniform scale behaves as expected and a squashed node keeps an even-looking outline instead of a stretched one.
- An unpaired contour in a morph grows from or shrinks to its own centroid, not its nearest partner's, so a ring's hole closes in place.
- Morphs are prepared when the segment is pushed and stored as two outlines of the same structure, so evaluating a frame is a lerp and the prepared data travels in the wire format to the editor.
- `PathData` is a core type rather than a wrapper around `kurbo::BezPath`, keeping `kinetic-core` free of geometry crates; vector converts it.

- The preview window presents through DX12 on Windows unless `WGPU_BACKEND` says otherwise. It was changed while chasing a whole-screen flicker that later turned out to come from the machine's graphics, not the preview; DX12 stays as the Windows default because it is the native path.
- The preview window opens at 70% of the screen, 16:9 and centred. A fixed 1280x720 logical size filled a 150%-scaled 1920x1080 laptop screen and hid the title bar.
- `kinetic preview` needs a scene directory; the hello scene is no longer compiled into the binary.
- A scene library is unloaded as soon as its bytes are copied out; nothing the editor keeps points into it.
- Scene builds use their own target directory (under the system temp folder) instead of the workspace's, so they never contend with the build running the editor.
- The wire format is JSON behind a `KNTC` + version header rather than a binary format: readable when debugging, exact for floats with serde_json's `float_roundtrip`, and fast enough for timelines of this size.
- Hot reload code lives in its own crate, `kinetic-reload`, depending only on `kinetic-core`; the editor depends on it.
- Until hot reload (M3), `kinetic preview` plays `scenes/hello` compiled into the binary; scenes depend only on `kinetic-core` so they can become dylibs unchanged.
- The hot reload wire format must round-trip `f64` exactly; serde_json does so only with its `float_roundtrip` feature.
- Spring progress closes its last sub-0.1% gap with a linear correction over the whole duration instead of snapping at the settle time, so a change that starts there does not see a step.
- Expo and elastic easings are anchored linearly to meet 0 and 1 exactly; the textbook formulas sit about 1/1024 off their ends and relied on clamping.
- On a track, the segment that started most recently owns the value; an interrupting change resolves its `from` at its own start, so hand-offs never jump.
- A segment returns exactly its target from its end time on, compared in absolute time, because `t - start` can round under the duration.
- Timing is one argument (`impl Into<Timing>`): seconds, `(seconds, Ease)`, `(seconds, CubicBezier)` or a `Spring`. Easings are an `Ease` enum rather than function pointers so timelines can be serialised.
- Render tests share one software-adapter device per test binary and take turns with it, because creating devices from parallel test threads crashed on Windows.
- Golden images pass on both WARP (Windows) and lavapipe (Linux CI) with a 2/255 per-channel tolerance, so one set of goldens serves both.
- Landing page street is layered SVG paper, to match the approved paper cut-out Kin; GSAP ScrollTrigger scrubs one master time, Three.js draws the flying things in depth, and signboard text morphs HarfBuzz-shaped outlines.
- Landing page is a Three.js scroll journey through Old Delhi (was an SVG rig), after the user shared a 3D fly-through reference; the page keeps the rule that everything is a function of scroll time.
- Spring API is a builder (`spring().stiffness(180.0)`), since Rust has no named arguments.
- Only serialised bytes cross the scene dylib boundary, to avoid depending on Rust's unstable ABI.
- Time is `f64` seconds; frame time is computed from the frame index, never accumulated.
