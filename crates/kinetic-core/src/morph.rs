//! Preparing two shapes so one can morph into the other point by point.
//!
//! Both shapes are flattened into closed contours, contours are paired up, each pair is
//! resampled to the same number of points, given the same winding and rotated so their
//! start points line up. After that, morphing is a plain interpolation of point lists, so
//! evaluating a morph at any time costs one lerp per point.

use glam::DVec2;

use crate::polyline::{length, resample};
use crate::shape::{signed_area, Shape};

/// Curves are flattened this close to their true shape, in scene units, before resampling.
const FLATTEN_TOLERANCE: f64 = 0.05;
/// Target distance between resampled points, in scene units.
const SPACING: f64 = 2.0;
/// Point counts per contour stay within these bounds.
const MIN_POINTS: usize = 32;
const MAX_POINTS: usize = 1024;

/// Gives `b` the same winding as `a`, reversing it if needed. Contours with no area (a
/// point, or a line walked out and back) have no winding and are left alone.
pub fn match_winding(a: &[DVec2], b: &mut [DVec2]) {
    let (area_a, area_b) = (signed_area(a), signed_area(b));
    if area_a.abs() > 1e-9 && area_b.abs() > 1e-9 && area_a.signum() != area_b.signum() {
        // Reverse while keeping the start point first.
        b[1..].reverse();
    }
}

/// Rotates `b` so that its points line up with `a`'s with the least total travel, measured
/// as the sum of squared distances. Both must have the same length.
pub fn align_start(a: &[DVec2], b: &mut [DVec2]) {
    let n = a.len();
    if n == 0 || b.len() != n {
        return;
    }
    let mut best = (0, f64::INFINITY);
    for shift in 0..n {
        let cost: f64 = (0..n).map(|i| a[i].distance_squared(b[(i + shift) % n])).sum();
        // Strictly less: ties keep the smallest shift, so the result is deterministic.
        if cost < best.1 {
            best = (shift, cost);
        }
    }
    b.rotate_left(best.0);
}

/// One closed contour of a shape, with what pairing needs to know about it.
struct Piece {
    points: Vec<DVec2>,
    area: f64,
    centroid: DVec2,
    length: f64,
}

/// The shape's contours, closed. An open contour (a line) is walked out and back so it
/// can morph like any other. Contours that have collapsed to a point, as the leftovers of
/// an earlier morph do, are dropped.
fn pieces(shape: &Shape) -> Vec<Piece> {
    let contours: Vec<(Vec<DVec2>, bool)> = match shape {
        Shape::Outline { contours } => contours.iter().map(|c| (c.clone(), true)).collect(),
        other => other
            .to_path()
            .flatten(FLATTEN_TOLERANCE)
            .into_iter()
            .map(|c| (c.points, c.closed))
            .collect(),
    };
    contours
        .into_iter()
        .filter_map(|(mut points, closed)| {
            if !closed && points.len() > 2 {
                let back: Vec<DVec2> = points[1..points.len() - 1].iter().rev().copied().collect();
                points.extend(back);
            }
            let length = length(&points, true);
            if points.is_empty() || length < 1e-9 {
                return None;
            }
            let centroid = points.iter().copied().sum::<DVec2>() / points.len() as f64;
            Some(Piece { area: signed_area(&points), centroid, length, points })
        })
        .collect()
}

/// Points to resample a contour pair to: about one every [`SPACING`] units.
fn point_count(length: f64) -> usize {
    ((length / SPACING).ceil() as usize).clamp(MIN_POINTS, MAX_POINTS)
}

/// The unused contour of `others` that best matches `piece`: close centroids and similar
/// size. With `same_winding`, only contours of the same orientation qualify, so outlines
/// pair with outlines and holes with holes.
fn best_partner(
    piece: &Piece,
    others: &[Piece],
    used: &[bool],
    same_winding: bool,
) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (j, other) in others.iter().enumerate() {
        if used[j] || (same_winding && piece.area.signum() != other.area.signum()) {
            continue;
        }
        let size = piece.area.abs().sqrt() - other.area.abs().sqrt();
        let cost = piece.centroid.distance_squared(other.centroid) + size * size;
        // Strictly less: ties go to the earlier contour.
        if best.is_none_or(|(_, c)| cost < c) {
            best = Some((j, cost));
        }
    }
    best.map(|(j, _)| j)
}

