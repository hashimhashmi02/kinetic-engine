//! Colours.
//!
//! Inside the engine a colour is always linear-light, premultiplied RGBA. sRGB only appears
//! at the API edge (constructors such as [`LinearRgba::from_srgb8`]) and at output
//! ([`LinearRgba::to_srgba8`]).

/// Linear-light RGBA with premultiplied alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Decodes one sRGB-encoded channel in `[0, 1]` to linear light.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Encodes one linear-light channel in `[0, 1]` to sRGB.
pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

impl LinearRgba {
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    /// Builds a colour from linear, already premultiplied components.
    pub const fn from_premultiplied(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Builds a colour from linear, straight (not premultiplied) components.
    pub fn from_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r: r * a, g: g * a, b: b * a, a }
    }

    /// Opaque colour from 8-bit sRGB components.
    pub fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_srgba8(r, g, b, 255)
    }

    /// Colour from 8-bit sRGB components with straight alpha.
    pub fn from_srgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let decode = |c: u8| srgb_to_linear(f32::from(c) / 255.0);
        Self::from_linear(decode(r), decode(g), decode(b), f32::from(a) / 255.0)
    }

    /// Opaque colour from a `0xRRGGBB` sRGB hex value.
    pub fn from_hex(rgb: u32) -> Self {
        let [_, r, g, b] = rgb.to_be_bytes();
        Self::from_srgb8(r, g, b)
    }

    /// The same colour with its opacity multiplied by `opacity`.
    pub fn with_opacity(self, opacity: f32) -> Self {
        Self { r: self.r * opacity, g: self.g * opacity, b: self.b * opacity, a: self.a * opacity }
    }

    /// Clamps every channel to `[0, 1]` and colour channels to at most alpha, which keeps the
    /// value a valid premultiplied colour after an overshooting interpolation.
    pub fn clamped(self) -> Self {
        let a = self.a.clamp(0.0, 1.0);
        Self { r: self.r.clamp(0.0, a), g: self.g.clamp(0.0, a), b: self.b.clamp(0.0, a), a }
    }

    /// Converts to 8-bit sRGB with straight alpha, as stored in PNG files.
    pub fn to_srgba8(self) -> [u8; 4] {
        let c = self.clamped();
        if c.a <= 0.0 {
            return [0, 0, 0, 0];
        }
        let encode = |v: f32| (linear_to_srgb(v / c.a) * 255.0).round() as u8;
        [encode(c.r), encode(c.g), encode(c.b), (c.a * 255.0).round() as u8]
    }

    /// Components as an array, in `[r, g, b, a]` order.
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// A small palette for examples and tests.
pub mod palette {
    use super::LinearRgba;

    /// Opaque white.
    pub const WHITE: LinearRgba = LinearRgba::from_premultiplied(1.0, 1.0, 1.0, 1.0);
    /// Opaque black.
    pub const BLACK: LinearRgba = LinearRgba::from_premultiplied(0.0, 0.0, 0.0, 1.0);

    /// `#3b82f6`
    pub fn blue() -> LinearRgba {
        LinearRgba::from_hex(0x3b82f6)
    }
    /// `#ef4444`
    pub fn red() -> LinearRgba {
        LinearRgba::from_hex(0xef4444)
    }
    /// `#22c55e`
    pub fn green() -> LinearRgba {
        LinearRgba::from_hex(0x22c55e)
    }
    /// `#f59e0b`
    pub fn amber() -> LinearRgba {
        LinearRgba::from_hex(0xf59e0b)
    }
    /// `#0f172a`, a dark slate that works as a default background.
    pub fn ink() -> LinearRgba {
        LinearRgba::from_hex(0x0f172a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_round_trips_every_8_bit_value() {
        for v in 0..=255u8 {
            let c = LinearRgba::from_srgb8(v, v, v);
            assert_eq!(c.to_srgba8(), [v, v, v, 255]);
        }
    }

    #[test]
    fn transfer_function_endpoints() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
        assert!((linear_to_srgb(1.0) - 1.0).abs() < 1e-6);
        // Middle grey: sRGB 0.5 is about 21.4% linear.
        assert!((srgb_to_linear(0.5) - 0.214).abs() < 1e-3);
    }

    #[test]
    fn constructors_premultiply() {
        let c = LinearRgba::from_linear(1.0, 0.5, 0.0, 0.5);
        assert_eq!(c, LinearRgba::from_premultiplied(0.5, 0.25, 0.0, 0.5));
    }

    #[test]
    fn half_transparent_round_trip_keeps_colour() {
        let c = LinearRgba::from_srgba8(200, 100, 50, 128);
        assert_eq!(c.to_srgba8(), [200, 100, 50, 128]);
    }

    #[test]
    fn hex_matches_bytes() {
        assert_eq!(LinearRgba::from_hex(0x3b82f6), LinearRgba::from_srgb8(0x3b, 0x82, 0xf6));
    }

    #[test]
    fn transparent_encodes_to_zero() {
        assert_eq!(LinearRgba::TRANSPARENT.to_srgba8(), [0, 0, 0, 0]);
    }

    #[test]
    fn clamp_restores_premultiplied_invariant() {
        let c = LinearRgba::from_premultiplied(1.2, -0.1, 0.4, 0.5).clamped();
        assert_eq!(c, LinearRgba::from_premultiplied(0.5, 0.0, 0.4, 0.5));
    }
}
