//! Builds scene crates, loads them as libraries and reloads them when they change.

mod build;

pub use build::{BuildError, SceneCrate};
