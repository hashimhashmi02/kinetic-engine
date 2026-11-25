//! Reusing tessellated meshes between frames.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::discriminant;

use kinetic_core::{DVec2, FrameState, PathEl, Shape};
use kinetic_vector::{Mesh, StrokeStyle};

use crate::lower::{lower, tessellate, Part};
use crate::Frame;

/// Everything a mesh depends on. Position, colour and opacity are not part of it, so a node
/// that only moves or fades keeps its mesh.
#[derive(Debug, Clone, PartialEq)]
struct Key {
    shape: Shape,
    scale: DVec2,
    rotation: f64,
    outline: Option<(StrokeStyle, DVec2)>,
    tolerance: f64,
}

impl Key {
    fn new(part: &Part, tolerance: f64) -> Self {
        Key {
            shape: part.shape.clone(),
            scale: part.scale,
            rotation: part.rotation,
            outline: part.outline,
            tolerance,
        }
    }

    fn matches(&self, part: &Part, tolerance: f64) -> bool {
        self.shape == *part.shape
            && self.scale == part.scale
            && self.rotation == part.rotation
            && self.outline == part.outline
            && self.tolerance == tolerance
    }
}

#[derive(Debug)]
struct Entry {
    key: Key,
    mesh: Mesh,
    /// The last frame that used this mesh.
    frame: u64,
}

/// Meshes from the previous frame, looked up by a hash of their geometry.
///
/// A hit is confirmed by comparing the full geometry, so a hash collision only costs a
/// re-tessellation and the output is always exactly that of [`Frame::from_state`]. After
/// each frame, meshes it did not use are dropped, so the cache never holds more than one
/// frame's worth.
#[derive(Debug, Default)]
pub struct MeshCache {
    entries: HashMap<u64, Entry>,
    frame: u64,
    tessellated: u64,
}

impl MeshCache {
    /// An empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same frame as [`Frame::from_state`], tessellating only the meshes whose geometry
    /// changed since the previous call.
    pub fn frame(&mut self, state: &FrameState, tolerance: f64) -> Frame {
        self.frame += 1;
        let now = self.frame;
        let (entries, tessellated) = (&mut self.entries, &mut self.tessellated);
        let frame = lower(state, |part| {
            let hash = geometry_hash(part, tolerance);
            if let Some(entry) = entries.get_mut(&hash).filter(|e| e.key.matches(part, tolerance)) {
                entry.frame = now;
                return Some(entry.mesh.clone());
            }
            *tessellated += 1;
            let mesh = tessellate(part, tolerance)?;
            let key = Key::new(part, tolerance);
            entries.insert(hash, Entry { key, mesh: mesh.clone(), frame: now });
            Some(mesh)
        });
        self.entries.retain(|_, entry| entry.frame == now);
        frame
    }

    /// Number of meshes held.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when no meshes are held.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// How many meshes have been tessellated since the cache was made.
    pub fn tessellated(&self) -> u64 {
        self.tessellated
    }
}

fn put(h: &mut DefaultHasher, x: f64) {
    // Adding zero turns -0.0 into 0.0, so values that compare equal hash equally.
    (x + 0.0).to_bits().hash(h);
}

fn put_vec(h: &mut DefaultHasher, v: DVec2) {
    put(h, v.x);
    put(h, v.y);
}

fn hash_shape(h: &mut DefaultHasher, shape: &Shape) {
    discriminant(shape).hash(h);
    match shape {
        Shape::Circle { radius } => put(h, *radius),
        Shape::Rect { size } => put_vec(h, *size),
        Shape::RoundedRect { size, radius } => {
            put_vec(h, *size);
            put(h, *radius);
        }
        Shape::Polygon { sides, radius } => {
            sides.hash(h);
            put(h, *radius);
        }
        Shape::Star { points, outer, inner } => {
            points.hash(h);
            put(h, *outer);
            put(h, *inner);
        }
        Shape::Line { from, to } => {
            put_vec(h, *from);
            put_vec(h, *to);
        }
        Shape::Path(path) => {
            for el in &path.els {
                discriminant(el).hash(h);
                match *el {
                    PathEl::MoveTo(p) | PathEl::LineTo(p) => put_vec(h, p),
                    PathEl::QuadTo(c, p) => {
                        put_vec(h, c);
                        put_vec(h, p);
                    }
                    PathEl::CubicTo(c1, c2, p) => {
                        put_vec(h, c1);
                        put_vec(h, c2);
                        put_vec(h, p);
                    }
                    PathEl::Close => {}
                }
            }
        }
        Shape::Outline { contours } => {
            for contour in contours {
                contour.len().hash(h);
                contour.iter().for_each(|&p| put_vec(h, p));
            }
        }
    }
}

