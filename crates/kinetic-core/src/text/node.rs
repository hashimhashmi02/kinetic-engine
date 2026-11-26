//! The text node builder.

use glam::DVec2;

use super::layout::{layout, Align, TextStyle};
use super::outline::glyph_path;
use super::Font;
use crate::color::LinearRgba;
use crate::node::{style_setters, LineCap, LineJoin, Props};
use crate::shape::Shape;

/// A block of text. [`Scene::add_text`](crate::Scene::add_text) adds it as one path node
/// per glyph, so the whole text can be animated as a group and each glyph on its own.
///
/// The style setters (`fill`, `stroke`, `at`, ...) apply to every glyph; `at` places the
/// text's anchor, which `align` and the vertical centring are measured from.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub(crate) text: String,
    pub(crate) style: TextStyle,
    pub(crate) props: Props,
}

/// The shape of one glyph, ready to become a node.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphShape {
    /// The outline as a path, about the glyph's pivot (the middle of its advance, on the
    /// baseline).
    pub shape: Shape,
    /// Where the pivot sits relative to the text's anchor.
    pub offset: DVec2,
    /// Byte offset in the text of the first character the glyph draws.
    pub cluster: usize,
}

impl Text {
    /// White text in Poppins Regular at size 64, centred on the origin.
    pub fn new(text: impl Into<String>) -> Self {
        Text { text: text.into(), style: TextStyle::default(), props: Props::default() }
    }

    /// Sets the font size in scene units.
    pub fn size(mut self, size: f64) -> Self {
        self.style.size = size.max(0.0);
        self
    }

    /// Sets the font.
    pub fn font(mut self, font: Font) -> Self {
        self.style.font = font;
        self
    }

    /// Uses the bundled bold font.
    pub fn bold(self) -> Self {
        self.font(Font::bold())
    }

    /// Sets how lines sit about the anchor.
    pub fn align(mut self, align: Align) -> Self {
        self.style.align = align;
        self
    }

    /// Sets the distance between baselines as a multiple of the size.
    pub fn line_height(mut self, line_height: f64) -> Self {
        self.style.line_height = line_height;
        self
    }

    /// Adds `spacing` scene units after every glyph.
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.style.letter_spacing = spacing;
        self
    }

    style_setters!();

    /// The text being set.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The glyphs that draw something, in visual order; spaces are left out.
    pub fn glyphs(&self) -> Vec<GlyphShape> {
        let placed = layout(&self.text, &self.style);
        let Some(face) = self.style.font.face() else {
            return Vec::new();
        };
        placed
            .glyphs
            .iter()
            .filter_map(|g| {
                let path = glyph_path(&face, g.id, placed.scale, g.origin - g.pivot)?;
                Some(GlyphShape { shape: Shape::Path(path), offset: g.pivot, cluster: g.cluster })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::PathData;
    use crate::shape::signed_area;

    fn path(shape: &Shape) -> &PathData {
        match shape {
            Shape::Path(p) => p,
            other => panic!("expected a path, got {other:?}"),
        }
    }

    fn bounds(shape: &Shape) -> (DVec2, DVec2) {
        let points = path(shape).flatten(0.1).into_iter().flat_map(|c| c.points);
        points.fold((DVec2::INFINITY, DVec2::NEG_INFINITY), |(lo, hi), p| (lo.min(p), hi.max(p)))
    }

    #[test]
    fn spaces_draw_nothing() {
        let glyphs = Text::new("a b  c").glyphs();
        assert_eq!(glyphs.iter().map(|g| g.cluster).collect::<Vec<_>>(), [0, 2, 5]);
    }

    #[test]
    fn a_glyph_stands_on_its_baseline_about_its_pivot() {
        let h = &Text::new("H").size(100.0).glyphs()[0];
        let (lo, hi) = bounds(&h.shape);
        assert!(lo.y.abs() < 1e-9, "sits on the baseline: {lo}");
        assert!(hi.y > 60.0 && hi.y < 80.0, "cap height: {hi}");
        assert!((lo.x + hi.x).abs() < 2.0, "centred on its advance: {lo} {hi}");
    }

    #[test]
    fn counters_wind_against_the_outer_contour() {
        let o = &Text::new("o").size(100.0).glyphs()[0];
        let contours = path(&o.shape).flatten(0.1);
        assert_eq!(contours.len(), 2);
        let (outer, inner) = (signed_area(&contours[0].points), signed_area(&contours[1].points));
        assert!(outer * inner < 0.0, "{outer} {inner}");
        assert!(outer.abs() > inner.abs());
    }

    #[test]
    fn offsets_follow_the_layout() {
        let glyphs = Text::new("ab").size(100.0).glyphs();
        let placed = layout("ab", &Text::new("").size(100.0).style);
        assert_eq!(glyphs[0].offset, placed.glyphs[0].pivot);
        assert_eq!(glyphs[1].offset, placed.glyphs[1].pivot);
    }

    #[test]
    fn builders_set_the_style() {
        let text = Text::new("x").size(-3.0).bold().align(Align::Left).line_height(2.0);
        let text = text.letter_spacing(4.0).fill(LinearRgba::TRANSPARENT).at((1.0, 2.0));
        assert_eq!(text.style.size, 0.0);
        assert_eq!(text.style.font, Font::bold());
        assert_eq!((text.style.align, text.style.line_height), (Align::Left, 2.0));
        assert_eq!(text.style.letter_spacing, 4.0);
        assert_eq!(text.props.position, DVec2::new(1.0, 2.0));
        assert_eq!(text.text(), "x");
    }
}
