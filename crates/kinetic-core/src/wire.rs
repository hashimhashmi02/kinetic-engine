//! The wire format: how a compiled timeline crosses from a scene library to the editor.
//!
//! Layout: the four bytes `KNTC`, then [`WIRE_VERSION`] as a little-endian `u32`, then the
//! timeline as JSON. JSON keeps the payload readable when debugging; floats are written and
//! parsed exactly, so a timeline that crosses the wire evaluates to the same frames.

use std::fmt;

use crate::timeline::Timeline;

/// Marks the start of an encoded timeline.
pub const MAGIC: [u8; 4] = *b"KNTC";

/// Bumped whenever the encoded shape of a timeline changes. A scene built against a core
/// with a different version is refused with a clear message instead of misread.
pub const WIRE_VERSION: u32 = 1;

const HEADER: usize = MAGIC.len() + 4;

/// An encoded timeline could not be produced or read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// The bytes do not start with the Kinetic header.
    NotATimeline,
    /// The scene was built against a different version of the format.
    Version { found: u32, expected: u32 },
    /// The header was fine but the payload could not be encoded or decoded.
    Malformed(String),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotATimeline => write!(f, "the scene did not return a Kinetic timeline"),
            Self::Version { found, expected } => write!(
                f,
                "the scene was built for timeline format {found}, but this editor reads format \
                 {expected}; rebuild the scene against the same kinetic-core"
            ),
            Self::Malformed(why) => write!(f, "the timeline is malformed: {why}"),
        }
    }
}

impl std::error::Error for WireError {}

/// Encodes `timeline`. Fails only if it holds values JSON cannot carry, such as NaN.
pub fn encode(timeline: &Timeline) -> Result<Vec<u8>, WireError> {
    let mut bytes = Vec::with_capacity(4096);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&WIRE_VERSION.to_le_bytes());
    serde_json::to_writer(&mut bytes, timeline).map_err(|e| WireError::Malformed(e.to_string()))?;
    Ok(bytes)
}

/// Decodes a timeline written by [`encode`], checking the header and version first.
pub fn decode(bytes: &[u8]) -> Result<Timeline, WireError> {
    if bytes.len() < HEADER || bytes[..MAGIC.len()] != MAGIC {
        return Err(WireError::NotATimeline);
    }
    let mut version = [0u8; 4];
    version.copy_from_slice(&bytes[MAGIC.len()..HEADER]);
    let found = u32::from_le_bytes(version);
    if found != WIRE_VERSION {
        return Err(WireError::Version { found, expected: WIRE_VERSION });
    }
    serde_json::from_slice(&bytes[HEADER..]).map_err(|e| WireError::Malformed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;
    use crate::{spring, Circle, CubicBezier, Ease, Rect, Scene};

    fn sample() -> Timeline {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(40.0).fill(palette::blue()));
        let card = s.add(Rect::new(120.0, 80.0).opacity(0.0));
        s.play(dot.move_to((123.456_789, -0.1)), (0.7, Ease::OutBack));
        s.parallel([card.fade_to(1.0), card.rotate_to(0.3)], (0.5, CubicBezier::EASE));
        s.play(dot.scale_to(1.5), spring().stiffness(220.0).damping(9.0));
        s.build()
    }

    #[test]
    fn round_trip_is_exact() {
        let timeline = sample();
        let bytes = encode(&timeline).expect("encode");
        assert_eq!(&bytes[..4], b"KNTC");
        assert_eq!(decode(&bytes).expect("decode"), timeline);
    }

    #[test]
    fn rejects_bytes_without_the_header() {
        assert_eq!(decode(b"{}"), Err(WireError::NotATimeline));
        assert_eq!(decode(b"KNT"), Err(WireError::NotATimeline));
        assert_eq!(decode(b"XXXX\x01\0\0\0{}"), Err(WireError::NotATimeline));
    }

    #[test]
    fn rejects_another_version_with_both_numbers_in_the_message() {
        let mut bytes = encode(&sample()).expect("encode");
        bytes[4..8].copy_from_slice(&99u32.to_le_bytes());
        let error = decode(&bytes).expect_err("must refuse");
        assert_eq!(error, WireError::Version { found: 99, expected: WIRE_VERSION });
        let message = error.to_string();
        assert!(message.contains("99") && message.contains(&WIRE_VERSION.to_string()));
    }

    #[test]
    fn rejects_a_damaged_payload() {
        let mut bytes = encode(&sample()).expect("encode");
        bytes.truncate(bytes.len() / 2);
        assert!(matches!(decode(&bytes), Err(WireError::Malformed(_))));
    }

    #[test]
    fn nan_cannot_be_encoded_silently() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(1.0));
        s.play(dot.move_to((f64::NAN, 0.0)), 1.0);
        let timeline = s.build();
        // JSON writes NaN as null, which cannot be read back as a number.
        let decoded = encode(&timeline).and_then(|bytes| decode(&bytes));
        assert!(decoded.is_err());
    }
}
