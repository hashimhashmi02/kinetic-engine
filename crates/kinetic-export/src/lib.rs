//! Offscreen frame rendering piped to ffmpeg.
//!
//! An export walks a timeline at a fixed frame rate ([`Frames`]); every frame's time comes
//! from its index, so a render is a pure function of the timeline and the settings.

mod frames;

use std::fmt;

pub use frames::{FrameTime, Frames};

/// An export could not run or did not finish.
#[derive(Debug)]
pub enum ExportError {
    /// The export settings cannot work: a bad frame rate, size, range or quality.
    Settings(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(why) => write!(f, "{why}"),
        }
    }
}

impl std::error::Error for ExportError {}
