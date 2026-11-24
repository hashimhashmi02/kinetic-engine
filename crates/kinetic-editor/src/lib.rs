//! Preview viewport, timeline scrubber and inspector.

mod app;
mod transport;

use std::fmt;

pub use app::{run, run_scene};

/// The preview window could not be opened or lost its graphics device.
#[derive(Debug)]
pub enum EditorError {
    /// The window or its drawing surface could not be created.
    Window(String),
    /// The renderer could not be set up.
    Render(kinetic_render::RenderError),
    /// The scene crate could not be opened or watched.
    Scene(String),
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(why) => write!(f, "could not open the preview window: {why}"),
            Self::Render(error) => write!(f, "{error}"),
            Self::Scene(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for EditorError {}
