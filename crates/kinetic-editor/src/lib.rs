//! Preview viewport, timeline scrubber and inspector.

mod app;
pub mod demo;

use std::fmt;

pub use app::run;

/// The preview window could not be opened or lost its graphics device.
#[derive(Debug)]
pub enum EditorError {
    /// The window or its drawing surface could not be created.
    Window(String),
    /// The renderer could not be set up.
    Render(kinetic_render::RenderError),
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Window(why) => write!(f, "could not open the preview window: {why}"),
            Self::Render(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for EditorError {}
