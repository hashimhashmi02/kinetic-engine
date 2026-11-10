//! Easing functions.
//!
//! An easing maps normalised time `u` in `[0, 1]` to progress. Every easing returns exactly
//! `0.0` at `u = 0` and exactly `1.0` at `u = 1`; inputs outside `[0, 1]` are clamped.
//! Progress itself may leave `[0, 1]` in between (see [`out_back`]).

/// The signature shared by all easing functions.
pub type EaseFn = fn(f64) -> f64;

/// Constant speed.
pub fn linear(u: f64) -> f64 {
    u.clamp(0.0, 1.0)
}

/// Starts fast and decelerates to a stop.
pub fn out_cubic(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    1.0 - (1.0 - u).powi(3)
}

/// Accelerates from rest, then decelerates to a stop.
pub fn in_out_cubic(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    if u < 0.5 {
        4.0 * u * u * u
    } else {
        1.0 - (2.0 - 2.0 * u).powi(3) / 2.0
    }
}

/// Overshoots the target by about 10% and settles back.
pub fn out_back(u: f64) -> f64 {
    const C1: f64 = 1.701_58;
    const C3: f64 = C1 + 1.0;
    let u = u.clamp(0.0, 1.0);
    if u == 1.0 {
        return 1.0;
    }
    let v = u - 1.0;
    1.0 + C3 * v * v * v + C1 * v * v
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [(&str, EaseFn); 4] = [
        ("linear", linear),
        ("out_cubic", out_cubic),
        ("in_out_cubic", in_out_cubic),
        ("out_back", out_back),
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
        // None of these easings has a slope above 5, so neighbouring samples 1/10000 apart
        // can differ by at most 5e-4.
        let n = 10_000;
        for (name, f) in ALL {
            for i in 0..n {
                let (a, b) = (f(i as f64 / n as f64), f((i + 1) as f64 / n as f64));
                assert!((b - a).abs() <= 5e-4, "{name} jumps between samples {i} and {}", i + 1);
            }
        }
    }

    #[test]
    fn monotonic_easings_never_go_backwards() {
        let n = 1_000;
        for (name, f) in [ALL[0], ALL[1], ALL[2]] {
            for i in 0..n {
                assert!(f((i + 1) as f64 / n as f64) >= f(i as f64 / n as f64), "{name} at {i}");
            }
        }
    }

    #[test]
    fn known_values() {
        assert_eq!(in_out_cubic(0.5), 0.5);
        assert_eq!(out_cubic(0.5), 0.875);
        assert_eq!(linear(0.25), 0.25);
    }

    #[test]
    fn out_back_overshoots_by_about_ten_percent() {
        let peak = (0..=1000).map(|i| out_back(f64::from(i) / 1000.0)).fold(f64::MIN, f64::max);
        assert!((peak - 1.1).abs() < 0.005, "peak was {peak}");
    }
}
