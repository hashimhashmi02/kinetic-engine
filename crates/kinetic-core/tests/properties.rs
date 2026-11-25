//! Property tests over randomly generated scenes.
//!
//! Scene code is generated as a list of operations (plays, waits, parallels, staggers on
//! random properties with random timings), built into a timeline, and the timeline is
//! checked for the guarantees the rest of the engine relies on.

use kinetic_core::color::palette;
use kinetic_core::{
    spring, Action, Circle, CubicBezier, DVec2, Ease, Handle, LinearRgba, Rect, Scene, Timeline,
    Timing, Value,
};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Change {
    Move(f64, f64),
    Scale(f64),
    Rotate(f64),
    Fade(f64),
    Fill(f32, f32, f32),
}

#[derive(Debug, Clone)]
enum Op {
    Play(usize, Change, Timing),
    Wait(f64),
    Parallel(Vec<(usize, Change)>, Timing),
    Stagger(Vec<(usize, Change)>, Timing, f64),
}

fn change() -> impl Strategy<Value = Change> {
    prop_oneof![
        (-500.0..500.0, -500.0..500.0).prop_map(|(x, y)| Change::Move(x, y)),
        (0.0..3.0).prop_map(Change::Scale),
        (-10.0..10.0).prop_map(Change::Rotate),
        (0.0..1.0).prop_map(Change::Fade),
        (0.0f32..1.0, 0.0f32..1.0, 0.0f32..1.0).prop_map(|(r, g, b)| Change::Fill(r, g, b)),
    ]
}

