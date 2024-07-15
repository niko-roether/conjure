use nalgebra::Vector2;

use crate::{bounding, visual};

pub struct SymbolItem {
    pub name: String,
}

pub struct PhraseItem {
    pub text: String,
}

pub struct SpecialShapeItem {
    pub kind: visual::SpecialShapeKind,
}

pub struct CircleItem {
    pub stroke: visual::StrokePattern,
    pub pattern: visual::CirclePattern,
}

pub struct RegularPolygonItem {
    pub sides: usize,
    pub stroke: visual::StrokePattern,
}

pub struct DecoratedItem {
    pub kind: visual::DecorationKind,
}

pub struct LinkItem {
    pub stroke: visual::StrokePattern,
}

pub enum LayoutItem {
    Circle(CircleItem),
    RegularPolygon(RegularPolygonItem),
    Link(LinkItem),
}

pub struct Node {
    pub item: LayoutItem,
    pub children: Vec<NodeChild>,
}

pub struct NodeChild {
    pub node: Node,
    pub offset: Vector2<f64>,
    pub rotation: f64,
}

impl From<visual::Symbol> for Node {
    fn from(value: visual::Symbol) -> Self {
        todo!()
    }
}
