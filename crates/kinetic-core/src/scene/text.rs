//! Text in a scene: a group of glyph nodes that can be animated together.

use glam::DVec2;

use std::marker::PhantomData;

use super::{Action, Change, Handle, Scene};
use crate::color::LinearRgba;
use crate::node::{NodeDef, NodeId, PropertyId, Props};
use crate::text::Text;
use crate::value::Value;

/// The kind of node a glyph of a text is, for [`Handle<Glyph>`](super::Handle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {}

/// A text added with [`Scene::add_text`]. Cheap to copy; its methods describe changes to
/// every glyph of the text and do not change anything themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextHandle(u32);

/// A change to a whole text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TextEdit {
    /// Set a property of every glyph.
    Set(PropertyId, Value),
    /// Move the anchor, and every glyph with it.
    Move(DVec2),
}

/// What the scene knows about a text while it is being built. Edits are applied in the
/// order they are scheduled.
#[derive(Debug, Clone)]
pub(crate) struct TextState {
    /// The glyph nodes now showing the text, in order, with their offsets from the anchor.
    pub glyphs: Vec<(NodeId, DVec2)>,
    pub anchor: DVec2,
    /// The text's properties, used for glyphs added later.
    pub props: Props,
}

impl TextHandle {
    fn edit(&self, edit: TextEdit) -> Action {
        Action { changes: vec![Change::Text { text: *self, edit }] }
    }

    /// Fades every glyph to `opacity`.
    pub fn fade_to(&self, opacity: f64) -> Action {
        self.edit(TextEdit::Set(PropertyId::Opacity, Value::Scalar(opacity.clamp(0.0, 1.0))))
    }

    /// Changes the fill colour of every glyph.
    pub fn fill_to(&self, color: LinearRgba) -> Action {
        self.edit(TextEdit::Set(PropertyId::Fill, Value::Color(color)))
    }

    /// Changes the outline colour of every glyph.
    pub fn stroke_to(&self, color: LinearRgba) -> Action {
        self.edit(TextEdit::Set(PropertyId::Stroke, Value::Color(color)))
    }

    /// Changes the outline width of every glyph.
    pub fn stroke_width_to(&self, width: f64) -> Action {
        self.edit(TextEdit::Set(PropertyId::StrokeWidth, Value::Scalar(width.max(0.0))))
    }

    /// Moves the text's anchor to `position`; the glyphs keep their places in the line.
    pub fn move_to(&self, position: impl Into<DVec2>) -> Action {
        self.edit(TextEdit::Move(position.into()))
    }
}

impl Scene {
    /// Adds `text` as one path node per glyph that draws something, in reading order and
    /// drawn above everything added before.
    pub fn add_text(&mut self, text: Text) -> TextHandle {
        let handle = TextHandle(self.texts.len() as u32);
        let anchor = text.props.position;
        let glyphs = text
            .glyphs()
            .into_iter()
            .map(|glyph| {
                let node = NodeId(self.nodes.len() as u32);
                let initial = Props { position: anchor + glyph.offset, ..text.props };
                self.nodes.push(NodeDef { shape: glyph.shape, initial });
                (node, glyph.offset)
            })
            .collect();
        self.texts.push(TextState { glyphs, anchor, props: text.props });
        handle
    }

    /// Handles to the glyph nodes now showing `text`, in reading order, for animating glyphs
    /// one at a time:
    ///
    /// ```
    /// # use kinetic_core::{Ease, Scene, Text};
    /// let mut s = Scene::new();
    /// let title = s.add_text(Text::new("Hello").opacity(0.0));
    /// s.stagger(s.glyphs(title).into_iter().map(|g| g.fade_to(1.0)), (0.4, Ease::OutCubic), 0.05);
    /// ```
    ///
    /// Empty for a text from another scene.
    pub fn glyphs(&self, text: TextHandle) -> Vec<Handle<Glyph>> {
        self.texts
            .get(text.0 as usize)
            .map(|state| {
                state.glyphs.iter().map(|&(id, _)| Handle { id, kind: PhantomData }).collect()
            })
            .unwrap_or_default()
    }

