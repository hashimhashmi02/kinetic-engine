//! Shaping text into positioned glyphs.

use glam::DVec2;
use serde::{Deserialize, Serialize};

use super::font::Font;

/// Where each line sits relative to the text's anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Align {
    /// Lines start at the anchor.
    Left,
    /// Lines are centred on the anchor.
    #[default]
    Center,
    /// Lines end at the anchor.
    Right,
}

/// How text is set.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// The face to shape with.
    pub font: Font,
    /// Font size in scene units: the height of the em square.
    pub size: f64,
    /// Horizontal alignment of every line.
    pub align: Align,
    /// Distance from one baseline to the next, as a multiple of `size`.
    pub line_height: f64,
    /// Extra space after every glyph that advances the pen, in scene units.
    pub letter_spacing: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: Font::regular(),
            size: 64.0,
            align: Align::Center,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

/// One shaped glyph, placed relative to the text's anchor. Scene units, y up.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedGlyph {
    /// The glyph's index in the font.
    pub id: u16,
    /// Byte offset in the text of the first character this glyph draws.
    pub cluster: usize,
    /// The line the glyph is on, counting from 0 at the top.
    pub line: usize,
    /// Where the font's outline origin goes: pen position plus the shaper's offset.
    pub origin: DVec2,
    /// The middle of the glyph's advance, on its baseline. Glyphs scale and turn about this
    /// point, so a row of them stays on its baseline.
    pub pivot: DVec2,
    /// How far the glyph moves the pen.
    pub advance: f64,
}

/// Shaped and placed text.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// Every glyph in visual order, line by line, spaces included.
    pub glyphs: Vec<PlacedGlyph>,
    /// Width of the widest line by the height of all line boxes.
    pub size: DVec2,
    /// Scene units per font unit.
    pub scale: f64,
}

