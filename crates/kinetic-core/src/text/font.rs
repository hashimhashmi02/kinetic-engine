//! Font faces and a small font database.

use std::fmt;
use std::sync::OnceLock;

use rustybuzz::ttf_parser;

/// How heavy a font is, on the usual 100 (thin) to 900 (black) scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Weight(pub u16);

impl Weight {
    /// 400.
    pub const REGULAR: Weight = Weight(400);
    /// 700.
    pub const BOLD: Weight = Weight(700);
}

/// The bytes given were not a font this crate can read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontError(String);

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not read font: {}", self.0)
    }
}

impl std::error::Error for FontError {}

/// One face of a TrueType or OpenType font, with its family name and weight.
///
/// The bytes are `'static` so that scenes can bundle fonts with `include_bytes!` and
/// shaping never reads a file.
#[derive(Clone)]
pub struct Font {
    data: &'static [u8],
    index: u32,
    family: String,
    weight: Weight,
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Font({} {})", self.family, self.weight.0)
    }
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && (std::ptr::eq(self.data, other.data) || self.data == other.data)
    }
}

static POPPINS_REGULAR: &[u8] = include_bytes!("../../fonts/Poppins-Regular.ttf");
static POPPINS_BOLD: &[u8] = include_bytes!("../../fonts/Poppins-Bold.ttf");

impl Font {
    /// Reads face `index` (0 for a plain font file) from `data`.
    pub fn from_static(data: &'static [u8], index: u32) -> Result<Font, FontError> {
        let face = ttf_parser::Face::parse(data, index).map_err(|e| FontError(e.to_string()))?;
        let name = |id| {
            face.names()
                .into_iter()
                .filter(|n| n.name_id == id && n.is_unicode())
                .find_map(|n| n.to_string())
        };
        let family = name(ttf_parser::name_id::TYPOGRAPHIC_FAMILY)
            .or_else(|| name(ttf_parser::name_id::FAMILY))
            .ok_or_else(|| FontError("the font has no family name".into()))?;
        Ok(Font { data, index, family, weight: Weight(face.weight().to_number()) })
    }

    /// Poppins Regular, bundled with Kinetic.
    pub fn regular() -> Font {
        static FONT: OnceLock<Font> = OnceLock::new();
        // The bundled bytes are fixed and parse (see the tests), so this cannot fail.
        FONT.get_or_init(|| Font::from_static(POPPINS_REGULAR, 0).expect("bundled font")).clone()
    }

    /// Poppins Bold, bundled with Kinetic.
    pub fn bold() -> Font {
        static FONT: OnceLock<Font> = OnceLock::new();
        // The bundled bytes are fixed and parse (see the tests), so this cannot fail.
        FONT.get_or_init(|| Font::from_static(POPPINS_BOLD, 0).expect("bundled font")).clone()
    }

    /// The family name, such as "Poppins".
    pub fn family(&self) -> &str {
        &self.family
    }

    /// The weight.
    pub fn weight(&self) -> Weight {
        self.weight
    }

    /// Whether the font has a glyph for `c`.
    pub fn has_glyph(&self, c: char) -> bool {
        self.face().and_then(|f| f.glyph_index(c)).is_some()
    }

    /// The parsed face. Only `None` if the bytes changed after `from_static` checked them,
    /// which `'static` bytes cannot.
    pub(crate) fn face(&self) -> Option<rustybuzz::Face<'static>> {
        rustybuzz::Face::from_slice(self.data, self.index)
    }
}

impl Default for Font {
    fn default() -> Self {
        Font::regular()
    }
}

/// A set of fonts to pick from by family and weight.
#[derive(Debug, Clone, Default)]
pub struct FontBook {
    fonts: Vec<Font>,
}

impl FontBook {
    /// An empty book.
    pub fn new() -> Self {
        Self::default()
    }

    /// The fonts bundled with Kinetic: Poppins Regular and Bold.
    pub fn bundled() -> Self {
        FontBook { fonts: vec![Font::regular(), Font::bold()] }
    }

    /// Adds a font. A font already in the book is not added twice.
    pub fn add(&mut self, font: Font) {
        if !self.fonts.contains(&font) {
            self.fonts.push(font);
        }
    }

    /// The face of `family` (ignoring case) closest to `weight`; between two equally close
    /// faces the heavier one wins. `None` if the family is not in the book.
    pub fn find(&self, family: &str, weight: Weight) -> Option<&Font> {
        self.fonts
            .iter()
            .filter(|f| f.family.eq_ignore_ascii_case(family))
            .min_by_key(|f| (f.weight.0.abs_diff(weight.0), std::cmp::Reverse(f.weight)))
    }

    /// The family names in the book, sorted, each once.
    pub fn families(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.fonts.iter().map(|f| f.family.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fonts_are_poppins() {
        let (regular, bold) = (Font::regular(), Font::bold());
        assert_eq!((regular.family(), regular.weight()), ("Poppins", Weight::REGULAR));
        assert_eq!((bold.family(), bold.weight()), ("Poppins", Weight::BOLD));
        assert_ne!(regular, bold);
        assert_eq!(Font::default(), regular);
    }

    #[test]
    fn bundled_fonts_cover_latin_and_devanagari() {
        for font in [Font::regular(), Font::bold()] {
            assert!("Hello, World! 0123456789".chars().all(|c| font.has_glyph(c)));
            assert!("नमस्ते".chars().all(|c| font.has_glyph(c)));
            assert!(!font.has_glyph('漢'));
        }
    }

    #[test]
    fn garbage_is_not_a_font() {
        assert!(Font::from_static(b"definitely not a font", 0).is_err());
        assert!(Font::from_static(POPPINS_REGULAR, 3).is_err());
    }

    #[test]
    fn book_finds_the_closest_weight() {
        let book = FontBook::bundled();
        assert_eq!(book.find("poppins", Weight(300)), Some(&Font::regular()));
        assert_eq!(book.find("Poppins", Weight(600)), Some(&Font::bold()));
        assert_eq!(book.find("Poppins", Weight(550)), Some(&Font::bold()), "ties go heavier");
        assert_eq!(book.find("Inter", Weight::REGULAR), None);
        assert_eq!(book.families(), vec!["Poppins"]);
    }

    #[test]
    fn adding_a_font_twice_keeps_one() {
        let mut book = FontBook::new();
        book.add(Font::regular());
        book.add(Font::regular());
        assert_eq!(book.families(), vec!["Poppins"]);
        assert_eq!(book.find("Poppins", Weight::BOLD), Some(&Font::regular()));
    }
}
