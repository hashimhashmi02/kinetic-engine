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

fn quad(u: f64) -> f64 {
    u * u
}

fn quart(u: f64) -> f64 {
    u * u * u * u
}

fn quint(u: f64) -> f64 {
    u * u * u * u * u
}

/// Accelerates from rest, gently.
pub fn in_quad(u: f64) -> f64 {
    finish(u, quad)
}

/// Decelerates to a stop, gently.
pub fn out_quad(u: f64) -> f64 {
    finish(u, |u| out_of(quad, u))
}

/// Gentle acceleration, then gentle deceleration.
pub fn in_out_quad(u: f64) -> f64 {
    finish(u, |u| in_out_of(quad, u))
}

/// Accelerates from rest, sharply.
pub fn in_quart(u: f64) -> f64 {
    finish(u, quart)
}

/// Decelerates to a stop, sharply.
pub fn out_quart(u: f64) -> f64 {
    finish(u, |u| out_of(quart, u))
}

/// Sharp acceleration, then sharp deceleration.
pub fn in_out_quart(u: f64) -> f64 {
    finish(u, |u| in_out_of(quart, u))
}

/// Accelerates from rest, very sharply.
pub fn in_quint(u: f64) -> f64 {
    finish(u, quint)
}

/// Decelerates to a stop, very sharply.
pub fn out_quint(u: f64) -> f64 {
    finish(u, |u| out_of(quint, u))
}

/// Very sharp acceleration, then very sharp deceleration.
pub fn in_out_quint(u: f64) -> f64 {
    finish(u, |u| in_out_of(quint, u))
}

fn sine(u: f64) -> f64 {
    1.0 - (u * std::f64::consts::FRAC_PI_2).cos()
}

fn expo(u: f64) -> f64 {
    (10.0 * u - 10.0).exp2()
}

fn circ(u: f64) -> f64 {
    1.0 - (1.0 - u * u).max(0.0).sqrt()
}

/// Accelerates from rest along a quarter sine wave; the softest start.
pub fn in_sine(u: f64) -> f64 {
    finish(u, sine)
}

/// Decelerates along a quarter sine wave; the softest stop.
pub fn out_sine(u: f64) -> f64 {
    finish(u, |u| out_of(sine, u))
}

/// Half a cosine wave: soft at both ends.
pub fn in_out_sine(u: f64) -> f64 {
    finish(u, |u| in_out_of(sine, u))
}

/// Almost still, then launches; doubles its speed every tenth of the way.
pub fn in_expo(u: f64) -> f64 {
    finish(u, expo)
}

/// Arrives almost at once, then creeps the last few percent.
pub fn out_expo(u: f64) -> f64 {
    finish(u, |u| out_of(expo, u))
}

/// Exponential in, exponential out: a snap through the middle.
pub fn in_out_expo(u: f64) -> f64 {
    finish(u, |u| in_out_of(expo, u))
}

/// Follows a quarter circle: slow start, then nearly vertical.
pub fn in_circ(u: f64) -> f64 {
    finish(u, circ)
}

/// Nearly vertical start, then a slow quarter-circle stop.
pub fn out_circ(u: f64) -> f64 {
    finish(u, |u| out_of(circ, u))
}

/// Quarter circle in, quarter circle out.
pub fn in_out_circ(u: f64) -> f64 {
    finish(u, |u| in_out_of(circ, u))
}

/// Pulls back about 10% before setting off.
pub fn in_back(u: f64) -> f64 {
    finish(u, back)
}

