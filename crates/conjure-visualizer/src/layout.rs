use nalgebra::Vector2;

use crate::{bounding, visual};

pub struct SymbolData {
    pub name: String,
}

pub struct PhraseData {
    pub text: String,
}

pub struct SpecialShapeData {
    pub kind: visual::SpecialShapeKind,
}

pub struct CircleData {
    pub stroke: visual::StrokePattern,
    pub pattern: visual::CirclePattern,
}

pub struct RegularPolygonData {
    pub sides: usize,
    pub stroke: visual::StrokePattern,
}

pub struct DecoratedData {
    pub kind: visual::DecorationKind,
}

pub struct LinkData {
    pub stroke: visual::StrokePattern,
}

pub enum NodeData {
    Circle(CircleData),
    RegularPolygon(RegularPolygonData),
    Link(LinkData),
}

pub struct Node {
    pub data: NodeData,
    pub boundary: Box<dyn bounding::Shape>,
    pub scale: f64,
    pub children: Vec<NodeChild>,
}

pub struct NodeChild {
    pub node: Node,
    pub offset: Vector2<f64>,
    pub rotation: f64,
}

pub fn compute_layout(visual: visual::Figure) -> Node {
    todo!()
}
