//! Rendering a timeline frame by frame on a real (software) GPU.

mod common;

use kinetic_core::color::palette;
use kinetic_core::{Ease, Rect, Scene, Timeline};
use kinetic_export::{Export, ExportError, Frames};

/// A white square sliding from the left edge to the right edge over one second.
fn slide() -> Timeline {
    let mut s = Scene::new();
    s.set_size(160.0, 90.0);
    s.set_background(palette::BLACK);
    let square = s.add(Rect::new(20.0, 20.0).fill(palette::WHITE).at((-60.0, 0.0)));
    s.play(square.move_to((60.0, 0.0)), (1.0, Ease::Linear));
    s.build()
}

/// Mean x of the white pixels in an RGBA image `width` pixels wide.
fn centre_x(rgba: &[u8], width: usize) -> f64 {
    let (mut sum, mut count) = (0.0, 0.0);
    for (i, p) in rgba.as_chunks::<4>().0.iter().enumerate() {
        let weight = f64::from(p[0]) / 255.0;
        sum += (i % width) as f64 * weight;
        count += weight;
    }
    sum / count
}

#[test]
fn every_frame_is_rendered_in_order() {
    common::with_gpu(|gpu| {
        let timeline = slide();
        let frames = Frames::new(timeline.duration, 10.0).expect("frames");
        let export = Export::new(&timeline, frames, 160, 90).expect("export");
        let mut seen = Vec::new();
        export
            .run(gpu, |frame, rgba| {
                assert_eq!(rgba.len(), 160 * 90 * 4);
                seen.push((frame.index, centre_x(rgba, 160)));
                Ok(())
            })
            .expect("run");
        assert_eq!(seen.iter().map(|s| s.0).collect::<Vec<_>>(), (0..=10).collect::<Vec<_>>());
        // The square moves 12 pixels a frame: from x = 20 to x = 140 in image pixels.
        for (index, x) in &seen {
            let want = 20.0 + 12.0 * *index as f64 - 0.5;
            assert!((x - want).abs() < 0.6, "frame {index}: centre {x}, want {want}");
        }
    });
}

#[test]
fn a_larger_image_scales_the_scene_up() {
    common::with_gpu(|gpu| {
        let timeline = slide();
        let frames = Frames::new(timeline.duration, 10.0).expect("frames").range(0.5, 0.5);
        let export = Export::new(&timeline, frames.expect("range"), 320, 180).expect("export");
        export
            .run(gpu, |_, rgba| {
                // Half way the square is centred, and at 2x it is 40 pixels wide.
                assert!((centre_x(rgba, 320) - 159.5).abs() < 0.6);
                let white = rgba.as_chunks::<4>().0.iter().filter(|p| p[0] == 255).count();
                assert_eq!(white, 40 * 40);
                Ok(())
            })
            .expect("run");
    });
}

#[test]
fn an_error_from_the_writer_stops_the_render() {
    common::with_gpu(|gpu| {
        let timeline = slide();
        let frames = Frames::new(timeline.duration, 10.0).expect("frames");
        let export = Export::new(&timeline, frames, 32, 18).expect("export");
        let mut calls = 0;
        let result = export.run(gpu, |frame, _| {
            calls += 1;
            if frame.index == 3 {
                Err(ExportError::Settings("disk full".into()))
            } else {
                Ok(())
            }
        });
        assert!(result.expect_err("stopped").to_string().contains("disk full"));
        assert_eq!(calls, 4);
    });
}
