//! Preparing two shapes so one can morph into the other point by point.
//!
//! Both shapes are flattened into closed contours, contours are paired up, each pair is
//! resampled to the same number of points, given the same winding and rotated so their
//! start points line up. After that, morphing is a plain interpolation of point lists, so
//! evaluating a morph at any time costs one lerp per point.

use glam::DVec2;

use crate::shape::signed_area;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyline::resample;

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
}
