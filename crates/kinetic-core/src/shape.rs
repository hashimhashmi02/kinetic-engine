//! The geometry a node draws, and its lowering to a [`PathData`].
//!
//! Every closed shape is wound counter-clockwise (positive area with +y up) and starts at
//! its topmost point, so shapes meet morphing on equal terms.

use std::f64::consts::{FRAC_PI_2, TAU};

use glam::DVec2;
use serde::{Deserialize, Serialize};

use crate::path::PathData;

/// Control point distance for a quarter circle drawn with one cubic, as a fraction of the
/// radius. The curve stays within 0.03% of the radius of the true circle.
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// The geometry a node draws, centred on the origin, before its transform is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shape {
    /// A circle.
    Circle { radius: f64 },
    /// An axis-aligned rectangle.
    Rect { size: DVec2 },
    /// A rectangle with circular corners. The radius is clamped to half the shorter side.
    RoundedRect { size: DVec2, radius: f64 },
    /// A regular polygon with a vertex at the top. At least three sides.
    Polygon { sides: u32, radius: f64 },
    /// A star with a point at the top. At least two points.
    Star { points: u32, outer: f64, inner: f64 },
    /// A straight line between two points. It has no area, so it shows only when stroked.
    Line { from: DVec2, to: DVec2 },
    /// Any path.
    Path(PathData),
}

impl Shape {
    /// The shape as a path.
    pub fn to_path(&self) -> PathData {
        match self {
            Self::Circle { radius } => circle(*radius),
            Self::Rect { size } => rounded_rect(*size, 0.0),
            Self::RoundedRect { size, radius } => rounded_rect(*size, *radius),
            Self::Polygon { sides, radius } => {
                let n = (*sides).max(3);
                closed_polygon((0..n).map(|k| polar(*radius, f64::from(k) / f64::from(n) * TAU)))
            }
            Self::Star { points, outer, inner } => {
                let n = (*points).max(2) * 2;
                closed_polygon((0..n).map(|k| {
                    let r = if k % 2 == 0 { *outer } else { *inner };
                    polar(r, f64::from(k) / f64::from(n) * TAU)
                }))
            }
            Self::Line { from, to } => PathData::new().move_to(*from).line_to(*to),
            Self::Path(path) => path.clone(),
        }
    }
}

/// A point at distance `r` from the origin, `angle` radians counter-clockwise from the top.
fn polar(r: f64, angle: f64) -> DVec2 {
    DVec2::new((angle + FRAC_PI_2).cos(), (angle + FRAC_PI_2).sin()) * r
}

fn closed_polygon(mut points: impl Iterator<Item = DVec2>) -> PathData {
    let Some(first) = points.next() else { return PathData::new() };
    points.fold(PathData::new().move_to(first), |p, q| p.line_to(q)).close()
}

fn circle(r: f64) -> PathData {
    let k = KAPPA * r;
    PathData::new()
        .move_to((0.0, r))
        .cubic_to((-k, r), (-r, k), (-r, 0.0))
        .cubic_to((-r, -k), (-k, -r), (0.0, -r))
        .cubic_to((k, -r), (r, -k), (r, 0.0))
        .cubic_to((r, k), (k, r), (0.0, r))
        .close()
}

fn rounded_rect(size: DVec2, radius: f64) -> PathData {
    let (hw, hh) = (size.x / 2.0, size.y / 2.0);
    let r = radius.clamp(0.0, hw.min(hh));
    let k = KAPPA * r;
    // Counter-clockwise from the middle of the top edge.
    let mut p = PathData::new().move_to((0.0, hh)).line_to((-hw + r, hh));
    if r > 0.0 {
        p = p.cubic_to((-hw + r - k, hh), (-hw, hh - r + k), (-hw, hh - r));
    }
    p = p.line_to((-hw, -hh + r));
    if r > 0.0 {
        p = p.cubic_to((-hw, -hh + r - k), (-hw + r - k, -hh), (-hw + r, -hh));
    }
    p = p.line_to((hw - r, -hh));
    if r > 0.0 {
        p = p.cubic_to((hw - r + k, -hh), (hw, -hh + r - k), (hw, -hh + r));
    }
    p = p.line_to((hw, hh - r));
    if r > 0.0 {
        p = p.cubic_to((hw, hh - r + k), (hw - r + k, hh), (hw - r, hh));
    }
    p.close()
}

