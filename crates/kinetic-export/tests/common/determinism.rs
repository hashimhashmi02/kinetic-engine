//! Determinism: rendering a scene twice gives the same bytes in every frame.
//!
//! The scene is built twice, from scratch, and every frame of both builds is rendered
//! through a fresh export (its own renderer, target and mesh cache). The frame hashes must
//! match one for one, and a range rendered on its own must reproduce the same frames of
//! the full render.
//!
//! Each example scene gets its own test binary: scene crates all export the same C entry
//! points, so two of them cannot be linked into one program.

use kinetic_core::Timeline;
use kinetic_export::{frame_hash, Export, Frames, Gpu};

/// Small frames keep the software rasteriser quick; determinism does not depend on size.
pub const SIZE: (u32, u32) = (192, 108);
pub const FPS: f64 = 30.0;

pub fn hashes(gpu: &Gpu, timeline: &Timeline, frames: Frames) -> Vec<(u64, u64)> {
    let export = Export::new(timeline, frames, SIZE.0, SIZE.1).expect("export");
    let mut hashes = Vec::with_capacity(frames.len() as usize);
    export
        .run(gpu, |frame, rgba| {
            hashes.push((frame.index, frame_hash(rgba)));
            Ok(())
        })
        .expect("render");
    hashes
}

/// Builds a scene twice and checks that every frame of both builds hashes the same, and
/// that a range rendered on its own reproduces the full render.
pub fn renders_the_same_twice(name: &str, build: fn() -> Timeline) {
    super::with_gpu(|gpu| {
        let (first, second) = (build(), build());
        assert_eq!(first, second, "{name}: the scene builds the same timeline twice");
        let frames = Frames::new(first.duration, FPS).expect("frames");
        let a = hashes(gpu, &first, frames);
        let b = hashes(gpu, &second, frames);
        assert_eq!(a.len() as u64, frames.len());
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x, y, "{name}: frame {} differs between runs", x.0);
        }

        // Frames from the middle, rendered on their own, match the full render.
        let middle = frames.range(first.duration * 0.4, first.duration * 0.6).expect("range");
        let part = hashes(gpu, &first, middle);
        let start = middle.first() as usize;
        assert_eq!(part, a[start..start + part.len()], "{name}: a range render differs");

        // Something moves, so equal hashes are not equal blanks.
        let distinct: std::collections::BTreeSet<_> = a.iter().map(|h| h.1).collect();
        assert!(distinct.len() > a.len() / 4, "{name}: only {} distinct frames", distinct.len());
        eprintln!("{name}: {} frames identical across runs", a.len());
    });
}
