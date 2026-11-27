//! PNG sequences: one file per frame, holding exactly the rendered pixels.

mod common;

use std::path::Path;

use kinetic_core::color::palette;
use kinetic_core::{Ease, LinearRgba, Rect, Scene, Timeline};
use kinetic_export::{Export, Frames, PngSequence};

/// A square sliding right; it starts off the pixel grid, so its edges are anti-aliased.
fn slide(background: LinearRgba) -> Timeline {
    let mut s = Scene::new();
    s.set_size(64.0, 36.0);
    s.set_background(background);
    let square = s.add(Rect::new(10.0, 10.0).fill(palette::WHITE).at((-19.6, 0.3)));
    s.play(square.move_to((20.0, 0.0)), (1.0, Ease::Linear));
    s.build()
}

fn read_png(path: &Path) -> (u32, u32, Vec<u8>) {
    let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("open {path:?}: {e}"));
    let mut reader =
        png::Decoder::new(std::io::BufReader::new(file)).read_info().expect("png info");
    let mut data = vec![0; reader.output_buffer_size().expect("png size")];
    let info = reader.next_frame(&mut data).expect("png frame");
    data.truncate(info.buffer_size());
    (info.width, info.height, data)
}

fn out_dir(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("png").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn each_frame_becomes_a_file_with_the_rendered_pixels() {
    common::with_gpu(|gpu| {
        let timeline = slide(palette::ink());
        let frames = Frames::new(timeline.duration, 4.0).expect("frames").range(0.5, 1.0);
        let export = Export::new(&timeline, frames.expect("range"), 64, 36).expect("export");
        let dir = out_dir("opaque");
        let mut sequence = PngSequence::create(&dir, 64, 36).expect("create");
        let mut rendered = Vec::new();
        export
            .run(gpu, |frame, rgba| {
                rendered.push((frame.index, rgba.to_vec()));
                sequence.write_frame(frame, rgba)
            })
            .expect("run");

        let mut names: Vec<_> = std::fs::read_dir(&dir)
            .expect("list")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, ["frame_00002.png", "frame_00003.png", "frame_00004.png"]);
        // An opaque frame is stored byte for byte.
        for (index, rgba) in rendered {
            let (w, h, stored) = read_png(&sequence.path(index));
            assert_eq!((w, h), (64, 36));
            assert!(stored == rgba, "frame {index} differs from what was rendered");
        }
    });
}

#[test]
fn a_transparent_background_stays_transparent() {
    common::with_gpu(|gpu| {
        let timeline = slide(LinearRgba::from_linear(0.0, 0.0, 0.0, 0.0));
        let frames = Frames::new(timeline.duration, 1.0).expect("frames").range(0.0, 0.0);
        let export = Export::new(&timeline, frames.expect("range"), 64, 36).expect("export");
        let dir = out_dir("transparent");
        let mut sequence = PngSequence::create(&dir, 64, 36).expect("create");
        export.run(gpu, |frame, rgba| sequence.write_frame(frame, rgba)).expect("run");

        let (_, _, stored) = read_png(&sequence.path(0));
        let pixel = |x: usize, y: usize| {
            let at = (y * 64 + x) * 4;
            [stored[at], stored[at + 1], stored[at + 2], stored[at + 3]]
        };
        assert_eq!(pixel(0, 0), [0, 0, 0, 0], "background");
        // The square starts about 20 units left of centre: pixel x = 12.
        assert_eq!(pixel(12, 18), [255, 255, 255, 255], "square");
        // Its anti-aliased edge is white with partial alpha, not darkened toward black.
        let edge: Vec<_> = (0..64)
            .flat_map(|x| (0..36).map(move |y| (x, y)))
            .map(|(x, y)| pixel(x, y))
            .filter(|p| p[3] > 0 && p[3] < 255)
            .collect();
        assert!(edge.len() > 10, "only {} edge pixels", edge.len());
        assert!(edge.iter().all(|p| p[0] >= 250), "edge pixels: {edge:?}");
    });
}
