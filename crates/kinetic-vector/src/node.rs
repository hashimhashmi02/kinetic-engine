//! Node shapes from `kinetic-core` lowered to paths.

use kinetic_core::{DVec2, Shape};
use kurbo::{Affine, BezPath};

use crate::shape;

/// The path for `shape`, scaled per axis and then rotated counter-clockwise by `rotation`
/// radians, all about the shape's own centre. Translation is left to the renderer, so a
/// node that only moves keeps the same path.
pub fn node_path(shape: &Shape, scale: DVec2, rotation: f64) -> BezPath {
    let base = match *shape {
        Shape::Circle { radius } => shape::circle(DVec2::ZERO, radius),
        Shape::Rect { size } => shape::rect(DVec2::ZERO, size),
    };
    Affine::rotate(rotation) * Affine::scale_non_uniform(scale.x, scale.y) * base
}

#[cfg(test)]
mod tests {
    use super::*;
    use kurbo::Shape as _;

    #[test]
    fn identity_transform_keeps_the_shape() {
        let path = node_path(&Shape::Rect { size: DVec2::new(4.0, 2.0) }, DVec2::ONE, 0.0);
        let b = path.bounding_box();
        assert_eq!((b.x0, b.y0, b.x1, b.y1), (-2.0, -1.0, 2.0, 1.0));
    }

    #[test]
    fn scale_applies_per_axis() {
        let path = node_path(&Shape::Circle { radius: 10.0 }, DVec2::new(2.0, 0.5), 0.0);
        let b = path.bounding_box();
        // Circles are cubic approximations accurate to 1e-3 scene units, scaled here.
        assert!((b.width() - 40.0).abs() < 2e-3 && (b.height() - 10.0).abs() < 2e-3, "{b:?}");
    }

    #[test]
    fn rotation_is_counter_clockwise() {
        // A wide rectangle turned a quarter turn becomes tall.
        let quarter = std::f64::consts::FRAC_PI_2;
        let b = node_path(&Shape::Rect { size: DVec2::new(10.0, 2.0) }, DVec2::ONE, quarter)
            .bounding_box();
        assert!((b.width() - 2.0).abs() < 1e-9 && (b.height() - 10.0).abs() < 1e-9);
        // The point (1, 0) goes to (0, 1) with +y up.
        let p = Affine::rotate(quarter) * kurbo::Point::new(1.0, 0.0);
        assert!(p.x.abs() < 1e-12 && (p.y - 1.0).abs() < 1e-12);
    }
}
