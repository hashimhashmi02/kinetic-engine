//! A timeline survives serialisation unchanged. Hot reload depends on this: scene
//! libraries hand the editor a serialised timeline.

use kinetic_core::color::palette;
use kinetic_core::{spring, Circle, CubicBezier, Ease, Rect, Scene, Timeline};

fn sample() -> Timeline {
    let mut s = Scene::new();
    s.set_size(1920.0, 1080.0);
    s.set_background(palette::BLACK);
    let dot = s.add(Circle::new(80.0).fill(palette::blue()).at((-300.0, 0.0)));
    let card = s.add(Rect::new(200.0, 120.0).opacity(0.0));
    s.play(dot.move_to((300.0, 0.0)), (0.8, Ease::OutBack));
    s.parallel([card.fade_to(1.0), card.rotate_to(0.3)], (0.6, CubicBezier::EASE));
    s.play(dot.scale_to(1.5), spring().stiffness(220.0).damping(9.0).velocity(2.0));
    s.stagger([dot.fill_to(palette::amber()), card.scale_xy_to((2.0, 0.5))], 0.4, 0.1);
    s.wait(0.25);
    s.build()
}

#[test]
fn json_round_trip_is_exact() {
    let timeline = sample();
    let json = serde_json::to_string(&timeline).expect("serialise");
    let back: Timeline = serde_json::from_str(&json).expect("deserialise");
    assert_eq!(back, timeline);
}

#[test]
fn round_tripped_timeline_renders_the_same_frames() {
    let timeline = sample();
    let back: Timeline =
        serde_json::from_str(&serde_json::to_string(&timeline).expect("serialise"))
            .expect("deserialise");
    for i in 0..=60 {
        let t = timeline.duration * f64::from(i) / 60.0;
        assert_eq!(back.evaluate(t), timeline.evaluate(t), "at {t}");
    }
}

#[test]
fn every_curve_kind_is_in_the_sample() {
    let json = serde_json::to_string(&sample()).expect("serialise");
    for kind in ["\"Ease\"", "\"Bezier\"", "\"Spring\""] {
        assert!(json.contains(kind), "sample has no {kind} curve");
    }
}
