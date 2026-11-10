# Kinetic

A native, GPU-accelerated motion engine. Write animations in Rust, watch them live, scrub the timeline, export to MP4.

> Status: early development. See [docs/MILESTONES.md](docs/MILESTONES.md) for what works today.

```rust
fn scene(s: &mut Scene) {
    let circle = s.add(Circle::new(80.0).fill(BLUE));
    let title = s.add(Text::new("Hello").size(64.0));

    s.play(circle.scale_to(1.5), 0.6, ease::out_back);
    s.parallel([circle.morph_to(Rect::new(200.0, 120.0)), title.text_to("World")], 1.0);
    s.play(circle.move_to((300.0, 0.0)), spring().stiffness(180.0));
}
```

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
```

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at your option.
