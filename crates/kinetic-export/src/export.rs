//! Rendering a timeline frame by frame.

use kinetic_core::{DVec2, Timeline};
use kinetic_render::{Frame, Gpu, MeshCache, Offscreen, Renderer, TARGET_FORMAT};

use crate::{ExportError, FrameTime, Frames};

/// Geometry is flattened to within this many pixels of the true curves, as in the editor.
const TOLERANCE_PX: f64 = 0.25;

/// One render of a timeline: which frames, at what pixel size.
///
/// The scene is never stretched. When the image's shape differs from the scene's, the
/// scene's rectangle is fitted inside the image and more of the scene shows around it.
#[derive(Debug, Clone, Copy)]
pub struct Export<'a> {
    timeline: &'a Timeline,
    frames: Frames,
    width: u32,
    height: u32,
}

impl<'a> Export<'a> {
    /// Renders `frames` of `timeline` at `width` x `height` pixels.
    pub fn new(
        timeline: &'a Timeline,
        frames: Frames,
        width: u32,
        height: u32,
    ) -> Result<Self, ExportError> {
        if width == 0 || height == 0 {
            return Err(ExportError::Settings(format!(
                "image size {width}x{height} has no pixels"
            )));
        }
        let scene = timeline.size;
        if !(scene.x > 0.0 && scene.y > 0.0 && scene.is_finite()) {
            return Err(ExportError::Settings(format!(
                "the scene's size {}x{} cannot be shown",
                scene.x, scene.y
            )));
        }
        Ok(Self { timeline, frames, width, height })
    }

    /// The frames this export renders.
    pub fn frames(&self) -> Frames {
        self.frames
    }

    /// Image width and height in pixels.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Pixels per scene unit: the largest scale at which the whole scene fits the image.
    pub fn pixels_per_unit(&self) -> f64 {
        let scene = self.timeline.size;
        (f64::from(self.width) / scene.x).min(f64::from(self.height) / scene.y)
    }

    /// The rectangle of scene units the image shows, centred on the origin. It contains the
    /// scene's own rectangle and has the image's shape.
    pub fn view(&self) -> DVec2 {
        DVec2::new(f64::from(self.width), f64::from(self.height)) / self.pixels_per_unit()
    }

    /// What to draw at `time`. Pure apart from the cache, which only saves work.
    pub fn frame(&self, time: f64, cache: &mut MeshCache) -> Frame {
        let state = self.timeline.evaluate(time);
        let mut frame = cache.frame(&state, TOLERANCE_PX / self.pixels_per_unit());
        frame.size = self.view();
        frame
    }

    /// Renders every frame in order and hands each one to `write` as tightly packed sRGB
    /// RGBA8 rows, top to bottom. Stops at the first error.
    pub fn run(
        &self,
        gpu: &Gpu,
        mut write: impl FnMut(FrameTime, &[u8]) -> Result<(), ExportError>,
    ) -> Result<(), ExportError> {
        let target = Offscreen::new(gpu, self.width, self.height)?;
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let mut cache = MeshCache::new();
        let mut pixels = Vec::new();
        for frame_time in self.frames.iter() {
            let frame = self.frame(frame_time.time, &mut cache);
            target.render(gpu, &mut renderer, &frame, &mut pixels)?;
            write(frame_time, &pixels)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::Scene;

    fn timeline(width: f64, height: f64) -> Timeline {
        let mut s = Scene::new();
        s.set_size(width, height);
        s.wait(1.0);
        s.build()
    }

    fn export(timeline: &Timeline, width: u32, height: u32) -> Export<'_> {
        let frames = Frames::new(timeline.duration, 30.0).expect("frames");
        Export::new(timeline, frames, width, height).expect("export")
    }

    #[test]
    fn same_shape_shows_exactly_the_scene() {
        let tl = timeline(1280.0, 720.0);
        let e = export(&tl, 1920, 1080);
        assert_eq!(e.pixels_per_unit(), 1.5);
        assert_eq!(e.view(), DVec2::new(1280.0, 720.0));
    }

    #[test]
    fn a_square_image_shows_more_above_and_below() {
        let tl = timeline(1280.0, 720.0);
        let e = export(&tl, 640, 640);
        assert_eq!(e.pixels_per_unit(), 0.5);
        assert_eq!(e.view(), DVec2::new(1280.0, 1280.0));
    }

    #[test]
    fn a_wide_image_shows_more_at_the_sides() {
        let tl = timeline(1000.0, 1000.0);
        let e = export(&tl, 400, 200);
        assert_eq!(e.view(), DVec2::new(2000.0, 1000.0));
    }

    #[test]
    fn frames_use_the_fitted_view() {
        let tl = timeline(1280.0, 720.0);
        let e = export(&tl, 720, 720);
        let frame = e.frame(0.5, &mut MeshCache::new());
        assert_eq!(frame.size, e.view());
        assert_eq!(frame.background, tl.background);
    }

    #[test]
    fn empty_images_are_refused() {
        let tl = timeline(1280.0, 720.0);
        let frames = Frames::new(1.0, 30.0).expect("frames");
        assert!(Export::new(&tl, frames, 0, 720).is_err());
    }
}