/// A hash of everything in [`Key`]. `DefaultHasher::new` uses fixed keys, so it is the same
/// on every run.
fn geometry_hash(part: &Part, tolerance: f64) -> u64 {
    let mut h = DefaultHasher::new();
    hash_shape(&mut h, part.shape);
    put_vec(&mut h, part.scale);
    put(&mut h, part.rotation);
    put(&mut h, tolerance);
    if let Some((style, trim)) = part.outline {
        put(&mut h, style.width);
        discriminant(&style.cap).hash(&mut h);
        discriminant(&style.join).hash(&mut h);
        put_vec(&mut h, trim);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetic_core::color::palette;
    use kinetic_core::{Circle, Ease, Rect, Scene, Star};

    fn morph_scene() -> kinetic_core::Timeline {
        let mut s = Scene::new();
        let a = s.add(Circle::new(40.0).stroke(palette::WHITE, 3.0));
        let b = s.add(Rect::new(30.0, 30.0).at((100.0, 0.0)));
        s.parallel([a.morph_to(Star::new(5, 60.0, 25.0)), b.move_to((-100.0, 0.0))], 1.0);
        s.play(a.trim_to(0.2, 0.9), (0.5, Ease::Linear));
        s.build()
    }

    #[test]
    fn cached_frames_match_uncached_frames() {
        let tl = morph_scene();
        let mut cache = MeshCache::new();
        // Forwards, backwards and repeated times, as when scrubbing.
        for t in [0.0, 0.3, 0.3, 1.2, 0.7, 1.5, 1.5, 0.0] {
            assert_eq!(cache.frame(&tl.evaluate(t), 0.1), Frame::from_state(&tl.evaluate(t), 0.1));
        }
    }

    #[test]
    fn moving_a_node_reuses_its_mesh() {
        let mut s = Scene::new();
        s.add(Circle::new(20.0));
        let b = s.add(Rect::new(10.0, 10.0));
        s.play(b.move_to((50.0, 0.0)), 1.0);
        let tl = s.build();
        let mut cache = MeshCache::new();
        for k in 0..=10 {
            cache.frame(&tl.evaluate(f64::from(k) / 10.0), 0.1);
        }
        assert_eq!(cache.tessellated(), 2);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn changing_geometry_tessellates_again() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(20.0));
        s.play(a.scale_to(2.0), 1.0);
        let tl = s.build();
        let mut cache = MeshCache::new();
        cache.frame(&tl.evaluate(0.25), 0.1);
        cache.frame(&tl.evaluate(0.5), 0.1);
        cache.frame(&tl.evaluate(0.5), 0.1);
        assert_eq!(cache.tessellated(), 2);
    }

    #[test]
    fn unused_meshes_are_dropped() {
        let mut s = Scene::new();
        let a = s.add(Circle::new(20.0));
        s.add(Rect::new(10.0, 10.0));
        s.play(a.fade_to(0.0), 1.0);
        let tl = s.build();
        let mut cache = MeshCache::new();
        cache.frame(&tl.evaluate(0.0), 0.1);
        assert_eq!(cache.len(), 2);
        cache.frame(&tl.evaluate(1.0), 0.1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn equal_values_hash_equally() {
        let shape = Shape::Circle { radius: 0.0 };
        let part = |rotation| Part { shape: &shape, scale: DVec2::ONE, rotation, outline: None };
        assert_eq!(geometry_hash(&part(0.0), 0.1), geometry_hash(&part(-0.0), 0.1));
    }
}
