//! Golden image tests. Regenerate with `cargo xtask bless` and look at the result.

mod common;

use kinetic_core::color::palette;
use kinetic_core::{
    Align, Circle, DVec2, Ease, Line, LineCap, LineJoin, LinearRgba, Node, PathData, PathShape,
    Polygon, Rect, Scene, Shape, Star, Text,
};
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

#[test]
fn outlines() {
    common::with_gpu(|gpu| {
        let mut s = Scene::new();
        let white = palette::WHITE;
        // Butt, round and square caps, top to bottom.
        for (cap, y) in [(LineCap::Butt, 45.0), (LineCap::Round, 0.0), (LineCap::Square, -45.0)] {
            s.add(
                Line::new((-40.0, 0.0), (40.0, 0.0))
                    .stroke(white, 14.0)
                    .line_cap(cap)
                    .at((-225.0, y)),
            );
        }
        // Miter, round and bevel joins on a triangle over its fill.
        for (join, x) in
            [(LineJoin::Miter, -100.0), (LineJoin::Round, 0.0), (LineJoin::Bevel, 100.0)]
        {
            s.add(
                Polygon::new(3, 42.0)
                    .fill(palette::amber())
                    .stroke(white, 12.0)
                    .line_join(join)
                    .at((x, -6.0)),
            );
        }
        // A circle's outline trimmed to its first seven tenths, with no fill.
        s.add(
            Circle::new(40.0)
                .fill(LinearRgba::TRANSPARENT)
                .stroke(palette::blue(), 10.0)
                .trim(0.0, 0.7)
                .at((225.0, 0.0)),
        );
        let (w, h) = (600, 160);
        let mut frame = Frame::from_state(&s.build().evaluate(0.0), 0.1);
        frame.background = palette::ink();
        frame.size = DVec2::new(f64::from(w), f64::from(h));
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
        common::assert_golden("outlines", w, h, &image);
    });
}

/// Renders `items` on the ink background at one scene unit per pixel and checks them.
fn golden_frame(name: &str, size: DVec2, items: Vec<Item>) {
    common::with_gpu(|gpu| {
        let frame = Frame { background: palette::ink(), size, items };
        let (w, h) = (size.x as u32, size.y as u32);
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
        common::assert_golden(name, w, h, &image);
    });
}

#[test]
fn text_paragraph() {
    let mut s = Scene::new();
    let bone = LinearRgba::from_hex(0xe2d6bf);
    s.add_text(Text::new("Kinetic").bold().size(56.0).align(Align::Left).at((-210.0, 70.0)));
    s.add_text(
        Text::new("Motion, written as code.\nEvery frame a function of time.")
            .size(22.0)
            .line_height(1.4)
            .align(Align::Left)
            .fill(bone)
            .at((-210.0, -10.0)),
    );
    s.add_text(
        Text::new("नमस्ते, दिल्ली")
            .size(30.0)
            .align(Align::Right)
            .fill(palette::amber())
            .at((210.0, -90.0)),
    );
    let items = Frame::from_state(&s.build().evaluate(0.0), 0.1).items;
    golden_frame("text_paragraph", DVec2::new(480.0, 270.0), items);
}

#[test]
fn text_hello_to_world() {
    // Hello turns into World; rows show t = 0, 0.25, 0.5, 0.75 and 1, top to bottom.
    let mut s = Scene::new();
    let word = s.add_text(Text::new("Hello").bold().size(64.0));
    s.play(word.text_to("World"), (1.0, Ease::Linear));
    let timeline = s.build();
    let mut items = Vec::new();
    for (k, t) in [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
        for mut item in Frame::from_state(&timeline.evaluate(t), 0.1).items {
            item.offset.y += 160.0 - 80.0 * k as f64;
            items.push(item);
        }
    }
    golden_frame("text_hello_to_world", DVec2::new(320.0, 420.0), items);
}
