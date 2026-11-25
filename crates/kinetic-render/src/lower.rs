//! Turning an evaluated scene into something drawable.

use kinetic_core::{FrameState, Shape};
use kinetic_vector::{fill, node_path, stroke, trim, StrokeStyle};

use crate::{Frame, Item};

impl Frame {
    /// Tessellates every node of `state` into a drawable frame.
    ///
    /// Each node gives its fill and then its outline, so the outline is drawn on top. A line
    /// has no fill. The outline is cut to the node's `trim` range and its width grows with
    /// the node's scale (by the square root of the area scale, so a squashed node keeps an
    /// even-looking outline).
    ///
    /// `tolerance` is the curve flattening tolerance in scene units; pass the size of a
    /// fraction of a pixel at the output resolution. Fully transparent nodes and parts are
    /// skipped, and so is a path that cannot be tessellated (it has collapsed to nothing).
    pub fn from_state(state: &FrameState, tolerance: f64) -> Self {
        let mut items = Vec::new();
        for node in state.nodes.iter().filter(|node| node.props.opacity > 0.0) {
            let p = &node.props;
            let opacity = p.opacity as f32;
            let path = node_path(&node.shape, p.scale, p.rotation);
            let mut push = |mesh: Option<kinetic_vector::Mesh>, color: kinetic_core::LinearRgba| {
                if let Some(mesh) = mesh.filter(|m| !m.is_empty()) {
                    items.push(Item {
                        mesh,
                        color: color.with_opacity(opacity),
                        offset: p.position,
                    });
                }
            };
            if p.fill.a > 0.0 && !matches!(node.shape, Shape::Line { .. }) {
                push(fill(&path, tolerance).ok(), p.fill);
            }
            let width = p.stroke_width * (p.scale.x * p.scale.y).abs().sqrt();
            if width > 0.0 && p.stroke.a > 0.0 {
                let style = StrokeStyle { width, cap: p.cap, join: p.join };
                push(stroke(&trim(&path, p.trim.x, p.trim.y), &style, tolerance).ok(), p.stroke);
            }
        }
        Frame { background: state.background, size: state.size, items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::color::palette;
    use kinetic_core::{Circle, Line, LineCap, Rect, Scene};

    #[test]
    fn one_item_per_visible_node_in_order() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(10.0).fill(palette::red()).at((5.0, 6.0)));
        s.add(Rect::new(4.0, 4.0).opacity(0.0));
        s.add(Rect::new(4.0, 4.0).fill(palette::blue()));
        s.play(a.fade_to(0.5), 1.0);
        let frame = Frame::from_state(&s.build().evaluate(1.0), 0.1);
        assert_eq!(frame.items.len(), 2);
        assert_eq!(frame.items[0].offset, kinetic_core::DVec2::new(5.0, 6.0));
        assert_eq!(frame.items[0].color, palette::red().with_opacity(0.5));
        assert_eq!(frame.items[1].color, palette::blue());
    }

    #[test]
    fn scale_changes_the_mesh_not_the_offset() {
        let mut s = Scene::new();
        let a = s.add(Rect::new(10.0, 10.0));
        s.play(a.scale_to(2.0), 1.0);
        let tl = s.build();
        let before = Frame::from_state(&tl.evaluate(0.0), 0.1);
        let after = Frame::from_state(&tl.evaluate(1.0), 0.1);
        assert!((after.items[0].mesh.area() - 4.0 * before.items[0].mesh.area()).abs() < 1e-6);
    }

    #[test]
    fn outline_is_drawn_over_the_fill() {
        let mut s = Scene::new();
        s.add(Circle::new(10.0).fill(palette::red()).stroke(palette::blue(), 2.0));
        let frame = Frame::from_state(&s.build().evaluate(0.0), 0.1);
        assert_eq!(frame.items.len(), 2);
        assert_eq!((frame.items[0].color, frame.items[1].color), (palette::red(), palette::blue()));
    }

    fn line(width: f64) -> Line {
        Line::new((0.0, 0.0), (100.0, 0.0)).stroke(palette::WHITE, width).line_cap(LineCap::Butt)
    }

    #[test]
    fn a_line_has_only_an_outline() {
        let mut s = Scene::new();
        s.add(line(10.0));
        let frame = Frame::from_state(&s.build().evaluate(0.0), 0.1);
        assert_eq!(frame.items.len(), 1);
        assert!((frame.items[0].mesh.area() - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn trim_draws_part_of_the_outline() {
        let mut s = Scene::new();
        let a = s.add(line(10.0).trim(0.0, 0.0));
        s.play(a.trim_to(0.0, 1.0), (1.0, kinetic_core::Ease::Linear));
        let tl = s.build();
        assert!(Frame::from_state(&tl.evaluate(0.0), 0.1).items.is_empty());
        let half = Frame::from_state(&tl.evaluate(0.5), 0.1);
        assert!((half.items[0].mesh.area() - 500.0).abs() < 1e-3);
    }

    #[test]
    fn outline_width_follows_scale() {
        let mut s = Scene::new();
        let a = s.add(line(10.0));
        s.play(a.scale_to(2.0), 1.0);
        let frame = Frame::from_state(&s.build().evaluate(1.0), 0.1);
        // Twice as long and twice as wide.
        assert!((frame.items[0].mesh.area() - 4000.0).abs() < 1e-2);
    }

    #[test]
    fn transparent_fill_is_skipped() {
        let mut s = Scene::new();
        s.add(Rect::new(4.0, 4.0).fill(kinetic_core::LinearRgba::TRANSPARENT));
        assert!(Frame::from_state(&s.build().evaluate(0.0), 0.1).items.is_empty());
    }
}
