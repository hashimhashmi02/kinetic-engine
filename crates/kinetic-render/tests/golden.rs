//! Golden image tests. Regenerate with `cargo xtask bless` and look at the result.

mod common;

use kinetic_core::color::palette;
use kinetic_core::{Circle, DVec2, Ease, Node, PathData, PathShape, Rect, Scene, Shape, Star};
use kinetic_render::{render_to_rgba, Frame, Item, Renderer, TARGET_FORMAT};
use kinetic_vector::{fill, shape};

#[test]
fn first_shapes() {
    common::with_gpu(|gpu| {
        let (w, h) = (256, 144);
        let item = |path, color, x| Item {
            mesh: fill(&path, 0.1).expect("tessellate"),
            color,
            offset: DVec2::new(x, 0.0),
        };
        let frame = Frame {
            background: palette::ink(),
            size: DVec2::new(f64::from(w), f64::from(h)),
            items: vec![
                item(shape::circle(DVec2::ZERO, 40.0), palette::blue(), -56.0),
                item(shape::rect(DVec2::ZERO, DVec2::new(84.0, 60.0)), palette::amber(), 62.0),
            ],
        };
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
        common::assert_golden("first_shapes", w, h, &image);
    });
}

/// Renders `from` morphing into `to` at t = 0, 0.25, 0.5, 0.75 and 1, left to right, through
/// the whole pipeline: scene, timeline, lowering and renderer.
fn morph_strip(name: &str, from: impl Node, to: impl Node) {
    common::with_gpu(|gpu| {
        let mut s = Scene::new();
        let node = s.add(from);
        s.play(node.morph_to(to), (1.0, Ease::Linear));
        let timeline = s.build();

        let (w, h) = (600, 120);
        let mut items = Vec::new();
        for (k, t) in [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
            let x = (k as f64 - 2.0) * 120.0;
            for mut item in Frame::from_state(&timeline.evaluate(t), 0.1).items {
                item.offset.x += x;
                items.push(item);
            }
        }
        let frame = Frame {
            background: palette::ink(),
            size: DVec2::new(f64::from(w), f64::from(h)),
            items,
        };
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
        common::assert_golden(name, w, h, &image);
    });
}

#[test]
fn morph_circle_to_rect() {
    morph_strip(
        "morph_circle_to_rect",
        Circle::new(40.0).fill(palette::blue()),
        Rect::new(90.0, 60.0),
    );
}

#[test]
fn morph_star_to_blob() {
    // An uneven closed curve, wound clockwise unlike the star, so winding is matched too.
    let blob = PathData::default()
        .move_to((0.0, 45.0))
        .cubic_to((30.0, 45.0), (48.0, 20.0), (42.0, -5.0))
        .cubic_to((36.0, -30.0), (10.0, -44.0), (-12.0, -40.0))
        .cubic_to((-38.0, -36.0), (-46.0, -8.0), (-40.0, 14.0))
        .cubic_to((-34.0, 36.0), (-28.0, 45.0), (0.0, 45.0))
        .close();
    morph_strip(
        "morph_star_to_blob",
        Star::new(5, 50.0, 22.0).fill(palette::amber()),
        PathShape::new(blob),
    );
}

#[test]
fn morph_ring_to_disc() {
    // The hole is the inner circle mirrored, so it winds the other way.
    let mut ring = Shape::Circle { radius: 44.0 }.to_path();
    let hole = Shape::Circle { radius: 24.0 }.to_path().map(|p| DVec2::new(-p.x, p.y));
    ring.els.extend(hole.els);
    morph_strip(
        "morph_ring_to_disc",
        PathShape::new(ring).fill(palette::green()),
        Circle::new(40.0),
    );
}
