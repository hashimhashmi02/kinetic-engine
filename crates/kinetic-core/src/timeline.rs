//! A compiled scene: nodes, tracks, and evaluation at any time.

use glam::DVec2;
use serde::{Deserialize, Serialize};

use crate::color::LinearRgba;
use crate::node::{NodeDef, NodeId, Props, Shape};
use crate::track::Track;

/// Everything a scene does, as data. Built once by running scene code; after that, any
/// frame is a pure function of time via [`Timeline::evaluate`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    /// Size of the visible scene rectangle in scene units, centred on the origin.
    pub size: DVec2,
    /// Colour behind everything.
    pub background: LinearRgba,
    /// Length in seconds.
    pub duration: f64,
    /// Nodes in creation order; a node's index is its [`NodeId`].
    pub nodes: Vec<NodeDef>,
    /// Tracks sorted by node, then property.
    pub tracks: Vec<Track>,
}

/// One node at one moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeState {
    pub id: NodeId,
    pub shape: Shape,
    pub props: Props,
}

/// The whole scene at one moment: what the renderer draws.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameState {
    pub time: f64,
    pub size: DVec2,
    pub background: LinearRgba,
    /// In drawing order.
    pub nodes: Vec<NodeState>,
}

impl Timeline {
    /// The scene at time `t` seconds. Pure: the same `t` always gives the same frame, in
    /// any order of calls. Times outside `[0, duration]` hold the first or last frame, and
    /// NaN is treated as 0.
    pub fn evaluate(&self, t: f64) -> FrameState {
        let t = if t.is_nan() { 0.0 } else { t };
        let mut nodes: Vec<NodeState> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, def)| NodeState {
                id: NodeId(i as u32),
                shape: def.shape,
                props: def.initial,
            })
            .collect();
        for track in &self.tracks {
            if let Some(node) = nodes.get_mut(track.node.0 as usize) {
                node.props.set(track.property, track.value_at(t));
            }
        }
        FrameState { time: t, size: self.size, background: self.background, nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::palette;
    use crate::ease::Ease;
    use crate::node::{Circle, Node, PropertyId};
    use crate::track::Curve;
    use crate::value::Value;

    fn timeline() -> Timeline {
        let mut track =
            Track::new(NodeId(0), PropertyId::Position, Value::Vec2(DVec2::new(-100.0, 0.0)));
        track.push(0.0, 2.0, Value::Vec2(DVec2::new(100.0, 0.0)), Curve::Ease(Ease::Linear));
        Timeline {
            size: DVec2::new(640.0, 360.0),
            background: palette::ink(),
            duration: 2.0,
            nodes: vec![
                Circle::new(10.0).at((-100.0, 0.0)).def(),
                Circle::new(5.0).fill(palette::red()).def(),
            ],
            tracks: vec![track],
        }
    }

    #[test]
    fn evaluates_tracks_onto_their_nodes() {
        let frame = timeline().evaluate(1.0);
        assert_eq!(frame.nodes.len(), 2);
        assert_eq!(frame.nodes[0].props.position, DVec2::ZERO);
        // The second node has no tracks and keeps its initial values.
        assert_eq!(frame.nodes[1].props.fill, palette::red());
        assert_eq!(frame.nodes[1].id, NodeId(1));
    }

    #[test]
    fn same_time_gives_same_frame_in_any_order() {
        let tl = timeline();
        let (a1, b1) = (tl.evaluate(0.3), tl.evaluate(1.7));
        let (b2, a2) = (tl.evaluate(1.7), tl.evaluate(0.3));
        assert_eq!(a1, a2);
        assert_eq!(b1, b2);
    }

    #[test]
    fn holds_ends_outside_the_range_and_survives_nan() {
        let tl = timeline();
        assert_eq!(tl.evaluate(-3.0).nodes[0].props.position, DVec2::new(-100.0, 0.0));
        assert_eq!(tl.evaluate(9.0).nodes[0].props.position, DVec2::new(100.0, 0.0));
        assert_eq!(tl.evaluate(f64::NAN).nodes, tl.evaluate(0.0).nodes);
    }

    #[test]
    fn track_for_a_missing_node_is_ignored() {
        let mut tl = timeline();
        tl.tracks.push(Track::new(NodeId(7), PropertyId::Opacity, Value::Scalar(0.0)));
        assert_eq!(tl.evaluate(1.0).nodes.len(), 2);
    }
}
