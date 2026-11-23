//! The scene builder: the API scene code is written against.
//!
//! ```
//! use kinetic_core::color::palette;
//! use kinetic_core::{Circle, Ease, Scene};
//!
//! let mut s = Scene::new();
//! let dot = s.add(Circle::new(40.0).fill(palette::blue()));
//! s.play(dot.move_to((300.0, 0.0)), (0.8, Ease::OutCubic));
//! let timeline = s.build();
//! assert_eq!(timeline.duration, 0.8);
//! ```

use std::fmt;
use std::marker::PhantomData;

use glam::DVec2;

use crate::bezier::CubicBezier;
use crate::color::{palette, LinearRgba};
use crate::ease::Ease;
use crate::node::{Node, NodeDef, NodeId, PropertyId};
use crate::spring::Spring;
use crate::timeline::Timeline;
use crate::track::{Curve, Track};
use crate::value::Value;

/// How long a change takes and how it moves.
///
/// Usually written implicitly: `0.6` (seconds, default easing), `(0.6, Ease::OutBack)`,
/// `(0.6, CubicBezier::EASE)` or a [`Spring`], whose duration is its settle time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    pub duration: f64,
    pub curve: Curve,
}

impl From<f64> for Timing {
    fn from(duration: f64) -> Self {
        Self { duration, curve: Curve::Ease(Ease::default()) }
    }
}

impl From<(f64, Ease)> for Timing {
    fn from((duration, ease): (f64, Ease)) -> Self {
        Self { duration, curve: Curve::Ease(ease) }
    }
}

impl From<(f64, CubicBezier)> for Timing {
    fn from((duration, bezier): (f64, CubicBezier)) -> Self {
        Self { duration, curve: Curve::Bezier(bezier) }
    }
}

impl From<Spring> for Timing {
    fn from(spring: Spring) -> Self {
        Self { duration: spring.settle_time(), curve: Curve::Spring(spring) }
    }
}

/// A requested change: set one property of one node to a target value. Created by
/// [`Handle`] methods and carried out by [`Scene::play`] and friends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Action {
    pub node: NodeId,
    pub property: PropertyId,
    pub target: Value,
}

/// A typed reference to a node in a scene. Cheap to copy; its methods describe changes
/// and do not change anything themselves.
pub struct Handle<N> {
    id: NodeId,
    kind: PhantomData<fn() -> N>,
}

impl<N> Clone for Handle<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N> Copy for Handle<N> {}

impl<N> fmt::Debug for Handle<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle({})", self.id.0)
    }
}

impl<N> PartialEq for Handle<N> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<N> Handle<N> {
    /// The node this handle refers to.
    pub fn id(&self) -> NodeId {
        self.id
    }

    fn action(&self, property: PropertyId, target: Value) -> Action {
        Action { node: self.id, property, target }
    }

    /// Moves the node's centre to `position`.
    pub fn move_to(&self, position: impl Into<DVec2>) -> Action {
        self.action(PropertyId::Position, Value::Vec2(position.into()))
    }
}

/// Builds a [`Timeline`]. Scene code adds nodes, then plays changes one after another; a
/// cursor tracks where in time the next change starts.
#[derive(Debug, Clone)]
pub struct Scene {
    size: DVec2,
    background: LinearRgba,
    nodes: Vec<NodeDef>,
    /// Sorted by node, then property.
    tracks: Vec<Track>,
    cursor: f64,
    end: f64,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    /// An empty 1280 x 720 scene on a dark background.
    pub fn new() -> Self {
        Self {
            size: DVec2::new(1280.0, 720.0),
            background: palette::ink(),
            nodes: Vec::new(),
            tracks: Vec::new(),
            cursor: 0.0,
            end: 0.0,
        }
    }

    /// Sets the visible scene size in scene units.
    pub fn set_size(&mut self, width: f64, height: f64) {
        self.size = DVec2::new(width.max(1.0), height.max(1.0));
    }

    /// Sets the background colour.
    pub fn set_background(&mut self, color: LinearRgba) {
        self.background = color;
    }

