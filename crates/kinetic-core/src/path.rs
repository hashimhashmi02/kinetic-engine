//! Bezier paths, and their flattening into polylines.
//!
//! Coordinates are scene units, +y up. Paths live in core, not in the vector crate,
//! because morphing prepares shapes when the scene is built, and building happens here.

use glam::DVec2;
use serde::{Deserialize, Serialize};

/// One drawing command.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathEl {
    /// Starts a new subpath at a point.
    MoveTo(DVec2),
    /// A straight line to a point.
    LineTo(DVec2),
    /// A quadratic bezier through a control point to an end point.
    QuadTo(DVec2, DVec2),
    /// A cubic bezier through two control points to an end point.
    CubicTo(DVec2, DVec2, DVec2),
    /// Closes the current subpath back to its start.
    Close,
}

/// A path made of one or more subpaths.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PathData {
    pub els: Vec<PathEl>,
}

/// A flattened subpath: straight segments between `points`, joined back to the first point
/// when `closed`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contour {
    pub points: Vec<DVec2>,
    pub closed: bool,
}

impl PathData {
    /// An empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a new subpath at `p`.
    pub fn move_to(mut self, p: impl Into<DVec2>) -> Self {
        self.els.push(PathEl::MoveTo(p.into()));
        self
    }

    /// A straight line to `p`.
    pub fn line_to(mut self, p: impl Into<DVec2>) -> Self {
        self.els.push(PathEl::LineTo(p.into()));
        self
    }

    /// A quadratic bezier through `c` to `p`.
    pub fn quad_to(mut self, c: impl Into<DVec2>, p: impl Into<DVec2>) -> Self {
        self.els.push(PathEl::QuadTo(c.into(), p.into()));
        self
    }

    /// A cubic bezier through `c1` and `c2` to `p`.
    pub fn cubic_to(
        mut self,
        c1: impl Into<DVec2>,
        c2: impl Into<DVec2>,
        p: impl Into<DVec2>,
    ) -> Self {
        self.els.push(PathEl::CubicTo(c1.into(), c2.into(), p.into()));
        self
    }

    /// Closes the current subpath.
    pub fn close(mut self) -> Self {
        self.els.push(PathEl::Close);
        self
    }

    /// The path with every point moved by `f`.
    pub fn map(&self, f: impl Fn(DVec2) -> DVec2) -> Self {
        let els = self
            .els
            .iter()
            .map(|el| match *el {
                PathEl::MoveTo(p) => PathEl::MoveTo(f(p)),
                PathEl::LineTo(p) => PathEl::LineTo(f(p)),
                PathEl::QuadTo(c, p) => PathEl::QuadTo(f(c), f(p)),
                PathEl::CubicTo(c1, c2, p) => PathEl::CubicTo(f(c1), f(c2), f(p)),
                PathEl::Close => PathEl::Close,
            })
            .collect();
        Self { els }
    }

    /// Flattens the path into polylines whose segments stay within `tolerance` scene units
    /// of the curves. The number of segments per curve comes from Wang's formula, so the
    /// result depends only on the path and the tolerance.
    pub fn flatten(&self, tolerance: f64) -> Vec<Contour> {
        let tolerance = tolerance.max(1e-6);
        let mut contours = Vec::new();
        let mut current: Vec<DVec2> = Vec::new();
        let mut pen = DVec2::ZERO;
        let finish = |points: &mut Vec<DVec2>, closed: bool, out: &mut Vec<Contour>| {
            if points.len() > 1 {
                if closed && points.first() == points.last() {
                    points.pop();
                }
                out.push(Contour { points: std::mem::take(points), closed });
            } else {
                points.clear();
            }
        };
        for el in &self.els {
            match *el {
                PathEl::MoveTo(p) => {
                    finish(&mut current, false, &mut contours);
                    current.push(p);
                    pen = p;
                }
                PathEl::LineTo(p) => {
                    if current.is_empty() {
                        current.push(pen);
                    }
                    current.push(p);
                    pen = p;
                }
                PathEl::QuadTo(c, p) => {
                    if current.is_empty() {
                        current.push(pen);
                    }
                    let d = (pen - 2.0 * c + p).length();
                    let n = segments((d / (4.0 * tolerance)).sqrt());
                    for k in 1..=n {
                        let u = k as f64 / n as f64;
                        let v = 1.0 - u;
                        current.push(v * v * pen + 2.0 * v * u * c + u * u * p);
                    }
                    pen = p;
                }
                PathEl::CubicTo(c1, c2, p) => {
                    if current.is_empty() {
                        current.push(pen);
                    }
                    let d = (pen - 2.0 * c1 + c2).length().max((c1 - 2.0 * c2 + p).length());
                    let n = segments((0.75 * d / tolerance).sqrt());
                    for k in 1..=n {
                        let u = k as f64 / n as f64;
                        let v = 1.0 - u;
                        current.push(
                            v * v * v * pen
                                + 3.0 * v * v * u * c1
                                + 3.0 * v * u * u * c2
                                + u * u * u * p,
                        );
                    }
                    pen = p;
                }
                PathEl::Close => {
                    if let Some(&start) = current.first() {
                        pen = start;
                    }
                    finish(&mut current, true, &mut contours);
                }
            }
        }
        finish(&mut current, false, &mut contours);
        contours
    }
}