/// Prepares `from` and `to` for morphing. The two outlines returned have the same number
/// of contours, each pair has the same number of points, the same winding and aligned start
/// points, so interpolating them point by point morphs one shape into the other.
///
/// Contours are paired largest first. A contour with no partner grows from, or shrinks to,
/// its own centroid.
pub fn prepare(from: &Shape, to: &Shape) -> (Shape, Shape) {
    let (a, b) = (pieces(from), pieces(to));
    let mut order: Vec<usize> = (0..a.len()).collect();
    // Stable, so equal areas keep their original order.
    order.sort_by(|&i, &j| a[j].area.abs().total_cmp(&a[i].area.abs()));

    let mut used = vec![false; b.len()];
    let mut pairs: Vec<(Option<usize>, Option<usize>)> = Vec::new();
    for i in order {
        let partner =
            best_partner(&a[i], &b, &used, true).or_else(|| best_partner(&a[i], &b, &used, false));
        if let Some(j) = partner {
            used[j] = true;
        }
        pairs.push((Some(i), partner));
    }
    pairs.extend((0..b.len()).filter(|&j| !used[j]).map(|j| (None, Some(j))));

    let (mut out_a, mut out_b) = (Vec::new(), Vec::new());
    for pair in pairs {
        let (pa, pb) = match pair {
            (Some(i), Some(j)) => {
                let n = point_count(a[i].length.max(b[j].length));
                let pa = resample(&a[i].points, true, n);
                let mut pb = resample(&b[j].points, true, n);
                match_winding(&pa, &mut pb);
                align_start(&pa, &mut pb);
                (pa, pb)
            }
            (Some(i), None) => {
                let n = point_count(a[i].length);
                (resample(&a[i].points, true, n), vec![a[i].centroid; n])
            }
            (None, Some(j)) => {
                let n = point_count(b[j].length);
                (vec![b[j].centroid; n], resample(&b[j].points, true, n))
            }
            (None, None) => continue,
        };
        out_a.push(pa);
        out_b.push(pb);
    }
    (Shape::Outline { contours: out_a }, Shape::Outline { contours: out_b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::PathData;

    fn square(size: f64) -> Vec<DVec2> {
        let h = size / 2.0;
        // Counter-clockwise from the top-left corner.
        [(-h, h), (-h, -h), (h, -h), (h, h)].map(DVec2::from).to_vec()
    }

    #[test]
    fn opposite_winding_is_reversed_keeping_the_start() {
        let a = square(2.0);
        let mut b: Vec<DVec2> = square(4.0).into_iter().rev().collect();
        let start = b[0];
        assert!(signed_area(&b) < 0.0);
        match_winding(&a, &mut b);
        assert!(signed_area(&b) > 0.0);
        assert_eq!(b[0], start);
    }

    #[test]
    fn same_winding_is_left_alone() {
        let a = square(2.0);
        let mut b = square(4.0);
        let before = b.clone();
        match_winding(&a, &mut b);
        assert_eq!(b, before);
    }

    #[test]
    fn contours_without_area_are_left_alone() {
        let a = square(2.0);
        let mut point = vec![DVec2::ONE; 4];
        match_winding(&a, &mut point);
        assert_eq!(point, vec![DVec2::ONE; 4]);
    }

    #[test]
    fn alignment_undoes_a_rotation() {
        let a = resample(&square(10.0), true, 40);
        let mut b = a.clone();
        b.rotate_left(13);
        align_start(&a, &mut b);
        assert_eq!(b, a);
    }

    #[test]
    fn alignment_matches_nearby_points_between_different_shapes() {
        // A larger square rotated by a quarter of its points should line up corner to
        // corner with the smaller one.
        let a = resample(&square(10.0), true, 40);
        let mut b = resample(&square(20.0), true, 40);
        b.rotate_left(10);
        align_start(&a, &mut b);
        assert!((b[0] - DVec2::new(-10.0, 10.0)).length() < 1e-9, "{}", b[0]);
    }

    #[test]
    fn mismatched_lengths_are_ignored() {
        let a = square(2.0);
        let mut b = square(2.0)[..3].to_vec();
        let before = b.clone();
        align_start(&a, &mut b);
        assert_eq!(b, before);
    }

    fn contours(shape: &Shape) -> &Vec<Vec<DVec2>> {
        match shape {
            Shape::Outline { contours } => contours,
            other => panic!("expected an outline, got {other:?}"),
        }
    }

    /// An annulus: a counter-clockwise outer circle with a clockwise hole.
    fn ring() -> Shape {
        let circle = |r: f64, clockwise: bool| -> Vec<DVec2> {
            let mut points: Vec<DVec2> = (0..64)
                .map(|k| {
                    let t = f64::from(k) / 64.0 * std::f64::consts::TAU;
                    DVec2::new(t.cos(), t.sin()) * r
                })
                .collect();
            if clockwise {
                points.reverse();
            }
            points
        };
        Shape::Outline { contours: vec![circle(50.0, false), circle(20.0, true)] }
    }

    #[test]
    fn circle_to_rect_is_one_matched_pair() {
        let (a, b) = prepare(
            &Shape::Circle { radius: 40.0 },
            &Shape::Rect { size: DVec2::new(120.0, 60.0) },
        );
        let (a, b) = (contours(&a), contours(&b));
        assert_eq!((a.len(), b.len()), (1, 1));
        assert_eq!(a[0].len(), b[0].len());
        assert!(signed_area(&a[0]) > 0.0 && signed_area(&b[0]) > 0.0);
        assert!(a[0].iter().all(|p| (p.length() - 40.0).abs() < 0.05));
        // The rectangle keeps its area through resampling.
        assert!((signed_area(&b[0]) - 7200.0).abs() < 1.0);
    }

    #[test]
    fn ring_to_disc_shrinks_the_hole_to_a_point() {
        let (a, b) = prepare(&ring(), &Shape::Circle { radius: 50.0 });
        let (a, b) = (contours(&a), contours(&b));
        assert_eq!((a.len(), b.len()), (2, 2));
        // The outer circle pairs with the disc; the hole has no partner and collapses.
        assert!(signed_area(&a[0]) > 0.0 && signed_area(&b[0]) > 0.0);
        assert!(signed_area(&a[1]) < 0.0);
        assert!(b[1].iter().all(|p| *p == b[1][0]), "hole should shrink to one point");
        assert!(b[1][0].length() < 1e-9, "at the hole's centre");
    }

    #[test]
    fn disc_to_ring_grows_the_hole_from_a_point() {
        let (a, b) = prepare(&Shape::Circle { radius: 50.0 }, &ring());
        let (a, b) = (contours(&a), contours(&b));
        assert_eq!(a.len(), 2);
        assert!(a[1].iter().all(|p| *p == a[1][0]));
        assert!(signed_area(&b[1]) < 0.0);
    }

    #[test]
    fn a_line_morphs_like_a_closed_contour() {
        let (a, b) = prepare(
            &Shape::Line { from: DVec2::new(-50.0, 0.0), to: DVec2::new(50.0, 0.0) },
            &Shape::Rect { size: DVec2::new(40.0, 40.0) },
        );
        let (a, b) = (contours(&a), contours(&b));
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].len(), b[0].len());
        assert!(a[0].iter().all(|p| p.y.abs() < 1e-9 && p.x.abs() <= 50.0 + 1e-9));
    }

    #[test]
    fn chained_morphs_drop_collapsed_leftovers() {
        let (_, disc) = prepare(&ring(), &Shape::Circle { radius: 50.0 });
        assert_eq!(contours(&disc).len(), 2);
        let (a, _) = prepare(&disc, &Shape::Rect { size: DVec2::splat(40.0) });
        assert_eq!(contours(&a).len(), 1, "the collapsed hole is not carried forward");
    }

    #[test]
    fn preparation_is_deterministic() {
        let star = Shape::Star { points: 5, outer: 60.0, inner: 25.0 };
        let blob = Shape::Path(
            PathData::new()
                .move_to((0.0, 50.0))
                .cubic_to((-60.0, 60.0), (-70.0, -40.0), (0.0, -45.0))
                .cubic_to((50.0, -50.0), (70.0, 40.0), (0.0, 50.0))
                .close(),
        );
        assert_eq!(prepare(&star, &blob), prepare(&star, &blob));
    }

    #[test]
    fn point_counts_follow_length_within_bounds() {
        assert_eq!(point_count(1.0), MIN_POINTS);
        assert_eq!(point_count(200.0), 100);
        assert_eq!(point_count(1e9), MAX_POINTS);
    }
}