    /// Adds a node, drawn above everything added before it.
    pub fn add<N: Node>(&mut self, node: N) -> Handle<N> {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node.def());
        Handle { id, kind: PhantomData }
    }

    /// Where the next change will start, in seconds.
    pub fn time(&self) -> f64 {
        self.cursor
    }

    /// Moves the cursor forward, leaving everything as it is for `seconds`.
    pub fn wait(&mut self, seconds: f64) {
        self.cursor += seconds.max(0.0);
        self.end = self.end.max(self.cursor);
    }

    /// Plays one change starting now, and moves the cursor to its end.
    pub fn play(&mut self, action: Action, timing: impl Into<Timing>) {
        let end = self.schedule(action, self.cursor, timing.into());
        self.cursor = end;
    }

    /// Adds `action` to its track starting at `start` and returns when it ends. Actions on
    /// a node that is not in this scene are ignored.
    fn schedule(&mut self, action: Action, start: f64, timing: Timing) -> f64 {
        let duration = timing.duration.max(0.0);
        let end = start + duration;
        if let Some(track) = self.track_mut(action.node, action.property) {
            track.push(start, duration, action.target, timing.curve);
            self.end = self.end.max(end);
        }
        end
    }

    fn track_mut(&mut self, node: NodeId, property: PropertyId) -> Option<&mut Track> {
        let initial = self.nodes.get(node.0 as usize)?.initial.get(property);
        let key = (node, property);
        let at = match self.tracks.binary_search_by_key(&key, |t| (t.node, t.property)) {
            Ok(i) => i,
            Err(i) => {
                self.tracks.insert(i, Track::new(node, property, initial));
                i
            }
        };
        self.tracks.get_mut(at)
    }

    /// Finishes the scene. Its duration runs to whichever is later: the last change or the
    /// cursor.
    pub fn build(self) -> Timeline {
        Timeline {
            size: self.size,
            background: self.background,
            duration: self.end.max(self.cursor),
            nodes: self.nodes,
            tracks: self.tracks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{Circle, Rect};
    use crate::spring::spring;

    #[test]
    fn plays_one_after_another() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        s.play(dot.move_to((100.0, 0.0)), 1.0);
        s.play(dot.move_to((100.0, 100.0)), 2.0);
        assert_eq!(s.time(), 3.0);
        let tl = s.build();
        assert_eq!(tl.duration, 3.0);
        assert_eq!(tl.evaluate(1.0).nodes[0].props.position, DVec2::new(100.0, 0.0));
        assert_eq!(tl.evaluate(3.0).nodes[0].props.position, DVec2::new(100.0, 100.0));
    }

    #[test]
    fn wait_holds_and_extends_the_duration() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        s.wait(0.5);
        s.play(dot.move_to((10.0, 0.0)), 1.0);
        s.wait(2.0);
        let tl = s.build();
        assert_eq!(tl.duration, 3.5);
        assert_eq!(tl.evaluate(0.5).nodes[0].props.position, DVec2::ZERO);
    }

    #[test]
    fn negative_wait_and_duration_do_not_go_back_in_time() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        s.wait(-4.0);
        s.play(dot.move_to((1.0, 0.0)), -1.0);
        assert_eq!(s.time(), 0.0);
    }

    #[test]
    fn timing_conversions() {
        assert_eq!(Timing::from(2.0).curve, Curve::Ease(Ease::InOutCubic));
        assert_eq!(Timing::from((1.0, Ease::OutBack)).curve, Curve::Ease(Ease::OutBack));
        let sp = spring();
        assert_eq!(Timing::from(sp).duration, sp.settle_time());
    }

    #[test]
    fn spring_play_lasts_its_settle_time() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        s.play(dot.move_to((50.0, 0.0)), spring());
        assert_eq!(s.time(), spring().settle_time());
    }

    #[test]
    fn tracks_stay_sorted_and_unique() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(1.0));
        let b = s.add(Rect::new(1.0, 1.0));
        s.play(b.move_to((1.0, 0.0)), 1.0);
        s.play(a.move_to((1.0, 0.0)), 1.0);
        s.play(a.move_to((2.0, 0.0)), 1.0);
        let tl = s.build();
        let keys: Vec<_> = tl.tracks.iter().map(|t| (t.node, t.property)).collect();
        assert_eq!(keys, [(a.id(), PropertyId::Position), (b.id(), PropertyId::Position)]);
        assert_eq!(tl.tracks[0].segments.len(), 2);
    }

    #[test]
    fn action_for_a_foreign_node_is_ignored() {
        let mut other = Scene::new();
        other.add(Circle::new(1.0));
        let foreign = other.add(Circle::new(1.0));
        let mut s = Scene::new();
        s.add(Circle::new(1.0));
        s.play(foreign.move_to((5.0, 0.0)), 1.0);
        assert!(s.build().tracks.is_empty());
    }
}
