//! Nodes in a scene and the properties they expose.

use glam::DVec2;
use serde::{Deserialize, Serialize};

use crate::color::{palette, LinearRgba};
use crate::path::PathData;
pub use crate::shape::Shape;
use crate::value::Value;

/// Identifies a node within one timeline. Ids are dense indices in creation order, which
/// is also the default drawing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// An animatable property of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PropertyId {
    /// Centre of the node in scene units, +y up. A `Vec2`.
    Position,
    /// Scale factor per axis. A `Vec2`.
    Scale,
    /// Rotation in radians, counter-clockwise. A `Scalar`.
    Rotation,
    /// Opacity in `[0, 1]`. A `Scalar`.
    Opacity,
    /// Fill colour. A `Color`.
    Fill,
}

/// Every animatable property of a node, resolved to a value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Props {
    pub position: DVec2,
    pub scale: DVec2,
    pub rotation: f64,
    pub opacity: f64,
    pub fill: LinearRgba,
}

impl Default for Props {
    fn default() -> Self {
        Self {
            position: DVec2::ZERO,
            scale: DVec2::ONE,
            rotation: 0.0,
            opacity: 1.0,
            fill: palette::WHITE,
        }
    }
}

impl Props {
    /// The current value of one property.
    pub fn get(&self, property: PropertyId) -> Value {
        match property {
            PropertyId::Position => Value::Vec2(self.position),
            PropertyId::Scale => Value::Vec2(self.scale),
            PropertyId::Rotation => Value::Scalar(self.rotation),
            PropertyId::Opacity => Value::Scalar(self.opacity),
            PropertyId::Fill => Value::Color(self.fill),
        }
    }

    /// Sets one property. A value of the wrong kind for the property is ignored.
    pub fn set(&mut self, property: PropertyId, value: Value) {
        match (property, value) {
            (PropertyId::Position, Value::Vec2(v)) => self.position = v,
            (PropertyId::Scale, Value::Vec2(v)) => self.scale = v,
            (PropertyId::Rotation, Value::Scalar(v)) => self.rotation = v,
            (PropertyId::Opacity, Value::Scalar(v)) => self.opacity = v.clamp(0.0, 1.0),
            (PropertyId::Fill, Value::Color(v)) => self.fill = v,
            _ => {}
        }
    }
}

/// A node as created: its shape and the values its properties start with.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDef {
    pub shape: Shape,
    pub initial: Props,
}

/// Something that can be added to a scene.
pub trait Node {
    /// The definition this node is created from.
    fn def(&self) -> NodeDef;
}

macro_rules! style_setters {
    () => {
        /// Sets the fill colour.
        pub fn fill(mut self, color: LinearRgba) -> Self {
            self.props.fill = color;
            self
        }

        /// Sets the starting position.
        pub fn at(mut self, position: impl Into<DVec2>) -> Self {
            self.props.position = position.into();
            self
        }

        /// Sets the starting opacity.
        pub fn opacity(mut self, opacity: f64) -> Self {
            self.props.opacity = opacity.clamp(0.0, 1.0);
            self
        }
    };
}

/// A circle node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    radius: f64,
    props: Props,
}

impl Circle {
    /// A white circle of `radius` at the origin.
    pub fn new(radius: f64) -> Self {
        Self { radius, props: Props::default() }
    }

    style_setters!();
}

impl Node for Circle {
    fn def(&self) -> NodeDef {
        NodeDef { shape: Shape::Circle { radius: self.radius }, initial: self.props }
    }
}

/// A rectangle node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    size: DVec2,
    props: Props,
}

impl Rect {
    /// A white `width` x `height` rectangle centred at the origin.
    pub fn new(width: f64, height: f64) -> Self {
        Self { size: DVec2::new(width, height), props: Props::default() }
    }

    style_setters!();
}

impl Node for Rect {
    fn def(&self) -> NodeDef {
        NodeDef { shape: Shape::Rect { size: self.size }, initial: self.props }
    }
}

macro_rules! shape_node {
    ($(#[$doc:meta])* $name:ident { $($field:ident: $ty:ty),* } => $shape:expr) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name {
            $($field: $ty,)*
            props: Props,
        }

        impl $name {
            style_setters!();
        }

        impl Node for $name {
            fn def(&self) -> NodeDef {
                let Self { $($field,)* props } = self.clone();
                NodeDef { shape: $shape, initial: props }
            }
        }
    };
}

shape_node!(
    /// A rectangle with rounded corners.
    RoundedRect { size: DVec2, radius: f64 } => Shape::RoundedRect { size, radius }
);

