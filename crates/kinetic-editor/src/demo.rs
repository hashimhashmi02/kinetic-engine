//! The first animation: one circle, one eased move.
//!
//! This is hard-coded on purpose. The timeline that will replace it arrives in the next
//! milestone; until then it proves the loop from a time value to pixels.

use kinetic_core::color::palette;
use kinetic_core::{ease, DVec2, Lerp};
use kinetic_render::{Frame, Item};
use kinetic_vector::{fill, shape, Mesh};

/// Length of the demo in seconds.
pub const DURATION: f64 = 2.0;
/// Scene size in scene units.
pub const SIZE: DVec2 = DVec2::new(1280.0, 720.0);

const FROM: DVec2 = DVec2::new(-360.0, 0.0);
const TO: DVec2 = DVec2::new(360.0, 0.0);

/// The demo's shapes, tessellated once.
#[derive(Debug, Clone)]
pub struct Demo {
    circle: Mesh,
    track: Mesh,
}

impl Demo {
    /// Builds the meshes. `tolerance` is the curve flattening tolerance in scene units.
    pub fn new(tolerance: f64) -> Self {
        // Filling a circle or a rectangle cannot fail; an empty mesh would only leave the
        // shape undrawn.
        let mesh = |path| fill(&path, tolerance).unwrap_or_default();
        Self {
            circle: mesh(shape::circle(DVec2::ZERO, 80.0)),
            track: mesh(shape::rect(DVec2::ZERO, DVec2::new(TO.x - FROM.x, 4.0))),
        }
    }

    /// Where the circle is at time `t` (seconds). A pure function of `t`.
    pub fn position(t: f64) -> DVec2 {
        Lerp::lerp(&FROM, &TO, ease::in_out_cubic(t / DURATION))
    }

    /// The frame at time `t`.
    pub fn frame(&self, t: f64) -> Frame {
        Frame {
            background: palette::ink(),
            size: SIZE,
            items: vec![
                Item {
                    mesh: self.track.clone(),
                    color: palette::WHITE.with_opacity(0.12),
                    offset: DVec2::ZERO,
                },
                Item {
                    mesh: self.circle.clone(),
                    color: palette::blue(),
                    offset: Self::position(t),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_and_ends_exactly_on_the_keyframes() {
        assert_eq!(Demo::position(0.0), FROM);
        assert_eq!(Demo::position(DURATION), TO);
    }

    #[test]
    fn time_outside_the_range_holds_the_ends() {
        assert_eq!(Demo::position(-1.0), FROM);
        assert_eq!(Demo::position(DURATION + 5.0), TO);
    }

    #[test]
    fn midpoint_is_halfway_and_easing_is_slow_at_the_ends() {
        assert_eq!(Demo::position(DURATION / 2.0), DVec2::ZERO);
        // A tenth of the way through time, the eased circle has covered far less than a
        // tenth of the distance.
        let early = Demo::position(DURATION * 0.1).x - FROM.x;
        assert!(early < 0.02 * (TO.x - FROM.x), "moved {early}");
    }

    #[test]
    fn frame_is_a_pure_function_of_time() {
        let demo = Demo::new(0.25);
        assert_eq!(demo.frame(0.7), demo.frame(0.7));
        assert_ne!(demo.frame(0.7), demo.frame(0.8));
    }
}
