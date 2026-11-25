//! Scene graph, animatable properties, timeline, easing and springs. No GPU, no IO.
//!
//! Everything in this crate is deterministic: the same inputs always produce the same
//! outputs, with no dependence on the wall clock, randomness or iteration order.

pub mod abi;
pub mod bezier;
pub mod color;
pub mod ease;
pub mod lerp;
pub mod morph;
pub mod node;
pub mod path;
pub mod polyline;
pub mod scene;
pub mod shape;
pub mod spring;
pub mod timeline;
pub mod track;
pub mod value;
pub mod wire;

pub use bezier::CubicBezier;
pub use color::LinearRgba;
pub use ease::Ease;
pub use glam::DVec2;
pub use lerp::Lerp;
pub use node::{
    Circle, Line, LineCap, LineJoin, Node, NodeDef, NodeId, PathShape, Polygon, PropertyId, Props,
    Rect, RoundedRect, Star,
};
pub use path::{Contour, PathData, PathEl};
pub use scene::{Action, Handle, Scene, Timing};
pub use shape::Shape;
pub use spring::{spring, Spring};
pub use timeline::{FrameState, NodeState, Timeline};
pub use track::{Curve, Segment, Track};
pub use value::Value;
