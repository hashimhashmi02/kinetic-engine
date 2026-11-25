//! Triangle meshes produced by tessellating paths.

use std::fmt;

use kinetic_core::{LineCap, LineJoin};
use kurbo::{BezPath, PathEl};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, VertexBuffers,
};

/// An indexed triangle list in scene units.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// Vertex positions.
    pub vertices: Vec<[f32; 2]>,
    /// Triangles, three indices each, into `vertices`.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// True when the mesh has no triangles.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Total area covered by the triangles, in square scene units.
    pub fn area(&self) -> f64 {
        self.indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|t| {
                let [a, b, c] = t.map(|i| self.vertices[i as usize]);
                let cross = f64::from(b[0] - a[0]) * f64::from(c[1] - a[1])
                    - f64::from(c[0] - a[0]) * f64::from(b[1] - a[1]);
                cross.abs() / 2.0
            })
            .sum()
    }
}

/// A path could not be tessellated.
#[derive(Debug, Clone, PartialEq)]
pub struct TessellateError(String);

impl fmt::Display for TessellateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not tessellate path: {}", self.0)
    }
}

impl std::error::Error for TessellateError {}

fn to_lyon(path: &BezPath) -> Path {
    let p = |q: kurbo::Point| point(q.x as f32, q.y as f32);
    let mut builder = Path::builder();
    let mut open = false;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(a) => {
                if open {
                    builder.end(false);
                }
                builder.begin(p(a));
                open = true;
            }
            // Drawing commands before any MoveTo have no start point; skip them.
            _ if !open => {}
            PathEl::LineTo(a) => {
                builder.line_to(p(a));
            }
            PathEl::QuadTo(c, a) => {
                builder.quadratic_bezier_to(p(c), p(a));
            }
            PathEl::CurveTo(c1, c2, a) => {
                builder.cubic_bezier_to(p(c1), p(c2), p(a));
            }
            PathEl::ClosePath => {
                builder.end(true);
                open = false;
            }
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

/// Tessellates the interior of `path` (non-zero fill rule) into triangles.
///
/// `tolerance` is the largest allowed distance between a curve and the straight segments
/// that approximate it, in scene units. Open subpaths are filled as if closed.
pub fn fill(path: &BezPath, tolerance: f64) -> Result<Mesh, TessellateError> {
    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let options = FillOptions::tolerance(tolerance as f32)
        .with_fill_rule(lyon_tessellation::FillRule::NonZero);
    FillTessellator::new()
        .tessellate_path(
            &to_lyon(path),
            &options,
            &mut BuffersBuilder::new(&mut buffers, |v: FillVertex| v.position().to_array()),
        )
        .map_err(|e| TessellateError(format!("{e:?}")))?;
    Ok(Mesh { vertices: buffers.vertices, indices: buffers.indices })
}

/// How an outline is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeStyle {
    /// Width in scene units.
    pub width: f64,
    /// How the ends of open subpaths are drawn.
    pub cap: LineCap,
    /// How corners are drawn.
    pub join: LineJoin,
}

/// Miter joins longer than this many half-widths are cut off, so very sharp corners do
/// not grow long spikes.
const MITER_LIMIT: f32 = 4.0;

