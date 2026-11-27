//! Offscreen frame rendering piped to ffmpeg.
//!
//! An export walks a timeline at a fixed frame rate ([`Frames`]); every frame's time comes
//! from its index, so a render is a pure function of the timeline and the settings.

mod export;
mod ffmpeg;
mod frames;

use std::fmt;

use kinetic_render::RenderError;

pub use export::Export;
pub use ffmpeg::{ffmpeg_program, Encoder, VideoSettings};
pub use frames::{FrameTime, Frames};
/// The device types an [`Export`] runs on, so callers need not depend on the renderer.
pub use kinetic_render::{AdapterChoice, Gpu};

/// An export could not run or did not finish.
#[derive(Debug)]
pub enum ExportError {
    /// The export settings cannot work: a bad frame rate, size, range or quality.
    Settings(String),
    /// The ffmpeg program could not be found.
    FfmpegMissing {
        /// The program that was tried.
        program: String,
    },
    /// ffmpeg could not be started, or failed while encoding. Holds what it reported.
    Ffmpeg(String),
    /// Rendering a frame failed.
    Render(RenderError),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Settings(why) => write!(f, "{why}"),
            Self::FfmpegMissing { program } => write!(
                f,
                "ffmpeg was not found (tried `{program}`). Install it and put it on PATH, set \
                 KINETIC_FFMPEG to its path, or write PNG frames instead"
            ),
            Self::Ffmpeg(why) => write!(f, "{why}"),
            Self::Render(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RenderError> for ExportError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}
