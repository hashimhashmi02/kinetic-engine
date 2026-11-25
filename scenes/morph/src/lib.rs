//! Shapes that change into other shapes. An outline draws itself on and fills in, then
//! becomes a square, a kite on a string and a star before it rolls back into a circle.
//! To the side, a ring closes into a sun and a hexagon's outline chases itself round.
//!
//! Every change of shape is matched up once, when the scene is built, so scrubbing a
//! morph costs no more than scrubbing a move.

use kinetic_core::color::palette;
use kinetic_core::{
    spring, Circle, Ease, Line, LineCap, LinearRgba, PathShape, Polygon, RoundedRect, Scene, Shape,
    Star, Timeline,
};

/// Writes the scene into `s`.
pub fn scene(s: &mut Scene) {
    let kite_red = LinearRgba::from_hex(0xd8402f);
    let marigold = LinearRgba::from_hex(0xf2a03d);
    let bone = LinearRgba::from_hex(0xe2d6bf);

    // The hero starts as an outline that has not been drawn yet.
    let hero = s.add(
        Circle::new(120.0)
            .fill(LinearRgba::TRANSPARENT)
            .stroke(bone, 6.0)
            .trim(0.0, 0.0)
            .at((0.0, 40.0)),
    );
    let string = s.add(
        Line::new((0.0, 0.0), (-40.0, -230.0))
            .stroke(bone, 3.0)
            .line_cap(LineCap::Round)
            .trim(0.0, 0.0)
            .at((0.0, -80.0)),
    );
    // A ring: the hole is a smaller circle mirrored so it winds the other way.
    let mut ring = Shape::Circle { radius: 70.0 }.to_path();
    ring.els.extend(Shape::Circle { radius: 42.0 }.to_path().map(|p| (-p.x, p.y).into()).els);
    let sun = s.add(PathShape::new(ring).fill(marigold).at((440.0, 140.0)).opacity(0.0));
    let hexagon = s.add(
        Polygon::new(6, 70.0)
            .fill(LinearRgba::TRANSPARENT)
            .stroke(palette::blue(), 8.0)
            .trim(0.0, 0.0)
            .at((-440.0, -140.0)),
    );

    // Draw the outline on, then fill it in.
    s.play(hero.trim_to(0.0, 1.0), (1.2, Ease::InOutCubic));
    s.play(hero.fill_to(palette::blue()), (0.4, Ease::OutCubic));

    // Circle to square.
    s.play(hero.morph_to(RoundedRect::new(220.0, 220.0, 24.0)), (0.8, Ease::InOutCubic));

    // Square to kite, which gets its string, while the sun rises and the hexagon draws on.
    s.parallel(
        [
            hero.morph_to(Polygon::new(4, 150.0)),
            hero.fill_to(kite_red),
            string.trim_to(0.0, 1.0),
            sun.fade_to(1.0),
            hexagon.trim_to(0.0, 1.0),
        ],
        (0.9, Ease::InOutCubic),
    );
    s.wait(0.3);

    // The kite lets go of its string and springs into a star; the ring closes into a sun.
    s.parallel([string.trim_to(1.0, 1.0), hexagon.trim_to(1.0, 1.0)], (0.6, Ease::InCubic));
    s.parallel(
        [
            hero.morph_to(Star::new(5, 160.0, 66.0)),
            hero.fill_to(marigold),
            sun.morph_to(Circle::new(70.0)),
        ],
        spring().stiffness(160.0).damping(11.0),
    );
    s.wait(0.4);

    // Back to a circle, and the outline undraws itself.
    s.play(hero.morph_to(Circle::new(120.0)), (1.0, Ease::InOutExpo));
    s.parallel([hero.fill_to(LinearRgba::TRANSPARENT), sun.fade_to(0.0)], (0.5, Ease::OutCubic));
    s.play(hero.trim_to(1.0, 1.0), (1.0, Ease::InOutCubic));
    s.wait(0.4);
}

kinetic_core::kinetic_scene!(scene);

/// The compiled scene.
pub fn build() -> Timeline {
    let mut s = Scene::new();
    scene(&mut s);
    s.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(shape: &Shape) -> f64 {
        match shape {
            Shape::Outline { contours } => {
                contours.iter().map(|c| kinetic_core::shape::signed_area(c)).sum()
            }
            other => panic!("expected an outline, got {other:?}"),
        }
    }

    #[test]
    fn builds_the_same_timeline_every_time() {
        assert_eq!(build(), build());
    }

    #[test]
    fn opens_and_closes_on_an_undrawn_outline() {
        let tl = build();
        for t in [0.0, tl.duration] {
            let hero = &tl.evaluate(t).nodes[0];
            assert_eq!(hero.props.trim.x, hero.props.trim.y, "nothing drawn at {t}");
        }
        let end = &tl.evaluate(tl.duration).nodes[0];
        let circle = std::f64::consts::PI * 120.0 * 120.0;
        assert!((area(&end.shape) - circle).abs() < circle * 0.01);
    }

    #[test]
    fn the_ring_closes_into_a_disc() {
        let tl = build();
        let sun = &tl.evaluate(tl.duration).nodes[2];
        let disc = std::f64::consts::PI * 70.0 * 70.0;
        assert!((area(&sun.shape) - disc).abs() < disc * 0.01);
    }

    #[test]
    fn exported_entry_points_return_the_same_timeline() {
        use kinetic_core::abi::{SceneBytes, ABI_VERSION, STATUS_OK};
        assert_eq!(kinetic_scene_abi_version(), ABI_VERSION);
        let mut out = SceneBytes::EMPTY;
        // SAFETY: out is a valid SceneBytes; the bytes are read, then handed back once.
        let timeline = unsafe {
            assert_eq!(kinetic_scene_build(&mut out), STATUS_OK);
            let timeline = kinetic_core::wire::decode(out.as_slice()).expect("decode");
            kinetic_scene_free(out);
            timeline
        };
        assert_eq!(timeline, build());
    }
}
