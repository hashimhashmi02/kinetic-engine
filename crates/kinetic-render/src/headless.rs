//! Rendering to memory, for export and tests.

use crate::{Frame, Gpu, RenderError, Renderer, TARGET_FORMAT};

/// Bytes per row of a texture-to-buffer copy for an image `width` pixels wide: rows of
/// RGBA8 pixels, rounded up to wgpu's 256-byte copy alignment.
pub fn padded_row_bytes(width: u32) -> usize {
    let row = width as usize * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    row.div_ceil(align) * align
}

/// An offscreen render target and the buffer it is read back through.
///
/// Both are allocated once, so rendering many frames of the same size (an export) costs
/// no allocation per frame beyond the frame's own geometry.
#[derive(Debug)]
pub struct Offscreen {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    buffer: wgpu::Buffer,
}

impl Offscreen {
    /// A `width` x `height` target in [`TARGET_FORMAT`].
    ///
    /// Fails if either dimension is zero or larger than the device allows.
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Result<Self, RenderError> {
        let max = gpu.device.limits().max_texture_dimension_2d;
        if width == 0 || height == 0 || width > max || height > max {
            return Err(RenderError::Size(format!(
                "{width}x{height} pixels; each side must be between 1 and {max} on this device"
            )));
        }
        let extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("kinetic offscreen"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kinetic readback"),
            size: (padded_row_bytes(width) * height as usize) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        Ok(Self { width, height, texture, view, buffer })
    }

    /// Width and height in pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Renders `frame` and replaces the contents of `pixels` with the image: tightly packed
    /// RGBA8 rows, top to bottom, sRGB-encoded with premultiplied alpha.
    ///
    /// `renderer` must have been created for [`TARGET_FORMAT`].
    pub fn render(
        &self,
        gpu: &Gpu,
        renderer: &mut Renderer,
        frame: &Frame,
        pixels: &mut Vec<u8>,
    ) -> Result<(), RenderError> {
        let (width, height) = (self.width, self.height);
        let padded = padded_row_bytes(width);
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        renderer.draw(&gpu.device, &gpu.queue, &mut encoder, &self.view, (width, height), frame);
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &self.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded as u32),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        gpu.queue.submit([encoder.finish()]);

        let (sender, receiver) = std::sync::mpsc::channel();
        self.buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            // The receiver only goes away if this function already returned an error.
            let _ = sender.send(result);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        receiver
            .recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(e.to_string()))?;

        // Each row arrives padded to the copy alignment; keep only the pixels.
        let row = width as usize * 4;
        pixels.clear();
        pixels.reserve(row * height as usize);
        {
            let mapped = self
                .buffer
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(e.to_string()))?;
            for line in mapped.chunks(padded) {
                pixels.extend_from_slice(&line[..row]);
            }
        }
        self.buffer.unmap();
        Ok(())
    }
}

/// Renders `frame` at `width` x `height` pixels and returns the image as tightly packed
/// RGBA8 rows, top to bottom, sRGB-encoded with premultiplied alpha.
///
/// `renderer` must have been created for [`TARGET_FORMAT`]. A zero dimension gives an
/// empty image. To render many frames, keep an [`Offscreen`] instead.
pub fn render_to_rgba(
    gpu: &Gpu,
    renderer: &mut Renderer,
    width: u32,
    height: u32,
    frame: &Frame,
) -> Result<Vec<u8>, RenderError> {
    if width == 0 || height == 0 {
        return Ok(Vec::new());
    }
    let mut pixels = Vec::new();
    Offscreen::new(gpu, width, height)?.render(gpu, renderer, frame, &mut pixels)?;
    Ok(pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_padded_to_the_copy_alignment() {
        assert_eq!(padded_row_bytes(1), 256);
        assert_eq!(padded_row_bytes(64), 256);
        assert_eq!(padded_row_bytes(65), 512);
        assert_eq!(padded_row_bytes(333), 1536);
        assert_eq!(padded_row_bytes(1920), 7680);
    }
}
