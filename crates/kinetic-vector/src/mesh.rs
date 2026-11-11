//! Triangle meshes produced by tessellating paths.

use std::fmt;

use kurbo::{BezPath, PathEl};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};

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
}
