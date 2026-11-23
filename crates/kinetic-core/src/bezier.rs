//! Cubic bezier easing, as in CSS `cubic-bezier(x1, y1, x2, y2)`.

use serde::{Deserialize, Serialize};

/// An easing curve from (0, 0) to (1, 1) with two control points.
///
/// The x axis is time and the y axis is progress. Control point x values are clamped to
/// `[0, 1]` so time always moves forward; y values may leave that range to overshoot.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl CubicBezier {
    /// A curve through control points `(x1, y1)` and `(x2, y2)`.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1: x1.clamp(0.0, 1.0), y1, x2: x2.clamp(0.0, 1.0), y2 }
    }

    /// The control points, after clamping.
    pub fn points(self) -> [f64; 4] {
        [self.x1, self.y1, self.x2, self.y2]
    }

    fn component(a: f64, b: f64, s: f64) -> f64 {
        // Bernstein form with P0 = 0 and P3 = 1.
        let r = 1.0 - s;
        3.0 * r * r * s * a + 3.0 * r * s * s * b + s * s * s
    }

    fn slope(a: f64, b: f64, s: f64) -> f64 {
        let r = 1.0 - s;
        3.0 * r * r * a + 6.0 * r * s * (b - a) + 3.0 * s * s * (1.0 - b)
    }

    /// Curve parameter `s` whose x coordinate is `u`.
    fn solve(self, u: f64) -> f64 {
        // Newton's method converges in a few steps almost everywhere.
        let mut s = u;
        for _ in 0..8 {
            let error = Self::component(self.x1, self.x2, s) - u;
            if error.abs() < 1e-9 {
                return s;
            }
            let d = Self::slope(self.x1, self.x2, s);
            if d.abs() < 1e-6 {
                break;
            }
            s -= error / d;
            if !(0.0..=1.0).contains(&s) {
                break;
            }
        }
        // Flat spots or a step outside the curve: bisection always works because x is
        // monotonic in s, and a fixed number of steps keeps it deterministic.
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..60 {
            let mid = (lo + hi) / 2.0;
            if Self::component(self.x1, self.x2, mid) < u {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) / 2.0
    }

    /// Progress at normalised time `u`. Exact at 0 and 1; `u` is clamped to `[0, 1]`.
    pub fn apply(self, u: f64) -> f64 {
        let u = u.clamp(0.0, 1.0);
        if u == 0.0 || u == 1.0 {
            return u;
        }
        Self::component(self.y1, self.y2, self.solve(u))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line_is_linear() {
        let line = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
        for i in 0..=100 {
            let u = f64::from(i) / 100.0;
            assert!((line.apply(u) - u).abs() < 1e-9, "at {u}");
        }
    }

    #[test]
    fn endpoints_are_exact_and_input_is_clamped() {
        let c = CubicBezier::new(0.3, -0.6, 0.7, 1.6);
        assert_eq!(c.apply(0.0), 0.0);
        assert_eq!(c.apply(1.0), 1.0);
        assert_eq!(c.apply(-2.0), 0.0);
        assert_eq!(c.apply(3.0), 1.0);
    }

    #[test]
    fn apply_inverts_the_curve() {
        // Walk the curve by its own parameter and check apply recovers each point.
        for c in [
            CubicBezier::new(0.25, 0.1, 0.25, 1.0),
            CubicBezier::new(0.68, -0.6, 0.32, 1.6),
            CubicBezier::new(0.0, 0.0, 0.0, 1.0),
            CubicBezier::new(1.0, 0.0, 1.0, 1.0),
        ] {
            for i in 1..100 {
                let s = f64::from(i) / 100.0;
                let x = CubicBezier::component(c.x1, c.x2, s);
                let y = CubicBezier::component(c.y1, c.y2, s);
                assert!((c.apply(x) - y).abs() < 1e-6, "{c:?} at s = {s}");
            }
        }
    }

    #[test]
    fn control_x_is_clamped_so_time_moves_forward() {
        assert_eq!(CubicBezier::new(-1.0, 0.0, 2.0, 1.0).points(), [0.0, 0.0, 1.0, 1.0]);
    }
}
