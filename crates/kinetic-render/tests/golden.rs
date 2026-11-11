//! Golden image tests. Regenerate with `cargo xtask bless` and look at the result.

mod common;

use kinetic_core::color::palette;
use kinetic_core::DVec2;
use kinetic_render::{render_to_rgba, Frame, Item, Renderer, TARGET_FORMAT};
use kinetic_vector::{fill, shape};

#[test]
fn first_shapes() {
    common::with_gpu(|gpu| {
        let (w, h) = (256, 144);
        let item = |path, color, x| Item {
            mesh: fill(&path, 0.1).expect("tessellate"),
            color,
            offset: DVec2::new(x, 0.0),
        };
        let frame = Frame {
            background: palette::ink(),
            size: DVec2::new(f64::from(w), f64::from(h)),
            items: vec![
                item(shape::circle(DVec2::ZERO, 40.0), palette::blue(), -56.0),
                item(shape::rect(DVec2::ZERO, DVec2::new(84.0, 60.0)), palette::amber(), 62.0),
            ],
        };
        let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
        let image = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
        common::assert_golden("first_shapes", w, h, &image);
    });
}
