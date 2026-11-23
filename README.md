# Kinetic

A native, GPU-accelerated motion engine. Write animations in Rust, watch them live, scrub the timeline, export to MP4.

> Status: early development. See [docs/MILESTONES.md](docs/MILESTONES.md) for what works today.

```rust
fn scene(s: &mut Scene) {
    let circle = s.add(Circle::new(80.0).fill(palette::blue()));
    let bars: Vec<_> = (0..5).map(|i| s.add(Rect::new(40.0, 120.0).at((i as f64 * 120.0, -200.0)))).collect();

    s.play(circle.scale_to(1.5), (0.6, Ease::OutBack));
    s.parallel([circle.move_to((0.0, 60.0)), circle.fill_to(red)], (1.0, CubicBezier::EASE));
    s.play(circle.move_to((300.0, 0.0)), spring().stiffness(180.0).damping(10.0));
    s.stagger(bars.iter().map(|b| b.fade_to(1.0)), (0.5, Ease::OutCubic), 0.08);
}
```

**[See the landing page](https://hashimhashmi02.github.io/kinetic-engine/)**: follow Kin through Old Delhi and scrub the street with your scroll.

## Why

- **Scrubbable by construction.** Scene code compiles to a timeline up front, so any frame is a pure function of time.
- **Springs without simulation.** Closed-form solutions, so springs scrub and export exactly like easings.
- **Everything is a path.** Shapes and text share one vector pipeline, so anything can morph into anything.
- **Hot reload.** Save the scene file and the preview updates in place.

## Layout

| Crate | Role |
|---|---|
| `kinetic-core` | Scene graph, properties, timeline, easing, springs |
| `kinetic-vector` | Paths, tessellation, morphing |
| `kinetic-text` | Shaping, glyphs as paths |
| `kinetic-render` | wgpu renderer and post-processing |
| `kinetic-export` | Frames to ffmpeg |
| `kinetic-editor` | Preview, timeline, inspector |
| `kinetic-cli` | `kinetic preview`, `kinetic render` |

## Building

Requires stable Rust and, for export, `ffmpeg` on `PATH`.

```bash
cargo test --workspace
cargo run -p kinetic-cli -- preview    # plays scenes/hello
```

Timings are written as seconds (`0.6`), seconds with an easing (`(0.6, Ease::OutBack)`), seconds with a cubic bezier (`(0.6, CubicBezier::EASE)`), or a spring, which lasts until it settles.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
