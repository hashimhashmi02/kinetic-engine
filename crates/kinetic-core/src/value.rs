//! Values that properties hold and animate between.

use glam::DVec2;
use serde::{Deserialize, Serialize};

use crate::color::LinearRgba;
use crate::lerp::Lerp;

/// The value of one animatable property.
///
/// Every track keeps one variant for its whole life: a position is always a `Vec2`, an
/// opacity always a `Scalar`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// A plain number: rotation in radians, opacity in `[0, 1]`.
    Scalar(f64),
    /// A 2D vector: position, scale.
    Vec2(DVec2),
    /// A colour, linear and premultiplied.
    Color(LinearRgba),
}

impl Lerp for Value {
    /// Interpolates matching variants. Mismatched variants cannot occur on a well-formed
    /// track; if they do, the value switches at the end rather than producing nonsense.
    fn lerp(&self, other: &Self, u: f64) -> Self {
        match (self, other) {
            (Self::Scalar(a), Self::Scalar(b)) => Self::Scalar(a.lerp(b, u)),
            (Self::Vec2(a), Self::Vec2(b)) => Self::Vec2(Lerp::lerp(a, b, u)),
            (Self::Color(a), Self::Color(b)) => Self::Color(a.lerp(b, u)),
            _ if u >= 1.0 => other.clone(),
            _ => self.clone(),
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Scalar(v)
    }
}

impl From<DVec2> for Value {
    fn from(v: DVec2) -> Self {
        Self::Vec2(v)
    }
}

impl From<LinearRgba> for Value {
    fn from(v: LinearRgba) -> Self {
        Self::Color(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;

    #[test]
    fn each_variant_interpolates_like_its_contents() {
        assert_eq!(Value::Scalar(2.0).lerp(&Value::Scalar(4.0), 0.5), Value::Scalar(3.0));
        assert_eq!(
            Value::Vec2(DVec2::ZERO).lerp(&Value::Vec2(DVec2::new(10.0, -10.0)), 0.25),
            Value::Vec2(DVec2::new(2.5, -2.5))
        );
        assert_eq!(
            Value::Color(palette::BLACK).lerp(&Value::Color(palette::WHITE), 1.0),
            Value::Color(palette::WHITE)
        );
    }

    #[test]
    fn mismatched_variants_switch_at_the_end() {
        let (a, b) = (Value::Scalar(1.0), Value::Vec2(DVec2::ONE));
        assert_eq!(a.lerp(&b, 0.99), a);
        assert_eq!(a.lerp(&b, 1.0), b);
    }

    #[test]
    fn conversions_pick_the_right_variant() {
        assert_eq!(Value::from(0.5), Value::Scalar(0.5));
        assert_eq!(Value::from(DVec2::X), Value::Vec2(DVec2::X));
        assert_eq!(Value::from(palette::WHITE), Value::Color(palette::WHITE));
    }
}
