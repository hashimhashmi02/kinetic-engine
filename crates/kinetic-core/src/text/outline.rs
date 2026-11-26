//! Glyph outlines as paths.

use glam::DVec2;
use rustybuzz::ttf_parser::{GlyphId, OutlineBuilder};

use crate::path::{PathData, PathEl};

/// Collects a glyph outline into a [`PathData`], scaling font units to scene units and
/// moving the result by `shift`.
struct Builder {
    els: Vec<PathEl>,
    scale: f64,
    shift: DVec2,
}

impl Builder {
    fn p(&self, x: f32, y: f32) -> DVec2 {
        DVec2::new(f64::from(x), f64::from(y)) * self.scale + self.shift
    }
}

impl OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.els.push(PathEl::MoveTo(p));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.p(x, y);
        self.els.push(PathEl::LineTo(p));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.p(x1, y1), self.p(x, y));
        self.els.push(PathEl::QuadTo(c, p));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.p(x1, y1), self.p(x2, y2), self.p(x, y));
        self.els.push(PathEl::CubicTo(c1, c2, p));
    }

    fn close(&mut self) {
        self.els.push(PathEl::Close);
    }
}

/// The outline of glyph `id`, in scene units (`scale` per font unit, y up) with the font's
/// origin at `shift`. `None` for a glyph that draws nothing, such as a space.
pub(crate) fn glyph_path(
    face: &rustybuzz::Face,
    id: u16,
    scale: f64,
    shift: DVec2,
) -> Option<PathData> {
    let mut builder = Builder { els: Vec::new(), scale, shift };
    face.outline_glyph(GlyphId(id), &mut builder)?;
    (!builder.els.is_empty()).then_some(PathData { els: builder.els })
}
