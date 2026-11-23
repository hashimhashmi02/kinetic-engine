//! The first Kinetic scene: a circle that grows, travels and springs, and a row of bars
//! that arrive one by one.
//!
//! Scenes depend only on `kinetic-core`. Until hot reload lands (M3) this one is compiled
//! into the `kinetic` binary as the built-in preview.

use std::f64::consts::FRAC_PI_4;

use kinetic_core::color::palette;
use kinetic_core::{spring, Circle, CubicBezier, Ease, LinearRgba, Rect, Scene, Spring, Timeline};

/// Writes the scene into `s`.
pub fn scene(s: &mut Scene) {
    let kite_red = LinearRgba::from_hex(0xd8402f);
    let bone = LinearRgba::from_hex(0xe2d6bf);

    let dot = s.add(Circle::new(80.0).fill(palette::blue()).at((-360.0, 60.0)).opacity(0.0));
    let bars: Vec<_> = (0..5)
        .map(|i| {
            let x = -240.0 + 120.0 * f64::from(i);
            s.add(Rect::new(40.0, 120.0).fill(bone).at((x, -200.0)).opacity(0.0))
        })
        .collect();

    // The circle appears, grows with an overshoot, then crosses the frame changing colour.
    s.play(dot.fade_to(1.0), 0.4);
    s.play(dot.scale_to(1.5), (0.6, Ease::OutBack));
    s.parallel([dot.move_to((0.0, 60.0)), dot.fill_to(kite_red)], (1.0, CubicBezier::EASE));
    s.play(dot.move_to((360.0, 60.0)), spring().stiffness(180.0).damping(10.0));

    // The bars arrive left to right, then tip over on a wobbly spring.
    s.stagger(bars.iter().map(|b| b.fade_to(1.0)), (0.5, Ease::OutCubic), 0.08);
    s.stagger(bars.iter().map(|b| b.rotate_to(FRAC_PI_4)), Spring::WOBBLY, 0.06);

    // Everything goes home.
    s.parallel([dot.scale_to(1.0), dot.move_to((-360.0, 60.0))], (1.2, Ease::InOutExpo));
    s.stagger(bars.iter().rev().map(|b| b.rotate_to(0.0)), (0.4, Ease::InOutCubic), 0.05);
    s.wait(0.5);
}

/// The compiled scene.
pub fn build() -> Timeline {
    let mut s = Scene::new();
    scene(&mut s);
    s.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::DVec2;

    #[test]
    fn builds_the_same_timeline_every_time() {
        assert_eq!(build(), build());
    }

    #[test]
    fn opens_empty_and_ends_where_it_started() {
        let tl = build();
        assert!(tl.duration > 5.0);
        let start = tl.evaluate(0.0);
        assert!(start.nodes.iter().all(|n| n.props.opacity == 0.0));
        let end = tl.evaluate(tl.duration);
        assert_eq!(end.nodes[0].props.position, DVec2::new(-360.0, 60.0));
        assert!(end.nodes.iter().skip(1).all(|n| n.props.rotation == 0.0));
    }
}
