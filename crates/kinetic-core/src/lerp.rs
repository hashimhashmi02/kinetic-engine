//! Linear interpolation between values of the same type.

use glam::DVec2;

use crate::color::LinearRgba;

/// Interpolation between two values.
///
/// `u` is normally in `[0, 1]` but may leave that range: easings such as `out_back` and
/// underdamped springs overshoot, and implementations extrapolate rather than clamp unless
/// the type has an invariant to protect (see [`LinearRgba`]).
///
/// Implementations return exactly `self` at `u == 0.0` and exactly `other` at `u == 1.0`.
pub trait Lerp: Sized {
    /// The value `u` of the way from `self` to `other`.
    fn lerp(&self, other: &Self, u: f64) -> Self;
}

impl Lerp for f64 {
    fn lerp(&self, other: &Self, u: f64) -> Self {
        // `a + (b - a) * u` does not hit `b` exactly at u = 1 for all inputs, so the
        // endpoints are handled explicitly.
        if u == 0.0 {
            *self
        } else if u == 1.0 {
            *other
        } else {
            self + (other - self) * u
        }
    }
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, u: f64) -> Self {
        f64::from(*self).lerp(&f64::from(*other), u) as f32
    }
}

impl Lerp for DVec2 {
    fn lerp(&self, other: &Self, u: f64) -> Self {
        DVec2::new(self.x.lerp(&other.x, u), self.y.lerp(&other.y, u))
    }
}

impl Lerp for LinearRgba {
    /// Interpolates in linear premultiplied space, then clamps so that an overshooting `u`
    /// cannot produce an invalid colour.
    fn lerp(&self, other: &Self, u: f64) -> Self {
        LinearRgba {
            r: self.r.lerp(&other.r, u),
            g: self.g.lerp(&other.g, u),
            b: self.b.lerp(&other.b, u),
            a: self.a.lerp(&other.a, u),
        }
        .clamped()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;

    #[test]
    fn scalar_endpoints_are_exact() {
        let (a, b) = (0.1_f64, 0.7_f64);
        assert_eq!(a.lerp(&b, 0.0), a);
        assert_eq!(a.lerp(&b, 1.0), b);
        assert_eq!(1e16_f64.lerp(&1.0, 1.0), 1.0);
    }

    #[test]
    fn scalar_midpoint_and_extrapolation() {
        assert_eq!(2.0_f64.lerp(&4.0, 0.5), 3.0);
        assert_eq!(2.0_f64.lerp(&4.0, 1.5), 5.0);
        assert_eq!(2.0_f64.lerp(&4.0, -0.5), 1.0);
    }

    #[test]
    fn vec2_interpolates_per_component() {
        let a = DVec2::new(0.0, 10.0);
        let b = DVec2::new(4.0, -10.0);
        assert_eq!(a.lerp(&b, 0.25), DVec2::new(1.0, 5.0));
        assert_eq!(a.lerp(&b, 1.0), b);
    }

    #[test]
    fn colour_fades_through_premultiplied_space() {
        let half = palette::WHITE.lerp(&LinearRgba::TRANSPARENT, 0.5);
        assert_eq!(half, LinearRgba::from_premultiplied(0.5, 0.5, 0.5, 0.5));
    }

    #[test]
    fn colour_overshoot_stays_valid() {
        let c = palette::BLACK.lerp(&palette::WHITE, 1.3);
        assert_eq!(c, palette::WHITE);
        let c = palette::WHITE.lerp(&LinearRgba::TRANSPARENT, 1.3);
        assert_eq!(c, LinearRgba::TRANSPARENT);
    }
}
