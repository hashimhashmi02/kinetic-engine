//! Measuring and resampling polylines by arc length.

use glam::DVec2;

/// Total length of a polyline, including the closing segment when `closed`.
pub fn length(points: &[DVec2], closed: bool) -> f64 {
    let open: f64 = points.windows(2).map(|w| (w[1] - w[0]).length()).sum();
    match (closed, points.first(), points.last()) {
        (true, Some(&first), Some(&last)) => open + (first - last).length(),
        _ => open,
    }
}

/// `count` points spread evenly along the polyline by arc length.
///
/// A closed polyline is sampled all the way round starting at its first point, without
/// repeating it at the end. An open one gets both of its end points. Corners between
/// samples are cut; callers pick `count` high enough for the detail they need.
pub fn resample(points: &[DVec2], closed: bool, count: usize) -> Vec<DVec2> {
    let Some(&first) = points.first() else { return Vec::new() };
    if count == 0 {
        return Vec::new();
    }
    let mut path: Vec<DVec2> = points.to_vec();
    if closed {
        path.push(first);
    }
    let total = length(points, closed);
    if total <= 0.0 || path.len() < 2 {
        return vec![first; count];
    }
    let step = if closed {
        total / count as f64
    } else if count > 1 {
        total / (count - 1) as f64
    } else {
        0.0
    };

    let mut out = Vec::with_capacity(count);
    let (mut segment, mut walked) = (0, 0.0);
    for k in 0..count {
        let target = (k as f64 * step).min(total);
        // Advance to the segment containing the target distance.
        loop {
            let span = (path[segment + 1] - path[segment]).length();
            if walked + span >= target || segment + 2 >= path.len() {
                let u = if span > 0.0 { ((target - walked) / span).clamp(0.0, 1.0) } else { 0.0 };
                out.push(path[segment].lerp(path[segment + 1], u));
                break;
            }
            walked += span;
            segment += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<DVec2> {
        [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)].map(DVec2::from).to_vec()
    }

    #[test]
    fn length_counts_the_closing_segment_only_when_closed() {
        assert_eq!(length(&square(), true), 40.0);
        assert_eq!(length(&square(), false), 30.0);
        assert_eq!(length(&[], true), 0.0);
    }

    #[test]
    fn closed_square_resamples_to_corners_and_midpoints() {
        let points = resample(&square(), true, 8);
        let expected = [
            (0.0, 0.0),
            (5.0, 0.0),
            (10.0, 0.0),
            (10.0, 5.0),
            (10.0, 10.0),
            (5.0, 10.0),
            (0.0, 10.0),
            (0.0, 5.0),
        ];
        for (got, want) in points.iter().zip(expected) {
            assert!((*got - DVec2::from(want)).length() < 1e-9, "{got} vs {want:?}");
        }
    }

    #[test]
    fn open_polyline_keeps_both_ends_and_even_spacing() {
        let uneven = [(0.0, 0.0), (1.0, 0.0), (9.0, 0.0), (10.0, 0.0)].map(DVec2::from);
        let points = resample(&uneven, false, 3);
        assert_eq!(points, [(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)].map(DVec2::from));
    }

    #[test]
    fn spacing_is_even_on_a_long_closed_polyline() {
        let points = resample(&square(), true, 37);
        let step = 40.0 / 37.0;
        for pair in points.windows(2) {
            // Samples that straddle a corner are closer in a straight line than along it.
            let d = (pair[1] - pair[0]).length();
            assert!(d <= step + 1e-9 && d >= step / 2.0_f64.sqrt() - 1e-9, "{d}");
        }
        assert_eq!(points.len(), 37);
    }

    #[test]
    fn degenerate_inputs() {
        assert!(resample(&[], true, 4).is_empty());
        assert!(resample(&square(), true, 0).is_empty());
        let point = [DVec2::new(3.0, 3.0)];
        assert_eq!(resample(&point, true, 3), vec![DVec2::new(3.0, 3.0); 3]);
        assert_eq!(resample(&square(), false, 1), vec![DVec2::ZERO]);
    }
}
