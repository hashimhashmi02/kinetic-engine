//! Offscreen frame rendering piped to ffmpeg.
//!
//! An export walks a timeline at a fixed frame rate ([`Frames`]); every frame's time comes
//! from its index, so a render is a pure function of the timeline and the settings.

mod export;
mod ffmpeg;
mod frames;
mod hash;
mod sequence;

use std::fmt;
use std::path::{Path, PathBuf};

use kinetic_render::RenderError;

pub use export::Export;
pub use ffmpeg::{ffmpeg_program, Encoder, VideoSettings};
pub use frames::{FrameTime, Frames};
pub use hash::frame_hash;
/// The device types an [`Export`] runs on, so callers need not depend on the renderer.
pub use kinetic_render::{wgpu, AdapterChoice, Gpu};
pub use sequence::{unpremultiply, PngSequence};

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
    /// Reading or writing a file failed.
    Io {
        /// The file or folder involved.
        path: PathBuf,
        /// What went wrong.
        error: std::io::Error,
    },
}

impl ExportError {
    pub(crate) fn io(path: &Path, error: std::io::Error) -> Self {
        Self::Io { path: path.to_path_buf(), error }
    }
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
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            Self::Io { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl From<RenderError> for ExportError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}
