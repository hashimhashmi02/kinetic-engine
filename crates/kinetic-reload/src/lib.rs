//! Builds scene crates, loads them as libraries and reloads them when they change.

mod build;
mod load;

pub use build::{BuildError, SceneCrate};
pub use load::{clean_scratch, load, LoadError};
