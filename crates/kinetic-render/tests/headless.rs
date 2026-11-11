//! Offscreen rendering: correct pixels, correct layout, same bytes every time.

mod common;

use kinetic_core::color::palette;
use kinetic_core::{DVec2, LinearRgba};
use kinetic_render::{render_to_rgba, Frame, Item, Renderer, TARGET_FORMAT};
use kinetic_vector::{fill, shape};

const W: u32 = 256;
const H: u32 = 144;

fn shapes() -> Frame {
    let item = |path, color, x| Item {
        mesh: fill(&path, 0.1).expect("tessellate"),
        color,
        offset: DVec2::new(x, 0.0),
    };
    Frame {
        background: palette::ink(),
        size: DVec2::new(f64::from(W), f64::from(H)),
        items: vec![
            item(shape::circle(DVec2::ZERO, 40.0), palette::blue(), -56.0),
            item(shape::rect(DVec2::ZERO, DVec2::new(84.0, 60.0)), palette::amber(), 62.0),
        ],
    }
}

fn pixel(image: &[u8], x: u32, y: u32) -> [u8; 4] {
    let at = ((y * W + x) * 4) as usize;
    [image[at], image[at + 1], image[at + 2], image[at + 3]]
}

fn close(a: [u8; 4], b: [u8; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 1)
}

#[test]
fn draws_shapes_where_the_scene_puts_them() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, W, H, &shapes()).expect("render");
        assert_eq!(image.len(), (W * H * 4) as usize);

        // Scene x = -56 is 56 pixels left of centre; rows run top to bottom.
        assert!(close(pixel(&image, 2, 2), palette::ink().to_srgba8()), "corner is background");
        assert!(close(pixel(&image, 128 - 56, 72), palette::blue().to_srgba8()), "circle centre");
        assert!(close(pixel(&image, 128 + 62, 72), palette::amber().to_srgba8()), "rect centre");
        // Just outside the circle's top edge (radius 40) is background again.
        assert!(close(pixel(&image, 128 - 56, 72 - 44), palette::ink().to_srgba8()));
    });
}

#[test]
fn positive_y_is_up() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let frame = Frame {
            background: palette::BLACK,
            size: DVec2::new(f64::from(W), f64::from(H)),
            items: vec![Item {
                mesh: fill(&shape::rect(DVec2::ZERO, DVec2::new(20.0, 20.0)), 0.1)
                    .expect("tessellate"),
                color: palette::WHITE,
                offset: DVec2::new(0.0, 40.0),
            }],
        };
        let image = render_to_rgba(gpu, &mut renderer, W, H, &frame).expect("render");
        assert_eq!(pixel(&image, 128, 72 - 40), [255, 255, 255, 255]);
        assert_eq!(pixel(&image, 128, 72 + 40), [0, 0, 0, 255]);
    });
}

#[test]
fn edges_are_antialiased() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, W, H, &shapes()).expect("render");
        let (bg, blue) = (palette::ink().to_srgba8(), palette::blue().to_srgba8());
        // Somewhere along the circle's edge there are pixels that are neither colour.
        let blended = (0..W * H)
            .map(|i| pixel(&image, i % W, i / W))
            .filter(|p| p[2] > bg[2] + 8 && p[2] < blue[2] - 8 && p[0] < 100)
            .count();
        assert!(blended > 20, "only {blended} blended edge pixels");
    });
}

#[test]
fn half_transparent_fill_blends_in_linear_space() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let frame = Frame {
            background: palette::BLACK,
            size: DVec2::new(f64::from(W), f64::from(H)),
            items: vec![Item {
                mesh: fill(&shape::rect(DVec2::ZERO, DVec2::new(100.0, 100.0)), 0.1)
                    .expect("tessellate"),
                color: LinearRgba::from_linear(1.0, 1.0, 1.0, 0.5),
                offset: DVec2::ZERO,
            }],
        };
        let image = render_to_rgba(gpu, &mut renderer, W, H, &frame).expect("render");
        // 50% linear white over black is sRGB 188, not 128.
        let p = pixel(&image, 128, 72);
        assert!(p[0].abs_diff(188) <= 1, "got {p:?}");
    });
}

#[test]
fn rendering_twice_gives_identical_bytes() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let first = render_to_rgba(gpu, &mut renderer, W, H, &shapes()).expect("render");
        let second = render_to_rgba(gpu, &mut renderer, W, H, &shapes()).expect("render");
        assert_eq!(first, second);
    });
}

#[test]
fn width_that_needs_row_padding_reads_back_cleanly() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let frame =
            Frame { background: palette::red(), size: DVec2::new(30.0, 10.0), items: vec![] };
        // 30 * 4 = 120 bytes per row, padded to 256 on the GPU side.
        let image = render_to_rgba(gpu, &mut renderer, 30, 10, &frame).expect("render");
        assert_eq!(image.len(), 30 * 10 * 4);
        let want = palette::red().to_srgba8();
        assert!(image.as_chunks::<4>().0.iter().all(|p| close(*p, want)));
    });
}

#[test]
fn zero_size_gives_empty_image() {
    common::with_gpu(|gpu| {
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        assert!(render_to_rgba(gpu, &mut renderer, 0, 10, &shapes()).expect("render").is_empty());
    });
}