fn timing() -> impl Strategy<Value = Timing> {
    prop_oneof![
        (0.0..3.0, 0..Ease::ALL.len()).prop_map(|(d, e)| Timing::from((d, Ease::ALL[e]))),
        (0.01..3.0, 0.0..1.0, -1.0..2.0, 0.0..1.0, -1.0..2.0)
            .prop_map(|(d, a, b, c, e)| Timing::from((d, CubicBezier::new(a, b, c, e)))),
        (20.0..400.0, 1.0..60.0, -5.0..5.0)
            .prop_map(|(k, c, v)| Timing::from(spring().stiffness(k).damping(c).velocity(v))),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    let target = || (0..3usize, change());
    prop_oneof![
        3 => (0..3usize, change(), timing()).prop_map(|(n, c, t)| Op::Play(n, c, t)),
        1 => (0.0..2.0).prop_map(Op::Wait),
        1 => (prop::collection::vec(target(), 0..4), timing()).prop_map(|(v, t)| Op::Parallel(v, t)),
        1 => (prop::collection::vec(target(), 0..5), timing(), 0.0..1.0)
            .prop_map(|(v, t, d)| Op::Stagger(v, t, d)),
    ]
}

fn action(
    handles: &[Handle<Circle>; 2],
    rect: Handle<Rect>,
    node: usize,
    change: &Change,
) -> Action {
    let by = |f: &dyn Fn(Handle<Circle>) -> Action, g: &dyn Fn(Handle<Rect>) -> Action| {
        if node < 2 {
            f(handles[node])
        } else {
            g(rect)
        }
    };
    match *change {
        Change::Move(x, y) => by(&|h| h.move_to((x, y)), &|h| h.move_to((x, y))),
        Change::Scale(s) => by(&|h| h.scale_to(s), &|h| h.scale_to(s)),
        Change::Rotate(r) => by(&|h| h.rotate_to(r), &|h| h.rotate_to(r)),
        Change::Fade(o) => by(&|h| h.fade_to(o), &|h| h.fade_to(o)),
        Change::Fill(r, g, b) => {
            let c = LinearRgba::from_linear(r, g, b, 1.0);
            by(&|h| h.fill_to(c), &|h| h.fill_to(c))
        }
    }
}

fn build(ops: &[Op]) -> Timeline {
    let mut s = Scene::new();
    let circles = [s.add(Circle::new(20.0)), s.add(Circle::new(10.0).fill(palette::red()))];
    let rect = s.add(Rect::new(40.0, 20.0).at((100.0, 0.0)));
    for op in ops {
        match op {
            Op::Play(n, c, t) => s.play(action(&circles, rect, *n, c), *t),
            Op::Wait(d) => s.wait(*d),
            Op::Parallel(v, t) => {
                s.parallel(v.iter().map(|(n, c)| action(&circles, rect, *n, c)), *t)
            }
            Op::Stagger(v, t, d) => {
                s.stagger(v.iter().map(|(n, c)| action(&circles, rect, *n, c)), *t, *d)
            }
        }
    }
    s.build()
}

fn magnitude(v: &Value) -> f64 {
    match *v {
        Value::Scalar(x) => x.abs(),
        Value::Vec2(x) => x.length(),
        Value::Color(c) => c.to_array().iter().map(|x| f64::from(x.abs())).fold(0.0, f64::max),
        // These scenes never change shape.
        Value::Shape(_) => 0.0,
    }
}

fn distance(a: &Value, b: &Value) -> f64 {
    match (a, b) {
        (Value::Scalar(x), Value::Scalar(y)) => (x - y).abs(),
        (Value::Vec2(x), Value::Vec2(y)) => (x - y).length(),
        (Value::Color(x), Value::Color(y)) => {
            let (x, y) = (x.to_array(), y.to_array());
            x.iter().zip(y).map(|(p, q)| f64::from((p - q).abs())).fold(0.0, f64::max)
        }
        _ => f64::INFINITY,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn evaluate_is_pure_and_order_independent(ops in prop::collection::vec(op(), 0..12), times in prop::collection::vec(-1.0..20.0, 1..8)) {
        let tl = build(&ops);
        let forward: Vec<_> = times.iter().map(|&t| tl.evaluate(t)).collect();
        let backward: Vec<_> = times.iter().rev().map(|&t| tl.evaluate(t)).collect();
        for (a, b) in forward.iter().zip(backward.iter().rev()) {
            prop_assert_eq!(a, b);
        }
        prop_assert_eq!(build(&ops), tl);
    }

    #[test]
    fn no_track_jumps_when_a_segment_starts(ops in prop::collection::vec(op(), 0..12)) {
        let tl = build(&ops);
        for track in &tl.tracks {
            for seg in &track.segments {
                // Probe very close: circ easings move like a square root near their ends.
                let before = track.value_at(seg.start - 1e-12);
                let at = track.value_at(seg.start);
                // A zero-length segment jumps by design, including when another change
                // starts at the same instant right after it.
                let jump_here = track.segments.iter().any(|s| s.start == seg.start && s.duration == 0.0);
                if !jump_here {
                    // Relative to the size of the values: a steep curve on a large value
                    // moves further in the probe interval than the same curve on a small one.
                    let size = 1.0 + magnitude(&before).max(magnitude(&at));
                    prop_assert!(distance(&before, &at) < 1e-5 * size, "jump of {} at {}", distance(&before, &at), seg.start);
                }
            }
        }
    }

    #[test]
    fn every_track_rests_on_its_last_target_at_the_end(ops in prop::collection::vec(op(), 0..12)) {
        let tl = build(&ops);
        for track in &tl.tracks {
            if let Some(last) = track.segments.last() {
                prop_assert!(last.end() <= tl.duration + 1e-9);
                prop_assert_eq!(track.value_at(tl.duration), last.to.clone());
            }
        }
    }

    #[test]
    fn frames_are_always_finite(ops in prop::collection::vec(op(), 0..12), t in -5.0..30.0f64) {
        for node in build(&ops).evaluate(t).nodes {
            let p = node.props;
            let numbers = [p.position.x, p.position.y, p.scale.x, p.scale.y, p.rotation, p.opacity];
            prop_assert!(numbers.iter().all(|v| v.is_finite()));
            prop_assert!(p.fill.to_array().iter().all(|v| v.is_finite()));
            prop_assert!((0.0..=1.0).contains(&p.opacity));
        }
    }
}

#[test]
fn generated_scenes_exercise_every_kind_of_op() {
    // Guard against a strategy silently collapsing to one branch.
    let tl = build(&[
        Op::Play(0, Change::Move(1.0, 2.0), Timing::from(1.0)),
        Op::Wait(0.5),
        Op::Parallel(vec![(1, Change::Fade(0.0)), (2, Change::Scale(2.0))], Timing::from(spring())),
        Op::Stagger(
            vec![(0, Change::Rotate(1.0)), (2, Change::Fill(1.0, 0.0, 0.0))],
            Timing::from(0.4),
            0.1,
        ),
    ]);
    assert_eq!(tl.tracks.len(), 5);
    assert_eq!(tl.evaluate(tl.duration).nodes[0].props.position, DVec2::new(1.0, 2.0));
}