/// Segment count from Wang's formula, clamped so a degenerate curve still gets one segment
/// and a wild one cannot explode the point count.
fn segments(estimate: f64) -> usize {
    if estimate.is_finite() {
        (estimate.ceil() as usize).clamp(1, 512)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_segments_flatten_to_their_points() {
        let square = PathData::new()
            .move_to((0.0, 0.0))
            .line_to((10.0, 0.0))
            .line_to((10.0, 10.0))
            .line_to((0.0, 10.0))
            .close();
        let contours = square.flatten(0.1);
        assert_eq!(contours.len(), 1);
        assert!(contours[0].closed);
        assert_eq!(contours[0].points.len(), 4);
    }

    #[test]
    fn curves_stay_within_tolerance() {
        // A quarter circle approximated by a cubic: every flattened point lies on the
        // cubic, and the chord midpoints stay within the tolerance of it.
        let k = 0.552_284_749_831;
        let arc = PathData::new().move_to((100.0, 0.0)).cubic_to(
            (100.0, 100.0 * k),
            (100.0 * k, 100.0),
            (0.0, 100.0),
        );
        for tolerance in [1.0, 0.1, 0.01] {
            let points = &arc.flatten(tolerance)[0].points;
            for pair in points.windows(2) {
                let mid = (pair[0] + pair[1]) / 2.0;
                // The cubic is within 0.03 units of the true circle of radius 100.
                assert!((100.0 - mid.length()) <= tolerance + 0.03, "tolerance {tolerance}");
            }
        }
    }

    #[test]
    fn finer_tolerance_means_more_points() {
        let curve = PathData::new().move_to((0.0, 0.0)).quad_to((50.0, 80.0), (100.0, 0.0));
        let coarse = curve.flatten(1.0)[0].points.len();
        let fine = curve.flatten(0.01)[0].points.len();
        assert!(fine > coarse);
    }

    #[test]
    fn subpaths_become_separate_contours() {
        let two = PathData::new()
            .move_to((0.0, 0.0))
            .line_to((1.0, 0.0))
            .line_to((1.0, 1.0))
            .close()
            .move_to((5.0, 5.0))
            .line_to((6.0, 5.0));
        let contours = two.flatten(0.1);
        assert_eq!(contours.len(), 2);
        assert!(contours[0].closed && !contours[1].closed);
    }

    #[test]
    fn closing_on_the_start_point_does_not_duplicate_it() {
        let tri = PathData::new()
            .move_to((0.0, 0.0))
            .line_to((1.0, 0.0))
            .line_to((0.0, 1.0))
            .line_to((0.0, 0.0))
            .close();
        assert_eq!(tri.flatten(0.1)[0].points.len(), 3);
    }

    #[test]
    fn map_moves_every_point() {
        let p = PathData::new().move_to((1.0, 2.0)).cubic_to((3.0, 4.0), (5.0, 6.0), (7.0, 8.0));
        let moved = p.map(|q| q * 2.0);
        assert_eq!(
            moved.els[1],
            PathEl::CubicTo(DVec2::new(6.0, 8.0), DVec2::new(10.0, 12.0), DVec2::new(14.0, 16.0))
        );
    }

    #[test]
    fn empty_and_lone_points_produce_nothing() {
        assert!(PathData::new().flatten(0.1).is_empty());
        assert!(PathData::new().move_to((1.0, 1.0)).flatten(0.1).is_empty());
    }
}
