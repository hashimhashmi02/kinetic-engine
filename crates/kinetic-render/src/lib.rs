//! wgpu renderer with MSAA and a post-processing stack.
//!
//! The renderer turns a [`Frame`] (a background colour and a list of coloured meshes) into
//! pixels. It knows nothing about time: evaluating a timeline into a frame happens above
//! this crate.

mod gpu;
mod headless;
mod renderer;

use std::fmt;

use kinetic_core::{DVec2, LinearRgba};
use kinetic_vector::Mesh;

pub use gpu::{AdapterChoice, Gpu};
pub use headless::render_to_rgba;
pub use renderer::{Renderer, SAMPLE_COUNT};
pub use wgpu;

/// The texture format frames are rendered in: 8-bit sRGB, so blending happens in linear
/// space and the stored bytes are what PNG files and ffmpeg expect.
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// One filled shape in a frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Geometry in scene units, relative to `offset`.
    pub mesh: Mesh,
    /// Fill colour.
    pub color: LinearRgba,
    /// Translation applied to the mesh, in scene units.
    pub offset: DVec2,
}

/// Everything needed to draw one image.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// Colour the target is cleared to.
    pub background: LinearRgba,
    /// Size of the visible scene rectangle in scene units, centred on the origin.
    pub size: DVec2,
    /// Shapes, drawn in order, later ones on top.
    pub items: Vec<Item>,
}

/// Something went wrong while setting up the GPU or reading pixels back.
#[derive(Debug)]
pub enum RenderError {
    /// No graphics adapter matched the request.
    NoAdapter(String),
    /// The adapter refused to create a device.
    Device(String),
    /// Reading the rendered image back from the GPU failed.
    Readback(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoAdapter(why) => write!(f, "no usable graphics adapter: {why}"),
            Self::Device(why) => write!(f, "could not create a graphics device: {why}"),
            Self::Readback(why) => write!(f, "could not read the rendered image back: {why}"),
        }
    }
}

impl std::error::Error for RenderError {}