/// Shapes `text` with `style` and places its glyphs.
///
/// Lines break only at `\n`. The block of line boxes (each `line_height * size` tall, the
/// font's ascent and descent centred in it) is centred vertically on the anchor, and each
/// line is aligned horizontally by `style.align`. Kerning, ligatures and complex scripts
/// come from the shaper; mixed left-to-right and right-to-left runs on one line are not
/// reordered.
pub fn layout(text: &str, style: &TextStyle) -> Layout {
    let Some(face) = style.font.face() else {
        return Layout { glyphs: Vec::new(), size: DVec2::ZERO, scale: 0.0 };
    };
    let units = f64::from(face.units_per_em());
    let scale = if units > 0.0 { style.size / units } else { 0.0 };
    let ascent = f64::from(face.ascender()) * scale;
    let descent = f64::from(face.descender()) * scale;
    let line_box = style.line_height * style.size;
    let lines: Vec<(usize, &str)> = text
        .split('\n')
        .scan(0, |start, line| {
            let at = *start;
            *start += line.len() + 1;
            Some((at, line))
        })
        .collect();
    let height = line_box * lines.len() as f64;
    let half_leading = (line_box - (ascent - descent)) / 2.0;

    let mut glyphs = Vec::new();
    let mut width: f64 = 0.0;
    for (k, &(start, line)) in lines.iter().enumerate() {
        let baseline = height / 2.0 - k as f64 * line_box - half_leading - ascent;
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(line);
        buffer.guess_segment_properties();
        let shaped = rustybuzz::shape(&face, &[], buffer);

        let first = glyphs.len();
        let mut pen = 0.0;
        for (info, pos) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
            let advance = f64::from(pos.x_advance) * scale;
            let offset = DVec2::new(f64::from(pos.x_offset), f64::from(pos.y_offset)) * scale;
            glyphs.push(PlacedGlyph {
                id: info.glyph_id as u16,
                cluster: start + info.cluster as usize,
                line: k,
                origin: DVec2::new(pen, baseline) + offset,
                pivot: DVec2::new(pen + advance / 2.0, baseline),
                advance,
            });
            pen += advance;
            if pos.x_advance != 0 {
                pen += style.letter_spacing;
            }
        }
        // No spacing after the last glyph.
        let line_width = if glyphs.len() > first { pen - style.letter_spacing } else { 0.0 };
        let line_width = line_width.max(0.0);
        let shift = match style.align {
            Align::Left => 0.0,
            Align::Center => -line_width / 2.0,
            Align::Right => -line_width,
        };
        for glyph in &mut glyphs[first..] {
            glyph.origin.x += shift;
            glyph.pivot.x += shift;
        }
        width = width.max(line_width);
    }
    Layout { glyphs, size: DVec2::new(width, height), scale }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> TextStyle {
        TextStyle { size: 100.0, ..TextStyle::default() }
    }

    fn with(f: impl FnOnce(&mut TextStyle)) -> TextStyle {
        let mut s = style();
        f(&mut s);
        s
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_line_is_as_wide_as_its_advances() {
        let one = layout("H", &style());
        let two = layout("HH", &style());
        assert_eq!(two.glyphs.len(), 2);
        assert!(close(two.size.x, 2.0 * one.glyphs[0].advance));
        assert!(close(two.glyphs[1].origin.x - two.glyphs[0].origin.x, one.glyphs[0].advance));
    }

    #[test]
    fn alignment_places_lines_about_the_anchor() {
        let left = layout("Hello", &with(|s| s.align = Align::Left));
        let right = layout("Hello", &with(|s| s.align = Align::Right));
        let centre = layout("Hello", &style());
        let end = |l: &Layout| l.glyphs.last().map(|g| g.pivot.x + g.advance / 2.0).unwrap();
        assert!(close(left.glyphs[0].pivot.x - left.glyphs[0].advance / 2.0, 0.0));
        assert!(close(end(&right), 0.0));
        let start = centre.glyphs[0].pivot.x - centre.glyphs[0].advance / 2.0;
        assert!(close(start, -end(&centre)));
    }

    #[test]
    fn lines_step_down_by_the_line_height_and_centre_as_a_block() {
        let l = layout("Hi\nthere\nyou", &with(|s| s.line_height = 1.5));
        let baseline = |line| l.glyphs.iter().find(|g| g.line == line).unwrap().pivot.y;
        assert!(close(baseline(0) - baseline(1), 150.0));
        assert!(close(baseline(1) - baseline(2), 150.0));
        assert!(close(l.size.y, 450.0));
        // The middle line's box is centred on the anchor.
        let one = layout("Hi", &with(|s| s.line_height = 1.5));
        assert!(close(baseline(1), one.glyphs[0].pivot.y));
    }

    #[test]
    fn clusters_point_into_the_whole_text() {
        let text = "ab\ncd";
        let l = layout(text, &style());
        let chars: String = l.glyphs.iter().map(|g| &text[g.cluster..g.cluster + 1]).collect();
        assert_eq!(chars, "abcd");
        assert_eq!(l.glyphs.iter().map(|g| g.line).collect::<Vec<_>>(), [0, 0, 1, 1]);
    }

    #[test]
    fn letter_spacing_widens_between_glyphs_only() {
        let plain = layout("Hello", &style());
        let spaced = layout("Hello", &with(|s| s.letter_spacing = 10.0));
        assert!(close(spaced.size.x - plain.size.x, 40.0));
    }

    #[test]
    fn size_scales_everything() {
        let small = layout("Kinetic\nmotion", &with(|s| s.size = 50.0));
        let big = layout("Kinetic\nmotion", &with(|s| s.size = 200.0));
        assert!(close(big.size.x, 4.0 * small.size.x) && close(big.size.y, 4.0 * small.size.y));
        for (a, b) in small.glyphs.iter().zip(&big.glyphs) {
            assert!((b.origin - 4.0 * a.origin).length() < 1e-9);
        }
    }

    #[test]
    fn the_shaper_positions_marks() {
        // Poppins positions Devanagari marks (its GPOS has no Latin kerning), so the
        // anusvara and candrabindu here sit off their pen positions.
        let l = layout("हिंदी चाँदनी", &style());
        let offset =
            |g: &PlacedGlyph| g.origin - DVec2::new(g.pivot.x - g.advance / 2.0, g.pivot.y);
        assert!(l.glyphs.iter().any(|g| offset(g).length() > 1e-6));
    }

    #[test]
    fn devanagari_is_shaped_not_mapped() {
        let text = "नमस्ते";
        let l = layout(text, &style());
        // The conjunct and the vowel sign are combined or reordered by the shaper.
        assert!(!l.glyphs.is_empty() && l.glyphs.len() < text.chars().count());
        assert!(l.glyphs.iter().all(|g| g.id != 0), "no missing glyphs");
    }

    #[test]
    fn empty_text_has_one_empty_line() {
        let l = layout("", &style());
        assert!(l.glyphs.is_empty());
        assert_eq!(l.size, DVec2::new(0.0, 120.0));
    }

    #[test]
    fn layout_is_deterministic() {
        let s = with(|s| s.letter_spacing = 3.5);
        assert_eq!(layout("Hello, नमस्ते!\nWorld", &s), layout("Hello, नमस्ते!\nWorld", &s));
    }
}
