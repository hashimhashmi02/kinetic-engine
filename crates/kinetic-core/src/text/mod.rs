//! Text: fonts, shaping, layout and glyph outlines.
//!
//! Text is shaped while the scene is built, so the timeline only holds paths and every
//! machine draws the same glyphs. Poppins (Regular and Bold, SIL Open Font License) is
//! bundled; other fonts can be added from bytes compiled into a scene.

mod font;
mod layout;

pub use font::{Font, FontBook, FontError, Weight};
pub use layout::{layout, Align, Layout, PlacedGlyph, TextStyle};
