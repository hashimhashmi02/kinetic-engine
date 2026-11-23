//! Springs solved in closed form.
//!
//! A spring moves from 0 to 1 the way a mass on a damped spring settles onto its rest
//! point. Because the motion is solved analytically rather than simulated step by step,
//! its value at any time can be computed directly, so springs scrub, rewind and export
//! exactly like easings. See `docs/ARCHITECTURE.md` for the formulas.

use serde::{Deserialize, Serialize};

/// Damping ratios this close to 1 use the critically damped solution, which avoids
/// dividing by a vanishing frequency.
const CRITICAL_BAND: f64 = 1e-6;

/// A damped spring from 0 to 1.
///
/// Build one with [`spring()`] and the setter methods:
///
/// ```
/// use kinetic_core::spring::spring;
/// let s = spring().stiffness(180.0).damping(12.0);
/// assert_eq!(s.value(0.0), 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    mass: f64,
    stiffness: f64,
    damping: f64,
    velocity: f64,
}

/// A spring with mass 1, stiffness 180, damping 12 and no initial velocity: lively, with
/// one clear overshoot.
pub fn spring() -> Spring {
    Spring { mass: 1.0, stiffness: 180.0, damping: 12.0, velocity: 0.0 }
}

/// The three ways a spring can settle.
enum Solution {
    /// Oscillates while it decays: `a` is the decay rate, `wd` the ringing frequency and
    /// `b` the sine coefficient.
    Under { a: f64, wd: f64, b: f64 },
    /// The fastest approach with no overshoot.
    Critical { w0: f64, c: f64 },
    /// Creeps in on two exponentials with rates `r1` (slow) and `r2` (fast).
    Over { r1: f64, r2: f64 },
}

impl Spring {
    /// Mass of the moving body. Heavier springs are slower and swing further. Clamped to
    /// a small positive value.
    pub fn mass(mut self, mass: f64) -> Self {
        self.mass = mass.max(1e-6);
        self
    }

    /// Stiffness: how hard the spring pulls towards its target. Clamped to a small
    /// positive value.
    pub fn stiffness(mut self, stiffness: f64) -> Self {
        self.stiffness = stiffness.max(1e-6);
        self
    }

    /// Damping: how quickly motion dies away. Zero would ring forever; negative values are
    /// treated as zero.
    pub fn damping(mut self, damping: f64) -> Self {
        self.damping = damping.max(0.0);
        self
    }

    /// Initial velocity, in units of the full distance per second. Positive values set off
    /// towards the target.
    pub fn velocity(mut self, velocity: f64) -> Self {
        self.velocity = velocity;
        self
    }

    /// Undamped angular frequency, `sqrt(k / m)`.
    fn natural_frequency(&self) -> f64 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Below 1 the spring overshoots, at 1 it is critically damped, above 1 it creeps.
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.stiffness * self.mass).sqrt())
    }

    fn solution(&self) -> Solution {
        let w0 = self.natural_frequency();
        let z = self.damping_ratio();
        let v0 = self.velocity;
        if (z - 1.0).abs() < CRITICAL_BAND {
            Solution::Critical { w0, c: w0 - v0 }
        } else if z < 1.0 {
            let a = z * w0;
            let wd = w0 * (1.0 - z * z).sqrt();
            Solution::Under { a, wd, b: (a - v0) / wd }
        } else {
            let s = (z * z - 1.0).sqrt();
            Solution::Over { r1: -w0 * (z - s), r2: -w0 * (z + s) }
        }
    }

    /// Position at time `t` seconds after release. May overshoot 1. Not snapped: see
    /// [`Spring::progress`] for the version with a finite duration.
    pub fn value(&self, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        let v0 = self.velocity;
        match self.solution() {
            Solution::Under { a, wd, b } => {
                1.0 - (-a * t).exp() * ((wd * t).cos() + b * (wd * t).sin())
            }
            Solution::Critical { w0, c } => 1.0 - (-w0 * t).exp() * (1.0 + c * t),
            Solution::Over { r1, r2 } => {
                1.0 + ((r2 + v0) * (r1 * t).exp() - (r1 + v0) * (r2 * t).exp()) / (r1 - r2)
            }
        }
    }

    /// Velocity at time `t` seconds after release, in units of the full distance per
    /// second. Used to hand motion from one spring to the next without a jolt.
    pub fn velocity_at(&self, t: f64) -> f64 {
        let v0 = self.velocity;
        let t = t.max(0.0);
        match self.solution() {
            Solution::Under { a, wd, b } => {
                (-a * t).exp() * (v0 * (wd * t).cos() + (a * b + wd) * (wd * t).sin())
            }
            Solution::Critical { w0, c } => (-w0 * t).exp() * (v0 + w0 * c * t),
            Solution::Over { r1, r2 } => {
                (r1 * (r2 + v0) * (r1 * t).exp() - r2 * (r1 + v0) * (r2 * t).exp()) / (r1 - r2)
            }
        }
    }
}