/// Signed area of a closed polygon: positive when counter-clockwise with +y up.
pub fn signed_area(points: &[DVec2]) -> f64 {
    let n = points.len();
    (0..n).map(|i| points[i].perp_dot(points[(i + 1) % n])).sum::<f64>() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    /// Area of the shape's first contour, flattened finely.
    fn area(shape: &Shape) -> f64 {
        signed_area(&shape.to_path().flatten(0.001)[0].points)
    }

    fn top(shape: &Shape) -> DVec2 {
        shape.to_path().flatten(0.001)[0].points[0]
    }

    #[test]
    fn every_closed_shape_is_counter_clockwise_with_the_right_area() {
        let cases = [
            // Four cubics bulge slightly outside the true circle: about 0.026% more area.
            (Shape::Circle { radius: 50.0 }, PI * 2500.0, 3.0),
            (Shape::Rect { size: DVec2::new(40.0, 20.0) }, 800.0, 1e-9),
            (
                Shape::RoundedRect { size: DVec2::new(40.0, 20.0), radius: 5.0 },
                800.0 - (4.0 - PI) * 25.0,
                0.1,
            ),
            (Shape::Polygon { sides: 6, radius: 10.0 }, 1.5 * 3.0_f64.sqrt() * 100.0, 1e-9),
            // A star with n points is 2n triangles of sides outer and inner at angle pi/n.
            (
                Shape::Star { points: 5, outer: 10.0, inner: 4.0 },
                10.0 * 10.0 * 4.0 * (PI / 5.0).sin() / 2.0,
                1e-9,
            ),
        ];
        for (shape, expected, tolerance) in cases {
            let got = area(&shape);
            assert!((got - expected).abs() <= tolerance, "{shape:?}: {got} vs {expected}");
        }
    }

    #[test]
    fn closed_shapes_start_at_their_top() {
        for shape in [
            Shape::Circle { radius: 5.0 },
            Shape::Polygon { sides: 5, radius: 5.0 },
            Shape::Star { points: 4, outer: 5.0, inner: 2.0 },
        ] {
            let p = top(&shape);
            assert!(p.x.abs() < 1e-9 && (p.y - 5.0).abs() < 1e-9, "{shape:?} starts at {p}");
        }
        assert_eq!(top(&Shape::Rect { size: DVec2::new(4.0, 2.0) }), DVec2::new(0.0, 1.0));
    }

    #[test]
    fn rounded_corner_radius_is_clamped() {
        let pill = Shape::RoundedRect { size: DVec2::new(40.0, 10.0), radius: 99.0 };
        let stadium = 30.0 * 10.0 + PI * 25.0;
        assert!((area(&pill) - stadium).abs() < 0.1);
    }

    #[test]
    fn line_is_an_open_two_point_contour() {
        let line = Shape::Line { from: DVec2::ZERO, to: DVec2::new(3.0, 4.0) };
        let contours = line.to_path().flatten(0.1);
        assert_eq!(contours.len(), 1);
        assert!(!contours[0].closed);
        assert_eq!(contours[0].points, [DVec2::ZERO, DVec2::new(3.0, 4.0)]);
    }

    #[test]
    fn degenerate_counts_are_raised_to_the_minimum() {
        let tri = Shape::Polygon { sides: 1, radius: 1.0 }.to_path().flatten(0.1);
        assert_eq!(tri[0].points.len(), 3);
        let star = Shape::Star { points: 0, outer: 2.0, inner: 1.0 }.to_path().flatten(0.1);
        assert_eq!(star[0].points.len(), 4);
    }

    #[test]
    fn signed_area_sign_follows_winding() {
        let ccw = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)].map(DVec2::from);
        let cw = [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0)].map(DVec2::from);
        assert_eq!(signed_area(&ccw), 0.5);
        assert_eq!(signed_area(&cw), -0.5);
    }
}
