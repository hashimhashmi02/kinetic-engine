# Architecture

## Crates

```
kinetic-core      scene graph, properties, timeline, easing, springs     (pure)
   ^
kinetic-vector    kurbo paths, lyon tessellation, resampling, morph
   ^
kinetic-text      cosmic-text shaping, glyph outlines to paths

kinetic-render    wgpu renderer, MSAA, post stack        (core, vector, text)
kinetic-export    offscreen frames to ffmpeg             (render)
kinetic-editor    winit + egui preview, hot reload       (render)
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
let circle: Handle<Circle> = s.add(Circle::new(80.0).fill(BLUE));
s.play(circle.scale_to(1.5), 0.6, ease::out_back);
s.parallel([a.move_to(p), b.fade_to(0.0)], 1.0);
s.stagger(items.iter().map(|i| i.fade_to(1.0)), 0.4, 0.05);
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

- `PathData` wraps `kurbo::BezPath`. Shapes (`Circle`, `Rect`, `Polygon`, `Star`, `Line`, custom) all lower to a path.
- **Morph** between paths A and B:
  1. Split into subpaths. Pair subpaths greedily by centroid distance and area. Unpaired subpaths morph to or from a point at the centroid of their nearest partner.
  2. For each pair, flatten to a polyline by arc length and resample both to N points (N from the longer perimeter, clamped).
  3. Make winding equal. Rotate B's start index to minimise the sum of squared point distances.
  4. Interpolate point-wise. Open and closed paths are handled by treating an open path as a closed one walked out and back.
- The prepared (resampled, aligned) pair is computed at build time and stored in the segment, so evaluation is a lerp.

## Text

- cosmic-text shapes and lays out with a bundled font (deterministic across machines). System fonts are opt-in.
- Each glyph outline is converted to a kurbo path and positioned. A text node is therefore a group of paths and renders through the vector pipeline.
- `text_to` morphs glyph-to-glyph in order; surplus glyphs grow from or shrink to a point on the baseline.

## Renderer

- Surface or offscreen target, `Rgba8UnormSrgb` output, 4x MSAA colour attachment.
- One pipeline for solid-colour triangles with per-node transform and colour in a uniform buffer array. Blending is premultiplied alpha in linear space.
- Post stack: an ordered list of fullscreen passes ping-ponging between two textures. Blur is separable Gaussian; glow is threshold, blur, add. Motion blur averages `n` sub-frame evaluations across the shutter interval, which is cheap because any `t` can be evaluated directly.
- The renderer takes a `FrameState` and a target; it knows nothing about time.

## Hot reload

- A scene crate is a `cdylib` depending on `kinetic-core` and exporting, via a macro:
  `extern "C" fn kinetic_scene_build(out: *mut ByteBuf) -> u32` which runs the scene, serialises the `Timeline`, and hands back bytes plus a free function.
- The editor watches the scene's `src/` with `notify`, debounces, runs `cargo build -p <scene>`, copies the built library to a unique temp path (Windows keeps loaded DLLs locked), loads it with `libloading`, calls the entry, copies the bytes, frees them, and deserialises.
- A wire-format version constant is checked first, so a stale scene built against an older core fails with a clear message.
- Build errors are shown as an overlay; the previous timeline keeps playing. The playhead is clamped to the new duration.

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

- Landing page is a Three.js scroll journey through Old Delhi (was an SVG rig), after the user shared a 3D fly-through reference; the page keeps the rule that everything is a function of scroll time.
- Spring API is a builder (`spring().stiffness(180.0)`), since Rust has no named arguments.
- Only serialised bytes cross the scene dylib boundary, to avoid depending on Rust's unstable ABI.
- Time is `f64` seconds; frame time is computed from the frame index, never accumulated.