/// Pulls back, sets off, overshoots and settles. The overshoot is a little larger than
/// [`out_back`]'s so it still reads at half the duration.
pub fn in_out_back(u: f64) -> f64 {
    const C2: f64 = BACK_C1 * 1.525;
    finish(u, |u| {
        if u < 0.5 {
            let v = 2.0 * u;
            v * v * ((C2 + 1.0) * v - C2) / 2.0
        } else {
            let v = 2.0 * u - 2.0;
            (v * v * ((C2 + 1.0) * v + C2) + 2.0) / 2.0
        }
    })
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
        ("in_quad", in_quad),
        ("out_quad", out_quad),
        ("in_out_quad", in_out_quad),
        ("in_quart", in_quart),
        ("out_quart", out_quart),
        ("in_out_quart", in_out_quart),
        ("in_quint", in_quint),
        ("out_quint", out_quint),
        ("in_out_quint", in_out_quint),
        ("in_sine", in_sine),
        ("out_sine", out_sine),
        ("in_out_sine", in_out_sine),
        ("in_expo", in_expo),
        ("out_expo", out_expo),
        ("in_out_expo", in_out_expo),
        ("in_circ", in_circ),
        ("out_circ", out_circ),
        ("in_out_circ", in_out_circ),
        ("in_back", in_back),
        ("in_out_back", in_out_back),
    ];

    /// Easings that never move backwards.
    const MONOTONIC: &[(&str, EaseFn)] = &[
        ("linear", linear),
        ("in_cubic", in_cubic),
        ("out_cubic", out_cubic),
        ("in_out_cubic", in_out_cubic),
        ("in_quad", in_quad),
        ("out_quad", out_quad),
        ("in_out_quad", in_out_quad),
        ("in_quart", in_quart),
        ("out_quart", out_quart),
        ("in_out_quart", in_out_quart),
        ("in_quint", in_quint),
        ("out_quint", out_quint),
        ("in_out_quint", in_out_quint),
        ("in_sine", in_sine),
        ("out_sine", out_sine),
        ("in_out_sine", in_out_sine),
        ("in_expo", in_expo),
        ("out_expo", out_expo),
        ("in_out_expo", in_out_expo),
        ("in_circ", in_circ),
        ("out_circ", out_circ),
        ("in_out_circ", in_out_circ),
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
        // Circ curves are the exception: they go vertical at their ends like a square root,
        // continuous but unbounded in slope, so a step of h can move them by sqrt(2h).
        let n = 10_000;
        for (name, f) in ALL {
            let bound = if name.contains("circ") { 0.015 } else { 3e-3 };
            for i in 0..n {
                let (a, b) = (f(i as f64 / n as f64), f((i + 1) as f64 / n as f64));
                assert!((b - a).abs() <= bound, "{name} jumps between samples {i} and {}", i + 1);
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

    #[test]
    fn polynomial_families_hit_their_powers_of_a_half() {
        assert_eq!(in_quad(0.5), 0.25);
        assert_eq!(in_quart(0.5), 0.0625);
        assert_eq!(in_quint(0.5), 0.03125);
        assert_eq!(out_quad(0.5), 0.75);
        for f in [in_out_quad, in_out_quart, in_out_quint] {
            assert_eq!(f(0.5), 0.5);
        }
    }

    #[test]
    fn sine_expo_and_circ_known_values() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(close(in_sine(0.5), 1.0 - std::f64::consts::FRAC_1_SQRT_2));
        assert!(close(out_sine(0.5), std::f64::consts::FRAC_1_SQRT_2));
        assert!(close(in_expo(0.5), 0.03125));
        assert!(close(in_circ(0.5), 1.0 - 0.75_f64.sqrt()));
        for f in [in_out_sine, in_out_expo, in_out_circ] {
            assert!(close(f(0.5), 0.5));
        }
    }

    #[test]
    fn back_easings_undershoot_and_overshoot() {
        let min = (0..=1000).map(|i| in_back(f64::from(i) / 1000.0)).fold(f64::MAX, f64::min);
        assert!((min + 0.1).abs() < 0.005, "in_back dipped to {min}");
        let samples: Vec<f64> = (0..=1000).map(|i| in_out_back(f64::from(i) / 1000.0)).collect();
        assert!(samples.iter().any(|&v| v < 0.0) && samples.iter().any(|&v| v > 1.0));
        assert_eq!(in_out_back(0.5), 0.5);
    }
}