/// Tessellates the outline of `path` drawn with `style` into triangles. A zero width gives
/// an empty mesh.
pub fn stroke(
    path: &BezPath,
    style: &StrokeStyle,
    tolerance: f64,
) -> Result<Mesh, TessellateError> {
    if style.width <= 0.0 {
        return Ok(Mesh::default());
    }
    let cap = match style.cap {
        LineCap::Butt => lyon_tessellation::LineCap::Butt,
        LineCap::Round => lyon_tessellation::LineCap::Round,
        LineCap::Square => lyon_tessellation::LineCap::Square,
    };
    let join = match style.join {
        LineJoin::Miter => lyon_tessellation::LineJoin::Miter,
        LineJoin::Round => lyon_tessellation::LineJoin::Round,
        LineJoin::Bevel => lyon_tessellation::LineJoin::Bevel,
    };
    let options = StrokeOptions::tolerance(tolerance as f32)
        .with_line_width(style.width as f32)
        .with_line_cap(cap)
        .with_line_join(join)
        .with_miter_limit(MITER_LIMIT);
    let mut buffers: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    StrokeTessellator::new()
        .tessellate_path(
            &to_lyon(path),
            &options,
            &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex| v.position().to_array()),
        )
        .map_err(|e| TessellateError(format!("{e:?}")))?;
    Ok(Mesh { vertices: buffers.vertices, indices: buffers.indices })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use kinetic_core::DVec2;

    #[test]
    fn rect_fills_to_its_exact_area() {
        let mesh = fill(&shape::rect(DVec2::ZERO, DVec2::new(200.0, 120.0)), 0.1).unwrap();
        assert_eq!(mesh.indices.len() % 3, 0);
        assert!((mesh.area() - 24_000.0).abs() < 1e-3, "area was {}", mesh.area());
    }

    #[test]
    fn circle_area_converges_with_tolerance() {
        let exact = std::f64::consts::PI * 80.0 * 80.0;
        let coarse = fill(&shape::circle(DVec2::ZERO, 80.0), 2.0).unwrap();
        let fine = fill(&shape::circle(DVec2::ZERO, 80.0), 0.02).unwrap();
        assert!(fine.vertices.len() > coarse.vertices.len());
        assert!((fine.area() - exact).abs() < (coarse.area() - exact).abs());
        assert!((fine.area() - exact).abs() / exact < 1e-3);
    }

    #[test]
    fn indices_stay_in_range() {
        let mesh = fill(&shape::circle(DVec2::new(5.0, 5.0), 30.0), 0.25).unwrap();
        assert!(mesh.indices.iter().all(|&i| (i as usize) < mesh.vertices.len()));
    }

    #[test]
    fn empty_path_gives_empty_mesh() {
        let mesh = fill(&BezPath::new(), 0.1).unwrap();
        assert!(mesh.is_empty());
        assert_eq!(mesh.area(), 0.0);
    }

    #[test]
    fn tessellation_is_deterministic() {
        let path = shape::circle(DVec2::ZERO, 64.0);
        assert_eq!(fill(&path, 0.1).unwrap(), fill(&path, 0.1).unwrap());
    }

    fn line() -> BezPath {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((100.0, 0.0));
        p
    }

    fn style(width: f64, cap: LineCap, join: LineJoin) -> StrokeStyle {
        StrokeStyle { width, cap, join }
    }

    #[test]
    fn caps_extend_an_open_line_by_the_expected_area() {
        let area = |cap| stroke(&line(), &style(10.0, cap, LineJoin::Miter), 0.01).unwrap().area();
        assert!((area(LineCap::Butt) - 1000.0).abs() < 1e-3);
        assert!((area(LineCap::Square) - 1100.0).abs() < 1e-3);
        let round = 1000.0 + std::f64::consts::PI * 25.0;
        assert!((area(LineCap::Round) - round).abs() < 0.5, "{}", area(LineCap::Round));
    }

    #[test]
    fn joins_shape_the_corners_of_a_closed_square() {
        let square = shape::rect(DVec2::ZERO, DVec2::new(100.0, 100.0));
        let area = |join| stroke(&square, &style(10.0, LineCap::Butt, join), 0.01).unwrap().area();
        // A 110 by 110 square minus a 90 by 90 hole, less what each join cuts off.
        let miter = 110.0 * 110.0 - 90.0 * 90.0;
        assert!((area(LineJoin::Miter) - miter).abs() < 1.0, "{}", area(LineJoin::Miter));
        assert!((area(LineJoin::Bevel) - (miter - 4.0 * 12.5)).abs() < 1.0);
        let round = miter - 4.0 * (25.0 - std::f64::consts::PI * 25.0 / 4.0);
        assert!((area(LineJoin::Round) - round).abs() < 1.0, "{}", area(LineJoin::Round));
    }

    #[test]
    fn zero_width_draws_nothing() {
        let mesh = stroke(&line(), &style(0.0, LineCap::Round, LineJoin::Round), 0.1).unwrap();
        assert!(mesh.is_empty());
    }
}