/// How close to the target counts as settled, as a share of the full distance.
pub const SETTLE_TOLERANCE: f64 = 1e-3;

/// Longest a spring may take, in seconds. An undamped spring would ring forever; it is
/// cut off here and snapped to its target.
pub const MAX_SETTLE: f64 = 60.0;

impl Spring {
    /// Time in seconds after which the spring stays within [`SETTLE_TOLERANCE`] of its
    /// target. This is the spring's duration on a timeline.
    ///
    /// It comes from the decay envelope of the closed-form solution, so it is an upper
    /// bound: the spring is guaranteed to be settled by then, and is usually settled a
    /// little before.
    pub fn settle_time(&self) -> f64 {
        // Aim slightly inside the band: at the exact bound the deviation equals the
        // tolerance, and rounding could leave the spring a hair outside it.
        let eps = SETTLE_TOLERANCE * 0.99;
        let v0 = self.velocity;
        let t = match self.solution() {
            Solution::Under { a, b, .. } => {
                let amplitude = (1.0 + b * b).sqrt();
                if a <= 0.0 {
                    f64::INFINITY
                } else {
                    (amplitude / eps).ln() / a
                }
            }
            Solution::Critical { w0, c } => {
                // |x - 1| <= e^(-w0 t) (1 + |c| t); the polynomial factor makes this
                // implicit, and a few fixed-point steps from the pure exponential solve it.
                let mut t = (1.0 / eps).ln() / w0;
                for _ in 0..32 {
                    t = ((1.0 + c.abs() * t) / eps).ln() / w0;
                }
                t
            }
            Solution::Over { r1, r2 } => {
                // Both terms decay at least as fast as the slow root r1.
                let weight = ((r2 + v0).abs() + (r1 + v0).abs()) / (r1 - r2);
                (weight / eps).ln() / -r1
            }
        };
        t.clamp(0.0, MAX_SETTLE)
    }

    /// Progress at time `t`, reaching exactly 1 at [`Spring::settle_time`] and holding
    /// there.
    ///
    /// The raw spring is still a tiny distance (under [`SETTLE_TOLERANCE`]) from its target
    /// when it settles. Rather than snap that gap shut, a linear correction closes it over
    /// the whole duration, so the motion has no step at the end.
    pub fn progress(&self, t: f64) -> f64 {
        let end = self.settle_time();
        if t >= end {
            1.0
        } else if t <= 0.0 {
            0.0
        } else {
            self.value(t) + (t / end) * (1.0 - self.value(end))
        }
    }
}

