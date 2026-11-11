//! Rendering to memory, for export and tests.

use crate::{Frame, Gpu, RenderError, Renderer, TARGET_FORMAT};

/// Renders `frame` at `width` x `height` pixels and returns the image as tightly packed
/// RGBA8 rows, top to bottom, sRGB-encoded with premultiplied alpha.
///
/// `renderer` must have been created for [`TARGET_FORMAT`]. A zero dimension gives an
/// empty image.
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
    let extent = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
    let target = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("kinetic offscreen"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&Default::default());

    // Rows in a texture-to-buffer copy must start on a 256-byte boundary.
    let row = width as usize * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
    let padded = row.div_ceil(align) * align;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("kinetic readback"),
        size: (padded * height as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    renderer.draw(&gpu.device, &gpu.queue, &mut encoder, &view, (width, height), frame);
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded as u32),
                rows_per_image: Some(height),
            },
        },
        extent,
    );
    gpu.queue.submit([encoder.finish()]);

    let (sender, receiver) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
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

    let mapped =
        buffer.slice(..).get_mapped_range().map_err(|e| RenderError::Readback(e.to_string()))?;
    let mut pixels = Vec::with_capacity(row * height as usize);
    for line in mapped.chunks(padded) {
        pixels.extend_from_slice(&line[..row]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
}
