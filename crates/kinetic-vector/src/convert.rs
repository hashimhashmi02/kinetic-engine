//! Converting core paths to kurbo paths.

use kinetic_core::{PathData, PathEl};
use kurbo::{BezPath, Point};

fn point(p: kinetic_core::DVec2) -> Point {
    Point::new(p.x, p.y)
}

/// The same path as a kurbo [`BezPath`].
pub fn to_kurbo(path: &PathData) -> BezPath {
    let mut out = BezPath::new();
    for el in &path.els {
        match *el {
            PathEl::MoveTo(p) => out.move_to(point(p)),
            PathEl::LineTo(p) => out.line_to(point(p)),
            PathEl::QuadTo(c, p) => out.quad_to(point(c), point(p)),
            PathEl::CubicTo(c1, c2, p) => out.curve_to(point(c1), point(c2), point(p)),
            PathEl::Close => out.close_path(),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::Shape;
    use kurbo::Shape as _;

    #[test]
    fn every_element_kind_converts() {
        let path = PathData::new()
            .move_to((0.0, 0.0))
            .line_to((4.0, 0.0))
            .quad_to((6.0, 2.0), (4.0, 4.0))
            .cubic_to((3.0, 5.0), (1.0, 5.0), (0.0, 4.0))
            .close();
        assert_eq!(to_kurbo(&path).elements().len(), path.els.len());
    }

    #[test]
    fn converted_circle_keeps_its_area() {
        let path = to_kurbo(&Shape::Circle { radius: 10.0 }.to_path());
        assert!((path.area() - std::f64::consts::PI * 100.0).abs() < 0.1);
        assert!(path.area() > 0.0, "counter-clockwise stays positive");
    }
}