impl Spring {
    /// Soft and slow, a slight overshoot.
    pub const GENTLE: Self = Self { mass: 1.0, stiffness: 120.0, damping: 14.0, velocity: 0.0 };
    /// Bouncy, two or three visible swings.
    pub const WOBBLY: Self = Self { mass: 1.0, stiffness: 180.0, damping: 12.0, velocity: 0.0 };
    /// Firm, a small overshoot.
    pub const STIFF: Self = Self { mass: 1.0, stiffness: 210.0, damping: 20.0, velocity: 0.0 };
    /// Heavy and unhurried, no overshoot.
    pub const SLOW: Self = Self { mass: 1.0, stiffness: 280.0, damping: 60.0, velocity: 0.0 };
    /// Quick to arrive, barely any overshoot.
    pub const SNAPPY: Self = Self { mass: 1.0, stiffness: 400.0, damping: 30.0, velocity: 0.0 };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One spring in each regime, with and without initial velocity.
    fn springs() -> Vec<Spring> {
        let mut out = Vec::new();
        for damping in [4.0, 12.0, 20.0, 60.0] {
            for velocity in [0.0, 6.0, -3.0] {
                out.push(spring().stiffness(100.0).damping(damping).velocity(velocity));
            }
        }
        out
    }

    #[test]
    fn regimes_follow_the_damping_ratio() {
        let base = spring().stiffness(100.0);
        assert!(base.damping(4.0).damping_ratio() < 1.0);
        assert_eq!(base.damping(20.0).damping_ratio(), 1.0);
        assert!(base.damping(60.0).damping_ratio() > 1.0);
    }

    #[test]
    fn starts_at_zero_with_the_given_velocity() {
        for s in springs() {
            assert_eq!(s.value(0.0), 0.0, "{s:?}");
            assert!((s.velocity_at(0.0) - s.velocity).abs() < 1e-9, "{s:?}");
            // The analytic derivative agrees with a numerical one at the start.
            let h = 1e-6;
            let numeric = s.value(h) / h;
            assert!((numeric - s.velocity).abs() < 1e-3 * (1.0 + s.velocity.abs()), "{s:?}");
        }
    }

    #[test]
    fn approaches_one() {
        for s in springs() {
            assert!((s.value(10.0) - 1.0).abs() < 1e-6, "{s:?}");
        }
    }

    #[test]
    fn velocity_is_the_derivative_of_value() {
        let h = 1e-6;
        for s in springs() {
            for t in [0.05, 0.2, 0.5, 1.3] {
                let numeric = (s.value(t + h) - s.value(t - h)) / (2.0 * h);
                assert!((numeric - s.velocity_at(t)).abs() < 1e-4, "{s:?} at {t}");
            }
        }
    }

    #[test]
    fn underdamped_overshoots_and_overdamped_does_not() {
        let peak =
            |s: Spring| (0..2000).map(|i| s.value(f64::from(i) / 1000.0)).fold(0.0, f64::max);
        assert!(peak(spring().stiffness(100.0).damping(4.0)) > 1.1);
        assert!(peak(spring().stiffness(100.0).damping(60.0)) <= 1.0);
    }

    #[test]
    fn setters_reject_impossible_values() {
        let s = spring().mass(-1.0).stiffness(0.0).damping(-5.0);
        assert!(s.mass > 0.0 && s.stiffness > 0.0);
        assert_eq!(s.damping, 0.0);
        assert!(s.value(0.5).is_finite());
    }

    #[test]
    fn settled_after_settle_time() {
        for s in springs() {
            let end = s.settle_time();
            assert!(end > 0.0 && end < MAX_SETTLE, "{s:?} settles at {end}");
            for i in 0..400 {
                let t = end + f64::from(i) * 0.01;
                assert!((s.value(t) - 1.0).abs() <= SETTLE_TOLERANCE, "{s:?} at {t}");
            }
        }
    }