    /// Turns an edit of a whole text into changes to its glyphs, and records it so later
    /// edits start from it. An unknown text gives no changes.
    pub(super) fn edit_text(&mut self, text: TextHandle, edit: TextEdit) -> Action {
        let Some(state) = self.texts.get_mut(text.0 as usize) else {
            return Action::all([]);
        };
        let changes = match edit {
            TextEdit::Set(property, target) => {
                state.props.set(property, target.clone());
                let set = |&(node, _): &(NodeId, DVec2)| Change::Set {
                    node,
                    property,
                    target: target.clone(),
                };
                state.glyphs.iter().map(set).collect()
            }
            TextEdit::Move(to) => {
                state.anchor = to;
                let set = |&(node, offset): &(NodeId, DVec2)| Change::Set {
                    node,
                    property: PropertyId::Position,
                    target: Value::Vec2(to + offset),
                };
                state.glyphs.iter().map(set).collect()
            }
        };
        Action { changes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;
    use crate::ease::Ease;
    use crate::shape::Shape;

    #[test]
    fn one_node_per_glyph_that_draws() {
        let mut s = Scene::new();
        s.add_text(Text::new("Hi you").at((10.0, 20.0)));
        let tl = s.build();
        assert_eq!(tl.nodes.len(), 5);
        assert!(tl.nodes.iter().all(|n| matches!(n.shape, Shape::Path(_))));
        let glyphs = Text::new("Hi you").glyphs();
        for (node, glyph) in tl.nodes.iter().zip(&glyphs) {
            assert_eq!(node.initial.position, DVec2::new(10.0, 20.0) + glyph.offset);
        }
    }

    #[test]
    fn glyphs_take_the_text_style() {
        let mut s = Scene::new();
        s.add_text(Text::new("ab").fill(palette::red()).opacity(0.5));
        let tl = s.build();
        assert!(tl.nodes.iter().all(|n| n.initial.fill == palette::red()));
        assert!(tl.nodes.iter().all(|n| n.initial.opacity == 0.5));
    }

    #[test]
    fn group_changes_reach_every_glyph() {
        let mut s = Scene::new();
        let word = s.add_text(Text::new("Hello"));
        s.parallel([word.fade_to(0.0), word.fill_to(palette::blue())], (1.0, Ease::Linear));
        let end = s.build().evaluate(1.0);
        assert_eq!(end.nodes.len(), 5);
        assert!(end.nodes.iter().all(|n| n.props.opacity == 0.0));
        assert!(end.nodes.iter().all(|n| n.props.fill == palette::blue()));
    }

    #[test]
    fn moving_a_text_keeps_its_spacing() {
        let mut s = Scene::new();
        let word = s.add_text(Text::new("Hello"));
        s.play(word.move_to((100.0, -50.0)), 1.0);
        s.play(word.move_to((0.0, 0.0)), 1.0);
        let tl = s.build();
        let at = |t: f64| tl.evaluate(t).nodes.iter().map(|n| n.props.position).collect::<Vec<_>>();
        let (start, moved, back) = (at(0.0), at(1.0), at(2.0));
        for (a, b) in start.iter().zip(&moved) {
            assert!((*b - *a - DVec2::new(100.0, -50.0)).length() < 1e-9);
        }
        assert_eq!(start, back);
    }

    #[test]
    fn edits_to_an_unknown_text_are_ignored() {
        let mut s = Scene::new();
        s.add_text(Text::new("a"));
        let mut other = Scene::new();
        other.add_text(Text::new("a"));
        let missing = other.add_text(Text::new("b"));
        s.play(missing.fade_to(0.0), 1.0);
        assert_eq!(s.build().evaluate(1.0).nodes[0].props.opacity, 1.0);
    }

    #[test]
    fn text_timelines_are_deterministic_and_survive_the_wire() {
        let build = || {
            let mut s = Scene::new();
            let w = s.add_text(Text::new("Kinetic\nनमस्ते").size(80.0));
            s.play(w.move_to((0.0, 100.0)), (0.6, Ease::OutCubic));
            s.build()
        };
        let tl = build();
        assert_eq!(tl, build());
        let back = crate::wire::decode(&crate::wire::encode(&tl).expect("encode")).expect("decode");
        assert_eq!(back, tl);
    }

    #[test]
    fn glyphs_stagger_one_by_one() {
        let mut s = Scene::new();
        let word = s.add_text(Text::new("Hello").opacity(0.0));
        let glyphs = s.glyphs(word);
        assert_eq!(glyphs.len(), 5);
        s.stagger(s.glyphs(word).into_iter().map(|g| g.fade_to(1.0)), (0.5, Ease::Linear), 0.1);
        let tl = s.build();
        assert!((tl.duration - 0.9).abs() < 1e-12);
        let opacity = tl.evaluate(0.25).nodes.iter().map(|n| n.props.opacity).collect::<Vec<_>>();
        let expected = [0.5, 0.3, 0.1, 0.0, 0.0];
        for (got, want) in opacity.iter().zip(expected) {
            assert!((got - want).abs() < 1e-9, "{opacity:?}");
        }
    }

    #[test]
    fn a_glyph_turns_and_grows_about_its_baseline() {
        let mut s = Scene::new();
        let word = s.add_text(Text::new("Hey"));
        let e = s.glyphs(word)[1];
        s.parallel([e.rotate_to(0.3), e.scale_to(2.0)], 1.0);
        let tl = s.build();
        let (start, end) = (tl.evaluate(0.0), tl.evaluate(1.0));
        assert_eq!(start.nodes[1].props.position, end.nodes[1].props.position);
        assert_eq!(end.nodes[1].props.rotation, 0.3);
        assert_eq!(end.nodes[0].props, start.nodes[0].props, "the others stay put");
    }

    #[test]
    fn a_foreign_text_has_no_glyphs() {
        let mut other = Scene::new();
        other.add_text(Text::new("a"));
        let foreign = other.add_text(Text::new("b"));
        assert!(Scene::new().glyphs(foreign).is_empty());
    }
}
