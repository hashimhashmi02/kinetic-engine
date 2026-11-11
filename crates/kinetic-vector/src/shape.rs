//! Basic shapes lowered to bezier paths.
//!
//! Coordinates are scene units with the origin at the centre of the frame, +x right and
//! +y up. Every shape becomes a [`BezPath`], so the rest of the pipeline only deals with
//! paths.

use kinetic_core::DVec2;
use kurbo::{BezPath, Circle, Rect, Shape};

/// Accuracy used when a shape with true curves is approximated by cubic beziers, in scene
/// units. This is far below a pixel; tessellation applies its own, coarser tolerance.
const CURVE_ACCURACY: f64 = 1e-3;

/// A circle of `radius` around `center`.
pub fn circle(center: DVec2, radius: f64) -> BezPath {
    Circle::new((center.x, center.y), radius).to_path(CURVE_ACCURACY)
}

/// An axis-aligned rectangle of `size` (width, height) centred on `center`.
pub fn rect(center: DVec2, size: DVec2) -> BezPath {
    let half = size / 2.0;
    Rect::new(center.x - half.x, center.y - half.y, center.x + half.x, center.y + half.y)
        .to_path(CURVE_ACCURACY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_bounds_match_radius() {
        let b = circle(DVec2::new(10.0, -4.0), 5.0).bounding_box();
        for (got, want) in [(b.x0, 5.0), (b.x1, 15.0), (b.y0, -9.0), (b.y1, 1.0)] {
            assert!((got - want).abs() < 1e-6, "{got} vs {want}");
        }
    }

    #[test]
    fn circle_area_is_pi_r_squared() {
        let area = circle(DVec2::ZERO, 80.0).area().abs();
        assert!((area - std::f64::consts::PI * 6400.0).abs() < 0.5, "area was {area}");
    }

    #[test]
    fn rect_bounds_and_area() {
        let path = rect(DVec2::new(0.0, 0.0), DVec2::new(200.0, 120.0));
        let b = path.bounding_box();
        assert_eq!((b.x0, b.y0, b.x1, b.y1), (-100.0, -60.0, 100.0, 60.0));
        assert_eq!(path.area().abs(), 24_000.0);
    }
}