    #[test]
    fn settle_time_is_not_wildly_pessimistic() {
        // The envelope bound should land within a couple of ringing periods of the real
        // moment the spring enters the tolerance band for good.
        for s in springs() {
            let end = s.settle_time();
            let last_outside = (0..=(end * 1000.0) as u32)
                .map(|i| f64::from(i) / 1000.0)
                .rfind(|&t| (s.value(t) - 1.0).abs() > SETTLE_TOLERANCE)
                .unwrap_or(0.0);
            assert!(end - last_outside < 0.8, "{s:?}: bound {end}, real {last_outside}");
        }
    }

    #[test]
    fn progress_snaps_to_one_exactly() {
        let s = spring();
        assert_eq!(s.progress(s.settle_time()), 1.0);
        assert_eq!(s.progress(100.0), 1.0);
        assert_eq!(s.progress(0.0), 0.0);
    }

    #[test]
    fn undamped_spring_is_cut_off() {
        assert_eq!(spring().damping(0.0).settle_time(), MAX_SETTLE);
    }

    #[test]
    fn presets_behave_as_described() {
        let peak =
            |s: Spring| (0..3000).map(|i| s.value(f64::from(i) / 1000.0)).fold(0.0, f64::max);
        assert!(peak(Spring::WOBBLY) > peak(Spring::GENTLE));
        assert!(peak(Spring::SLOW) <= 1.0, "slow must not overshoot");
        assert!(Spring::SNAPPY.settle_time() < Spring::GENTLE.settle_time());
        assert_eq!(spring(), Spring::WOBBLY);
    }

    /// Integrates `m x'' + c x' + k (x - 1) = 0` with classic fourth-order Runge-Kutta.
    fn integrate(s: Spring, until: f64) -> f64 {
        let accel = |x: f64, v: f64| -(s.damping * v + s.stiffness * (x - 1.0)) / s.mass;
        let (mut x, mut v) = (0.0, s.velocity);
        let steps = (until / 1e-4).round() as u32;
        let dt = until / f64::from(steps);
        for _ in 0..steps {
            let (k1x, k1v) = (v, accel(x, v));
            let (k2x, k2v) = (v + k1v * dt / 2.0, accel(x + k1x * dt / 2.0, v + k1v * dt / 2.0));
            let (k3x, k3v) = (v + k2v * dt / 2.0, accel(x + k2x * dt / 2.0, v + k2v * dt / 2.0));
            let (k4x, k4v) = (v + k3v * dt, accel(x + k3x * dt, v + k3v * dt));
            x += dt / 6.0 * (k1x + 2.0 * k2x + 2.0 * k3x + k4x);
            v += dt / 6.0 * (k1v + 2.0 * k2v + 2.0 * k3v + k4v);
        }
        x
    }

    #[test]
    fn closed_form_matches_numerical_integration() {
        for s in springs() {
            for t in [0.1, 0.35, 0.8, 1.5] {
                let (exact, numeric) = (s.value(t), integrate(s, t));
                assert!((exact - numeric).abs() < 1e-6, "{s:?} at {t}: {exact} vs {numeric}");
            }
        }
    }

    #[test]
    fn continuous_across_critical_damping() {
        // Just below, at and just above critical damping the three formulas must agree.
        let critical = 2.0 * 100.0_f64.sqrt();
        for velocity in [0.0, 4.0] {
            let at = |c: f64| spring().stiffness(100.0).damping(c).velocity(velocity);
            for t in [0.05, 0.2, 0.6] {
                let (below, exact, above) =
                    (at(critical * (1.0 - 1e-4)), at(critical), at(critical * (1.0 + 1e-4)));
                assert!((below.value(t) - exact.value(t)).abs() < 1e-3);
                assert!((above.value(t) - exact.value(t)).abs() < 1e-3);
            }
        }
    }

    #[test]
    fn progress_has_no_step_at_the_end() {
        for s in springs() {
            let end = s.settle_time();
            assert!((s.progress(end - 1e-9) - 1.0).abs() < 1e-6, "{s:?}");
            assert!((s.progress(end * 0.5) - s.value(end * 0.5)).abs() <= SETTLE_TOLERANCE);
        }
    }
}
