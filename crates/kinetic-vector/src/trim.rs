//! Cutting a path down to part of its length, for draw-on effects.

use kurbo::{BezPath, ParamCurve, ParamCurveArclen, PathEl, PathSeg};

/// Error allowed when measuring curve lengths, in scene units.
const ARCLEN_ACCURACY: f64 = 1e-4;

/// One subpath: its original elements, starting with a `MoveTo`, and its segments.
struct Subpath {
    els: Vec<PathEl>,
    segs: Vec<(PathSeg, f64)>,
}

impl Subpath {
    fn length(&self) -> f64 {
        self.segs.iter().map(|(_, len)| len).sum()
    }
}

fn subpaths(path: &BezPath) -> Vec<Subpath> {
    let mut groups: Vec<Vec<PathEl>> = Vec::new();
    for &el in path.elements() {
        match (el, groups.last_mut()) {
            (PathEl::MoveTo(_), _) | (_, None) => groups.push(vec![el]),
            (_, Some(group)) => group.push(el),
        }
    }
    groups
        .into_iter()
        .filter(|els| matches!(els.first(), Some(PathEl::MoveTo(_))))
        .map(|els| {
            let segs = kurbo::segments(els.iter().copied())
                .map(|seg| (seg, seg.arclen(ARCLEN_ACCURACY)))
                .collect();
            Subpath { els, segs }
        })
        .collect()
}

fn push_seg(out: &mut BezPath, seg: PathSeg) {
    match seg {
        PathSeg::Line(l) => out.line_to(l.p1),
        PathSeg::Quad(q) => out.quad_to(q.p1, q.p2),
        PathSeg::Cubic(c) => out.curve_to(c.p1, c.p2, c.p3),
    }
}

/// The part of `path` from `start` to `end`, given as fractions of its total length with
/// its subpaths measured one after another in order.
///
/// Curves stay curves; they are cut at the exact arc length. A subpath that lies wholly
/// inside the range is kept as it was, closed or not; a subpath that is cut becomes open.
/// `(0, 1)` returns the path unchanged and `end <= start` returns an empty path.
pub fn trim(path: &BezPath, start: f64, end: f64) -> BezPath {
    let (start, end) = (start.clamp(0.0, 1.0), end.clamp(0.0, 1.0));
    if start <= 0.0 && end >= 1.0 {
        return path.clone();
    }
    let mut out = BezPath::new();
    if end <= start {
        return out;
    }
    let subpaths = subpaths(path);
    let total: f64 = subpaths.iter().map(Subpath::length).sum();
    if total <= 0.0 {
        return out;
    }
    let (from, to) = (start * total, end * total);
    let mut offset = 0.0;
    for sub in &subpaths {
        let (s0, s1) = (offset, offset + sub.length());
        offset = s1;
        if s1 <= from || s0 >= to {
            continue;
        }
        if from <= s0 && s1 <= to {
            out.extend(sub.els.iter().copied());
            continue;
        }
        let mut pen_down = false;
        let mut g0 = s0;
        for &(seg, len) in &sub.segs {
            let g1 = g0 + len;
            let (lo, hi) = (from.max(g0), to.min(g1));
            if hi > lo {
                let t0 = if lo <= g0 { 0.0 } else { seg.inv_arclen(lo - g0, ARCLEN_ACCURACY) };
                let t1 = if hi >= g1 { 1.0 } else { seg.inv_arclen(hi - g0, ARCLEN_ACCURACY) };
                let piece = seg.subsegment(t0..t1);
                if !pen_down {
                    out.move_to(piece.start());
                    pen_down = true;
                }
                push_seg(&mut out, piece);
            }
            g0 = g1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use kinetic_core::DVec2;

    fn length(path: &BezPath) -> f64 {
        path.segments().map(|s| s.arclen(1e-6)).sum()
    }

    fn closes(path: &BezPath) -> usize {
        path.elements().iter().filter(|el| matches!(el, PathEl::ClosePath)).count()
    }

    fn two_lines() -> BezPath {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((100.0, 0.0));
        p.move_to((0.0, 10.0));
        p.line_to((100.0, 10.0));
        p
    }

    #[test]
    fn full_range_keeps_the_path() {
        let circle = shape::circle(DVec2::ZERO, 20.0);
        assert_eq!(trim(&circle, 0.0, 1.0), circle);
    }

    #[test]
    fn empty_range_draws_nothing() {
        let circle = shape::circle(DVec2::ZERO, 20.0);
        assert!(trim(&circle, 0.5, 0.5).is_empty());
        assert!(trim(&circle, 0.7, 0.2).is_empty());
        assert!(trim(&BezPath::new(), 0.2, 0.7).is_empty());
    }

    #[test]
    fn half_a_line_ends_half_way() {
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((100.0, 0.0));
        let half = trim(&line, 0.0, 0.5);
        assert_eq!(
            half.elements(),
            &[PathEl::MoveTo((0.0, 0.0).into()), PathEl::LineTo((50.0, 0.0).into())]
        );
    }

    #[test]
    fn curves_are_cut_by_arc_length() {
        let circle = shape::circle(DVec2::ZERO, 50.0);
        let whole = length(&circle);
        for (a, b) in [(0.0, 0.25), (0.1, 0.6), (0.3, 1.0)] {
            let part = trim(&circle, a, b);
            assert!((length(&part) - whole * (b - a)).abs() < 1e-2, "{a}..{b}");
            assert_eq!(closes(&part), 0, "a cut circle is open");
        }
    }

    #[test]
    fn range_spans_subpaths_in_order() {
        let part = trim(&two_lines(), 0.25, 0.75);
        assert_eq!(
            part.elements(),
            &[
                PathEl::MoveTo((50.0, 0.0).into()),
                PathEl::LineTo((100.0, 0.0).into()),
                PathEl::MoveTo((0.0, 10.0).into()),
                PathEl::LineTo((50.0, 10.0).into()),
            ]
        );
    }

    #[test]
    fn whole_subpaths_inside_the_range_stay_closed() {
        let mut ring = shape::circle(DVec2::ZERO, 40.0);
        ring.extend(shape::circle(DVec2::ZERO, 20.0));
        // The outer circle is two thirds of the length, so it is wholly inside.
        let part = trim(&ring, 0.0, 0.8);
        assert_eq!(closes(&part), 1);
        assert!((length(&part) - length(&ring) * 0.8).abs() < 1e-2);
    }

    #[test]
    fn trimming_is_deterministic() {
        let circle = shape::circle(DVec2::new(3.0, 4.0), 33.0);
        assert_eq!(trim(&circle, 0.17, 0.83), trim(&circle, 0.17, 0.83));
    }
}