impl RoundedRect {
    /// A white `width` x `height` rectangle with corners of `radius`, centred at the origin.
    pub fn new(width: f64, height: f64, radius: f64) -> Self {
        Self { size: DVec2::new(width, height), radius, props: Props::default() }
    }
}

shape_node!(
    /// A regular polygon with a vertex at the top.
    Polygon { sides: u32, radius: f64 } => Shape::Polygon { sides, radius }
);

impl Polygon {
    /// A white regular polygon with `sides` sides and corners `radius` from its centre.
    pub fn new(sides: u32, radius: f64) -> Self {
        Self { sides, radius, props: Props::default() }
    }
}

shape_node!(
    /// A star with a point at the top.
    Star { points: u32, outer: f64, inner: f64 } => Shape::Star { points, outer, inner }
);

impl Star {
    /// A white star with `points` points reaching `outer` from its centre, and inner
    /// corners at `inner`.
    pub fn new(points: u32, outer: f64, inner: f64) -> Self {
        Self { points, outer, inner, props: Props::default() }
    }
}

shape_node!(
    /// A straight line. It has no area, so it shows only when stroked.
    Line { from: DVec2, to: DVec2 } => Shape::Line { from, to }
);

impl Line {
    /// A line from `from` to `to`.
    pub fn new(from: impl Into<DVec2>, to: impl Into<DVec2>) -> Self {
        Self { from: from.into(), to: to.into(), props: Props::default() }
    }
}

shape_node!(
    /// Any path, built with [`PathData`].
    PathShape { path: PathData } => Shape::Path(path)
);

impl PathShape {
    /// A white shape drawn by `path`.
    pub fn new(path: PathData) -> Self {
        Self { path, props: Props::default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set_round_trip_every_property() {
        let mut props = Props::default();
        let values = [
            (PropertyId::Position, Value::Vec2(DVec2::new(3.0, 4.0))),
            (PropertyId::Scale, Value::Vec2(DVec2::splat(2.0))),
            (PropertyId::Rotation, Value::Scalar(1.5)),
            (PropertyId::Opacity, Value::Scalar(0.25)),
            (PropertyId::Fill, Value::Color(palette::blue())),
        ];
        for (property, value) in values {
            props.set(property, value.clone());
            assert_eq!(props.get(property), value, "{property:?}");
        }
    }

    #[test]
    fn wrong_kind_of_value_is_ignored() {
        let mut props = Props::default();
        props.set(PropertyId::Position, Value::Scalar(9.0));
        assert_eq!(props.position, DVec2::ZERO);
    }

    #[test]
    fn opacity_is_clamped() {
        let mut props = Props::default();
        props.set(PropertyId::Opacity, Value::Scalar(1.4));
        assert_eq!(props.opacity, 1.0);
    }

    #[test]
    fn builders_fill_in_the_definition() {
        let def = Circle::new(80.0).fill(palette::red()).at((10.0, -5.0)).opacity(0.5).def();
        assert_eq!(def.shape, Shape::Circle { radius: 80.0 });
        assert_eq!(def.initial.fill, palette::red());
        assert_eq!(def.initial.position, DVec2::new(10.0, -5.0));
        assert_eq!(def.initial.opacity, 0.5);
        assert_eq!(Rect::new(4.0, 2.0).def().shape, Shape::Rect { size: DVec2::new(4.0, 2.0) });
    }

    #[test]
    fn new_shape_builders_describe_their_shapes() {
        assert_eq!(Polygon::new(6, 3.0).def().shape, Shape::Polygon { sides: 6, radius: 3.0 });
        assert_eq!(
            Star::new(5, 4.0, 2.0).def().shape,
            Shape::Star { points: 5, outer: 4.0, inner: 2.0 }
        );
        let line = Line::new((0.0, 0.0), (1.0, 1.0)).fill(palette::red()).def();
        assert_eq!(line.shape, Shape::Line { from: DVec2::ZERO, to: DVec2::ONE });
        assert_eq!(line.initial.fill, palette::red());
        let rr = RoundedRect::new(4.0, 2.0, 0.5).at((1.0, 1.0)).def();
        assert_eq!(rr.shape, Shape::RoundedRect { size: DVec2::new(4.0, 2.0), radius: 0.5 });
        assert_eq!(rr.initial.position, DVec2::ONE);
        let path =
            PathData::new().move_to((0.0, 0.0)).line_to((1.0, 0.0)).line_to((0.0, 1.0)).close();
        assert_eq!(PathShape::new(path.clone()).def().shape, Shape::Path(path));
    }
}
