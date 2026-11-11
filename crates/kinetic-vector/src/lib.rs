//! Bezier paths, tessellation and shape morphing.

pub mod mesh;
pub mod shape;

pub use kurbo::BezPath;
pub use mesh::{fill, Mesh, TessellateError};
