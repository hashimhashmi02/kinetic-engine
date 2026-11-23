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
}
