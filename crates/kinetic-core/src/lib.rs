//! Scene graph, animatable properties, timeline, easing and springs. No GPU, no IO.
//!
//! Everything in this crate is deterministic: the same inputs always produce the same
//! outputs, with no dependence on the wall clock, randomness or iteration order.

pub mod color;

pub use color::LinearRgba;
pub use glam::DVec2;
