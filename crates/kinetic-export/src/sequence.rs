//! Writing frames as a numbered sequence of PNG files.

use std::path::{Path, PathBuf};

use kinetic_core::color::{linear_to_srgb, srgb_to_linear};

use crate::{ExportError, FrameTime};

/// Writes each frame to `<dir>/frame_<index>.png`, numbered by the frame's index in the
/// full timeline with at least five digits, so a range render names its frames the way a
/// full render would. Needs no ffmpeg.
#[derive(Debug)]
pub struct PngSequence {
    dir: PathBuf,
    width: u32,
    height: u32,
    straight: Vec<u8>,
}

impl PngSequence {
    /// Creates `dir` if needed. Existing frames with the same names are overwritten.
    pub fn create(dir: &Path, width: u32, height: u32) -> Result<Self, ExportError> {
        std::fs::create_dir_all(dir).map_err(|error| ExportError::io(dir, error))?;
        Ok(Self { dir: dir.to_path_buf(), width, height, straight: Vec::new() })
    }

    /// Where frame `index` goes.
    pub fn path(&self, index: u64) -> PathBuf {
        self.dir.join(format!("frame_{index:05}.png"))
    }

    /// Writes one frame given as tightly packed sRGB RGBA8 rows with premultiplied alpha,
    /// as the renderer produces them. PNG stores straight alpha, so translucent pixels are
    /// converted first.
    pub fn write_frame(&mut self, frame: FrameTime, rgba: &[u8]) -> Result<(), ExportError> {
        let expected = self.width as usize * self.height as usize * 4;
        if rgba.len() != expected {
            return Err(ExportError::Settings(format!(
                "frame has {} bytes; the sequence expects {expected}",
                rgba.len()
            )));
        }
        self.straight.clear();
        self.straight.extend_from_slice(rgba);
        unpremultiply(&mut self.straight);

        let path = self.path(frame.index);
        let file = std::fs::File::create(&path).map_err(|e| ExportError::io(&path, e))?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        // Frames are working files for an editor or encoder: favour speed over size.
        encoder.set_compression(png::Compression::Fast);
        let as_io = |e: png::EncodingError| ExportError::io(&path, std::io::Error::other(e));
        let mut writer = encoder.write_header().map_err(as_io)?;
        writer.write_image_data(&self.straight).map_err(as_io)?;
        writer.finish().map_err(as_io)
    }
}

/// Converts sRGB-encoded premultiplied RGBA8 pixels to straight alpha in place.
///
/// The renderer premultiplies in linear light and then encodes to sRGB, so each colour is
/// decoded, divided by alpha and encoded again. Opaque pixels are left as they are and
/// fully transparent ones become all zero.
pub fn unpremultiply(rgba: &mut [u8]) {
    for p in rgba.as_chunks_mut::<4>().0 {
        match p[3] {
            255 => {}
            0 => *p = [0; 4],
            a => {
                let a = f32::from(a) / 255.0;
                for c in &mut p[..3] {
                    let linear = srgb_to_linear(f32::from(*c) / 255.0) / a;
                    *c = (linear_to_srgb(linear.min(1.0)) * 255.0).round() as u8;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::LinearRgba;

    #[test]
    fn opaque_pixels_are_untouched() {
        let mut p = [12, 200, 99, 255, 0, 0, 0, 255];
        unpremultiply(&mut p);
        assert_eq!(p, [12, 200, 99, 255, 0, 0, 0, 255]);
    }

    #[test]
    fn transparent_pixels_become_zero() {
        let mut p = [3, 2, 1, 0];
        unpremultiply(&mut p);
        assert_eq!(p, [0; 4]);
    }

    #[test]
    fn translucent_pixels_get_their_colour_back() {
        // What the renderer stores for each colour at each alpha: linear premultiplied,
        // then sRGB-encoded per channel.
        for colour in [[1.0, 1.0, 1.0], [0.9, 0.3, 0.05], [0.2, 0.5, 0.8]] {
            for alpha in [0.25f32, 0.5, 0.8] {
                let stored = |c: f32| (linear_to_srgb(c * alpha) * 255.0).round() as u8;
                let mut p = [
                    stored(colour[0]),
                    stored(colour[1]),
                    stored(colour[2]),
                    (alpha * 255.0).round() as u8,
                ];
                unpremultiply(&mut p);
                let want = LinearRgba::from_linear(colour[0], colour[1], colour[2], alpha);
                let want = want.to_srgba8();
                for (got, want) in p.iter().zip(want) {
                    assert!(got.abs_diff(want) <= 2, "{colour:?} at {alpha}: {p:?} vs {want:?}");
                }
            }
        }
    }

    #[test]
    fn frames_are_named_by_their_index() {
        let seq = PngSequence { dir: "out".into(), width: 1, height: 1, straight: Vec::new() };
        assert_eq!(seq.path(7), Path::new("out").join("frame_00007.png"));
        assert_eq!(seq.path(123_456), Path::new("out").join("frame_123456.png"));
    }
}
