//! Easing functions.
//!
//! An easing maps normalised time `u` in `[0, 1]` to progress. Every easing returns exactly
//! `0.0` at `u = 0` and exactly `1.0` at `u = 1`; inputs outside `[0, 1]` are clamped.
//! Progress itself may leave `[0, 1]` in between (see [`out_back`]).
//!
//! Each family is written once as an ease-in curve; the ease-out and ease-in-out versions
//! are derived from it by mirroring, so the three always agree.

/// The signature shared by all easing functions.
pub type EaseFn = fn(f64) -> f64;

/// Clamps `u` to `[0, 1]` and pins the endpoints, so rounding inside `f` can never make an
/// easing miss exactly 0 or 1.
fn finish(u: f64, f: impl Fn(f64) -> f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    if u == 0.0 {
        0.0
    } else if u == 1.0 {
        1.0
    } else {
        f(u)
    }
}

/// The ease-out matching an ease-in: the same curve played backwards and upside down.
fn out_of(f: impl Fn(f64) -> f64, u: f64) -> f64 {
    1.0 - f(1.0 - u)
}

/// The ease-in-out matching an ease-in: the in curve for the first half, the out curve for
/// the second, each squeezed into half the time.
fn in_out_of(f: impl Fn(f64) -> f64, u: f64) -> f64 {
    if u < 0.5 {
        f(2.0 * u) / 2.0
    } else {
        1.0 - f(2.0 - 2.0 * u) / 2.0
    }
}

fn cubic(u: f64) -> f64 {
    u * u * u
}

const BACK_C1: f64 = 1.701_58;
const BACK_C3: f64 = BACK_C1 + 1.0;

fn back(u: f64) -> f64 {
    BACK_C3 * u * u * u - BACK_C1 * u * u
}

/// Constant speed.
pub fn linear(u: f64) -> f64 {
    finish(u, |u| u)
}

/// Accelerates from rest.
pub fn in_cubic(u: f64) -> f64 {
    finish(u, cubic)
}

/// Starts fast and decelerates to a stop.
pub fn out_cubic(u: f64) -> f64 {
    finish(u, |u| out_of(cubic, u))
}

/// Accelerates from rest, then decelerates to a stop.
pub fn in_out_cubic(u: f64) -> f64 {
    finish(u, |u| in_out_of(cubic, u))
}

/// Overshoots the target by about 10% and settles back.
pub fn out_back(u: f64) -> f64 {
    finish(u, |u| out_of(back, u))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every easing, by name.
    const ALL: &[(&str, EaseFn)] = &[
        ("linear", linear),
        ("in_cubic", in_cubic),
        ("out_cubic", out_cubic),
        ("in_out_cubic", in_out_cubic),
        ("out_back", out_back),
    ];

    /// Easings that never move backwards.
    const MONOTONIC: &[(&str, EaseFn)] = &[
        ("linear", linear),
        ("in_cubic", in_cubic),
        ("out_cubic", out_cubic),
        ("in_out_cubic", in_out_cubic),
    ];

    #[test]
    fn endpoints_are_exact() {
        for (name, f) in ALL {
            assert_eq!(f(0.0), 0.0, "{name} at 0");
            assert_eq!(f(1.0), 1.0, "{name} at 1");
        }
    }

    #[test]
    fn input_is_clamped() {
        for (name, f) in ALL {
            assert_eq!(f(-0.5), 0.0, "{name} below range");
            assert_eq!(f(1.5), 1.0, "{name} above range");
        }
    }

    #[test]
    fn continuous_on_a_fine_grid() {
        // No easing here is steeper than 30, so samples 1/10000 apart differ by at most 3e-3.
        let n = 10_000;
        for (name, f) in ALL {
            for i in 0..n {
                let (a, b) = (f(i as f64 / n as f64), f((i + 1) as f64 / n as f64));
                assert!((b - a).abs() <= 3e-3, "{name} jumps between samples {i} and {}", i + 1);
            }
        }
    }

    #[test]
    fn monotonic_easings_never_go_backwards() {
        let n = 1_000;
        for (name, f) in MONOTONIC {
            for i in 0..n {
                assert!(f((i + 1) as f64 / n as f64) >= f(i as f64 / n as f64), "{name} at {i}");
            }
        }
    }

    #[test]
    fn known_values() {
        assert_eq!(in_out_cubic(0.5), 0.5);
        assert_eq!(in_cubic(0.5), 0.125);
        assert_eq!(out_cubic(0.5), 0.875);
        assert_eq!(linear(0.25), 0.25);
    }

    #[test]
    fn out_back_overshoots_by_about_ten_percent() {
        let peak = (0..=1000).map(|i| out_back(f64::from(i) / 1000.0)).fold(f64::MIN, f64::max);
        assert!((peak - 1.1).abs() < 0.005, "peak was {peak}");
    }
}
