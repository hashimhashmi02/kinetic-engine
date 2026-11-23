//! Scene graph, animatable properties, timeline, easing and springs. No GPU, no IO.
//!
//! Everything in this crate is deterministic: the same inputs always produce the same
//! outputs, with no dependence on the wall clock, randomness or iteration order.

pub mod bezier;
pub mod color;
pub mod ease;
pub mod lerp;
pub mod node;
pub mod spring;
pub mod value;

pub use bezier::CubicBezier;
pub use color::LinearRgba;
pub use ease::Ease;
pub use glam::DVec2;
pub use lerp::Lerp;
pub use node::{Circle, Node, NodeDef, NodeId, PropertyId, Props, Rect, Shape};
pub use spring::{spring, Spring};
pub use value::Value;
