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
#[derive(Debug, Clone, PartialEq)]
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

impl<N> Handle<N> {
    /// Scales the node uniformly; 1 is its original size.
    pub fn scale_to(&self, factor: f64) -> Action {
        self.action(PropertyId::Scale, Value::Vec2(DVec2::splat(factor)))
    }

    /// Scales the node by separate factors along x and y.
    pub fn scale_xy_to(&self, factors: impl Into<DVec2>) -> Action {
        self.action(PropertyId::Scale, Value::Vec2(factors.into()))
    }

    /// Rotates the node to `radians`, counter-clockwise. Takes the long way round when told
    /// to: going from 0 to `TAU` is a full turn.
    pub fn rotate_to(&self, radians: f64) -> Action {
        self.action(PropertyId::Rotation, Value::Scalar(radians))
    }

    /// Fades the node to `opacity`, where 0 is invisible and 1 is solid.
    pub fn fade_to(&self, opacity: f64) -> Action {
        self.action(PropertyId::Opacity, Value::Scalar(opacity.clamp(0.0, 1.0)))
    }

    /// Changes the node's fill colour; the blend happens in linear light.
    pub fn fill_to(&self, color: LinearRgba) -> Action {
        self.action(PropertyId::Fill, Value::Color(color))
    }
}

impl Scene {
    /// Plays several changes at once, all with the same timing, and moves the cursor to
    /// their common end.
    pub fn parallel(
        &mut self,
        actions: impl IntoIterator<Item = Action>,
        timing: impl Into<Timing>,
    ) {
        let timing = timing.into();
        let start = self.cursor;
        let mut end = start;
        for action in actions {
            end = end.max(self.schedule(action, start, timing));
        }
        self.cursor = end.max(start + timing.duration.max(0.0));
    }

    /// Plays several changes one after another, each with the same timing.
    pub fn sequence(
        &mut self,
        actions: impl IntoIterator<Item = Action>,
        timing: impl Into<Timing>,
    ) {
        let timing = timing.into();
        for action in actions {
            self.play(action, timing);
        }
    }
}

impl Scene {
    /// Plays several changes with the same timing, each starting `delay` seconds after the
    /// one before, and moves the cursor to when the last one ends. The classic way to bring
    /// in a row of items one by one.
    pub fn stagger(
        &mut self,
        actions: impl IntoIterator<Item = Action>,
        timing: impl Into<Timing>,
        delay: f64,
    ) {
        let timing = timing.into();
        let delay = delay.max(0.0);
        let start = self.cursor;
        let mut end = start;
        for (i, action) in actions.into_iter().enumerate() {
            end = end.max(self.schedule(action, start + i as f64 * delay, timing));
        }
        self.cursor = end;
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

    #[test]
    fn every_action_lands_on_its_property() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        s.play(dot.scale_to(2.0), 1.0);
        s.play(dot.scale_xy_to((3.0, 0.5)), 1.0);
        s.play(dot.rotate_to(std::f64::consts::TAU), 1.0);
        s.play(dot.fade_to(0.25), 1.0);
        s.play(dot.fill_to(palette::red()), 1.0);
        let tl = s.build();
        let mid_scale = tl.evaluate(1.0).nodes[0].props.scale;
        assert_eq!(mid_scale, DVec2::splat(2.0));
        let end = tl.evaluate(tl.duration).nodes[0].props;
        assert_eq!(end.scale, DVec2::new(3.0, 0.5));
        assert_eq!(end.rotation, std::f64::consts::TAU);
        assert_eq!(end.opacity, 0.25);
        assert_eq!(end.fill, palette::red());
    }

    #[test]
    fn fade_target_is_clamped() {
        let mut s = Scene::new();
        let dot = s.add(Circle::new(10.0));
        assert_eq!(dot.fade_to(3.0).target, Value::Scalar(1.0));
        s.play(dot.fade_to(-1.0), 1.0);
        assert_eq!(s.build().evaluate(1.0).nodes[0].props.opacity, 0.0);
    }

    #[test]
    fn parallel_starts_together_and_ends_together() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(1.0));
        let b = s.add(Circle::new(1.0));
        s.wait(1.0);
        s.parallel([a.move_to((10.0, 0.0)), b.fade_to(0.0)], (2.0, Ease::Linear));
        assert_eq!(s.time(), 3.0);
        let mid = s.build().evaluate(2.0);
        assert_eq!(mid.nodes[0].props.position, DVec2::new(5.0, 0.0));
        assert_eq!(mid.nodes[1].props.opacity, 0.5);
    }

    #[test]
    fn empty_parallel_still_takes_its_time() {
        let mut s = Scene::new();
        s.parallel([], 1.5);
        assert_eq!(s.time(), 1.5);
    }

    #[test]
    fn sequence_chains_with_equal_timing() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(1.0));
        s.sequence([a.move_to((10.0, 0.0)), a.move_to((10.0, 10.0)), a.fade_to(0.0)], 0.5);
        assert_eq!(s.time(), 1.5);
        let tl = s.build();
        assert_eq!(tl.evaluate(0.5).nodes[0].props.position, DVec2::new(10.0, 0.0));
        assert_eq!(tl.evaluate(1.0).nodes[0].props.position, DVec2::new(10.0, 10.0));
    }

    #[test]
    fn stagger_offsets_each_start() {
        let mut s = Scene::new();
        let items: Vec<_> = (0..4).map(|_| s.add(Circle::new(1.0).opacity(0.0))).collect();
        s.stagger(items.iter().map(|h| h.fade_to(1.0)), (1.0, Ease::Linear), 0.25);
        assert_eq!(s.time(), 1.75);
        let frame = s.build().evaluate(0.5);
        let opacities: Vec<f64> = frame.nodes.iter().map(|n| n.props.opacity).collect();
        assert_eq!(opacities, [0.5, 0.25, 0.0, 0.0]);
    }

    #[test]
    fn stagger_on_one_property_hands_over_smoothly() {
        // Overlapping changes to the same track: each takes over where the last had got to.
        let mut s = Scene::new();
        let dot = s.add(Circle::new(1.0));
        s.stagger([dot.move_to((10.0, 0.0)), dot.move_to((0.0, 10.0))], (1.0, Ease::Linear), 0.5);
        let tl = s.build();
        let at = |t: f64| tl.evaluate(t).nodes[0].props.position;
        assert!((at(0.5 - 1e-9) - at(0.5)).length() < 1e-6);
        assert_eq!(at(1.5), DVec2::new(0.0, 10.0));
    }
}
