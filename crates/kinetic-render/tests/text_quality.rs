//! Text quality: the renderer's anti-aliased glyphs against an exact-coverage reference
//! rasteriser, at 1x and 2x.

mod common;

use ab_glyph_rasterizer::{point, Point, Rasterizer};
use kinetic_core::color::{palette, srgb_to_linear};
use kinetic_core::{DVec2, FrameState, PathEl, Scene, Text};
use kinetic_render::{render_to_rgba, Frame, Renderer, TARGET_FORMAT};

/// Scene units of the test image.
const SIZE: DVec2 = DVec2::new(360.0, 100.0);

fn scene() -> FrameState {
    let mut s = Scene::new();
    s.set_size(SIZE.x, SIZE.y);
    s.set_background(palette::BLACK);
    // Small text, with thin strokes and tight curves, is where anti-aliasing shows.
    s.add_text(Text::new("Kinetic aeg 0.5").size(36.0).at((0.0, 18.0)));
    s.add_text(Text::new("नमस्ते Hello").size(24.0).at((0.0, -24.0)));
    s.build().evaluate(0.0)
}

/// Coverage per pixel from the renderer: white on black, read back as linear light.
fn ours(gpu: &kinetic_render::Gpu, state: &FrameState, ppu: f64) -> Vec<f32> {
    let (w, h) = ((SIZE.x * ppu) as u32, (SIZE.y * ppu) as u32);
    let frame = Frame::from_state(state, 0.25 / ppu);
    let mut renderer = Renderer::new(&gpu.device, TARGET_FORMAT);
    let rgba = render_to_rgba(gpu, &mut renderer, w, h, &frame).expect("render");
    rgba.as_chunks::<4>().0.iter().map(|p| srgb_to_linear(f32::from(p[0]) / 255.0)).collect()
}

/// Exact area coverage of the same glyph outlines, the way font rasterisers compute it.
fn reference(state: &FrameState, ppu: f64) -> Vec<f32> {
    let (w, h) = ((SIZE.x * ppu) as usize, (SIZE.y * ppu) as usize);
    let mut raster = Rasterizer::new(w, h);
    let to_px =
        |p: DVec2| point(((p.x + SIZE.x / 2.0) * ppu) as f32, ((SIZE.y / 2.0 - p.y) * ppu) as f32);
    for node in &state.nodes {
        let at = node.props.position;
        let (mut start, mut pen): (Point, Point) = (point(0.0, 0.0), point(0.0, 0.0));
        for el in &node.shape.to_path().els {
            match *el {
                PathEl::MoveTo(p) => {
                    start = to_px(p + at);
                    pen = start;
                }
                PathEl::LineTo(p) => {
                    let p = to_px(p + at);
                    raster.draw_line(pen, p);
                    pen = p;
                }
                PathEl::QuadTo(c, p) => {
                    let p = to_px(p + at);
                    raster.draw_quad(pen, to_px(c + at), p);
                    pen = p;
                }
                PathEl::CubicTo(c1, c2, p) => {
                    let p = to_px(p + at);
                    raster.draw_cubic(pen, to_px(c1 + at), to_px(c2 + at), p);
                    pen = p;
                }
                PathEl::Close => {
                    raster.draw_line(pen, start);
                    pen = start;
                }
            }
        }
    }
    let mut out = vec![0.0; w * h];
    raster.for_each_pixel(|i, a| out[i] = a.clamp(0.0, 1.0));
    out
}

struct Quality {
    /// Total ink relative to the reference: 1 means neither bolder nor thinner.
    ink: f64,
    /// Mean coverage error over pixels on an edge.
    edge_error: f64,
    /// Largest coverage error anywhere.
    worst: f64,
    /// Partly covered pixels relative to the reference: well above 1 means blurred edges.
    softness: f64,
}

fn compare(ours: &[f32], reference: &[f32]) -> Quality {
    assert_eq!(ours.len(), reference.len());
    let ink = ours.iter().map(|&c| f64::from(c)).sum::<f64>()
        / reference.iter().map(|&c| f64::from(c)).sum::<f64>();
    let edge = |c: f32| c > 0.02 && c < 0.98;
    let errors: Vec<f64> = ours
        .iter()
        .zip(reference)
        .filter(|(a, b)| edge(**a) || edge(**b))
        .map(|(a, b)| f64::from((a - b).abs()))
        .collect();
    let edge_error = errors.iter().sum::<f64>() / errors.len().max(1) as f64;
    let edges = |c: &[f32]| c.iter().filter(|&&c| edge(c)).count() as f64;
    let softness = edges(ours) / edges(reference);
    let worst =
        ours.iter().zip(reference).map(|(a, b)| f64::from((a - b).abs())).fold(0.0, f64::max);
    Quality { ink, edge_error, worst, softness }
}

#[test]
fn text_matches_an_exact_rasteriser_at_1x_and_2x() {
    common::with_gpu(|gpu| {
        let state = scene();
        for ppu in [1.0, 2.0] {
            let q = compare(&ours(gpu, &state, ppu), &reference(&state, ppu));
            eprintln!(
                "{ppu}x: ink {:.4}, edge error {:.4}, worst {:.4}, softness {:.3}",
                q.ink, q.edge_error, q.worst, q.softness
            );
            // Glyphs are neither bolder nor thinner than the outlines.
            assert!((q.ink - 1.0).abs() < 0.01, "{ppu}x ink {:.4}", q.ink);
            // 4x MSAA has five coverage levels, so an edge pixel is off by about 0.08 on
            // average (measured); without anti-aliasing it would be about 0.25.
            assert!(q.edge_error < 0.1, "{ppu}x edge error {:.4}", q.edge_error);
            assert!(q.worst < 0.5, "{ppu}x worst {:.4}", q.worst);
            // Edges are as sharp as the reference's: no more partly covered pixels.
            assert!(q.softness < 1.1, "{ppu}x softness {:.3}", q.softness);
        }
    });
}
