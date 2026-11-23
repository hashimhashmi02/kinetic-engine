//! Turning an evaluated scene into something drawable.

use kinetic_core::FrameState;
use kinetic_vector::{fill, node_path};

use crate::{Frame, Item};

impl Frame {
    /// Tessellates every node of `state` into a drawable frame.
    ///
    /// `tolerance` is the curve flattening tolerance in scene units; pass the size of a
    /// fraction of a pixel at the output resolution. Fully transparent nodes are skipped. A
    /// node whose path cannot be tessellated (it has collapsed to nothing) is skipped too.
    pub fn from_state(state: &FrameState, tolerance: f64) -> Self {
        let items = state
            .nodes
            .iter()
            .filter(|node| node.props.opacity > 0.0)
            .filter_map(|node| {
                let p = node.props;
                let mesh = fill(&node_path(&node.shape, p.scale, p.rotation), tolerance).ok()?;
                Some(Item {
                    mesh,
                    color: p.fill.with_opacity(p.opacity as f32),
                    offset: p.position,
                })
            })
            .collect();
        Frame { background: state.background, size: state.size, items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::color::palette;
    use kinetic_core::{Circle, Rect, Scene};

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
}
