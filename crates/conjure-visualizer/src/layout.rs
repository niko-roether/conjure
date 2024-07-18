use std::f64;

use nalgebra::{vector, Vector2};

use crate::{
    bounding::{self, OuterShape, ShapeMut},
    font::Font,
    visual::{self, StrokePattern},
};

trait LayoutNode {
    type Boundary: OuterShape;

    fn boundary(&self) -> Self::Boundary;

    fn scale(&mut self, factor: f64);

    fn rotate(&mut self, angle: f64);

    fn translate(&mut self, amount: Vector2<f64>);
}

pub struct LayoutParams<'a> {
    pub circle_padding: f64,
    pub circle_thickness: f64,
    pub circle_max_rim_overlap: f64,
    pub circle_min_rim_size: f64,
    pub circle_max_rim_size: f64,
    pub polygon_padding: f64,
    pub phrase_font: &'a Font,
    pub phrase_font_size: f32,
    pub symbol_font: &'a Font,
    pub symbol_font_size: f32,
}

pub struct Symbol {
    pub name: String,
    pub boundary: Vec<bounding::Rect>,
}

impl Symbol {
    fn construct(params: &LayoutParams, symbol: visual::Symbol) -> Self {
        let boundary = params
            .symbol_font
            .measure(&symbol.0, params.symbol_font_size);
        Self {
            name: symbol.0,
            boundary,
        }
    }
}

impl LayoutNode for Symbol {
    type Boundary = Vec<bounding::Rect>;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.iter_mut().for_each(|b| b.scale(factor));
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.iter_mut().for_each(|b| b.rotate(angle));
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.iter_mut().for_each(|b| b.translate(amount));
    }
}

pub struct Phrase {
    pub text: String,
    pub boundary: Vec<bounding::Rect>,
}

impl Phrase {
    fn construct(params: &LayoutParams, phrase: visual::Phrase) -> Self {
        let boundary = params
            .phrase_font
            .measure(&phrase.0, params.phrase_font_size);
        Self {
            text: phrase.0,
            boundary,
        }
    }
}

impl LayoutNode for Phrase {
    type Boundary = Vec<bounding::Rect>;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.iter_mut().for_each(|b| b.scale(factor));
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.iter_mut().for_each(|b| b.rotate(angle));
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.iter_mut().for_each(|b| b.translate(amount))
    }
}

pub struct Pentagram {
    pub boundary: bounding::RegularPolygon,
    pub child: Box<Node>,
}

impl Pentagram {
    const INNER_ROTATION: f64 = 0.25 * f64::consts::TAU;
    const OUTER_ROTATION: f64 = -0.25 * f64::consts::TAU;
    const INNER_OUTER_RADIUS_RATIO: f64 = 2.618033988749895;

    fn construct(params: &LayoutParams, pentagram: visual::Pentagram) -> Self {
        let child = Node::construct(params, *pentagram.content);
        let inner_pentagon = bounding::RegularPolygon::wrap(
            &child.boundary(),
            5,
            Self::INNER_ROTATION,
            params.polygon_padding,
        );

        let boundary = bounding::RegularPolygon::new(
            5,
            Self::INNER_OUTER_RADIUS_RATIO * inner_pentagon.outer_radius(),
            Self::OUTER_ROTATION,
        );

        Self {
            boundary,
            child: Box::new(child),
        }
    }
}

impl LayoutNode for Pentagram {
    type Boundary = bounding::RegularPolygon;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        self.child.rotate(angle);
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        self.child.translate(amount);
    }
}

pub struct Circle {
    pub stroke: visual::StrokePattern,
    pub pattern: visual::CirclePattern,
    pub boundary: bounding::Circle,
    pub rim: Vec<Node>,
    pub content: Box<Node>,
}

struct RimNodeData {
    node: Node,
    boundary: Box<dyn OuterShape>,
    outer_radius: f64,
}

impl Circle {
    fn construct(params: &LayoutParams, circle: visual::Circle) -> Self {
        let circle_thickness = match circle.stroke {
            StrokePattern::DoubleLine => params.circle_thickness,
            _ => 0.0,
        };

        let mut content = Node::construct(params, *circle.content);
        let mut inner_circle = bounding::Circle::wrap(content.boundary(), params.circle_padding);

        let mut rim_node_data: Vec<_> = circle
            .rim
            .into_iter()
            .map(|f| {
                let node = Node::construct(params, f);
                let boundary = node.boundary();
                let outer_radius = boundary.outer_radius();
                RimNodeData {
                    node,
                    boundary,
                    outer_radius,
                }
            })
            .collect();
        let num_rim_items = rim_node_data.len();

        let highest_rim_size = rim_node_data
            .iter()
            .map(|d| d.outer_radius)
            .fold(0.0, f64::max);

        let max_rim_size = params.circle_max_rim_overlap * inner_circle.radius();
        let min_content_size = max_rim_size * highest_rim_size;
        if inner_circle.radius() < min_content_size {
            let factor = min_content_size / inner_circle.radius();
            content.scale(factor);
            inner_circle.scale(factor);
        }

        let min_rim_size = params.circle_min_rim_size * inner_circle.radius();
        for data in &mut rim_node_data {
            if data.outer_radius < min_rim_size {
                let factor = min_rim_size / data.outer_radius;
                data.node.scale(factor);
                data.boundary = Box::new(data.node.boundary());
                data.outer_radius = data.boundary.outer_radius();
            }
        }

        let max_rim_overlap = params.circle_max_rim_overlap * inner_circle.radius();
        let rim_anchor_offset = 0.5 * circle_thickness;
        let rim_anchor_radius = inner_circle.radius() + rim_anchor_offset;
        for (i, data) in rim_node_data.iter_mut().enumerate() {
            let angle = (i as f64) * f64::consts::TAU / (num_rim_items as f64);
            let inward_radius = data
                .boundary
                .outer_radius_at(angle - 0.5 * f64::consts::TAU);
            let initial_overlap = inward_radius - rim_anchor_offset;
            let offset = f64::max(0.0, initial_overlap - max_rim_overlap);

            let translation = (rim_anchor_radius + offset) * vector![angle.cos(), angle.sin()];
            data.node.rotate(angle);
            data.node.translate(translation);
            data.boundary = data.node.boundary();
            data.outer_radius = data.boundary.outer_radius();
        }

        let rim = rim_node_data.into_iter().map(|d| d.node).collect();
        let outer_circle = bounding::Circle::new(
            inner_circle.radius() + circle_thickness,
            inner_circle.center(),
        );
        Self {
            stroke: circle.stroke,
            pattern: circle.pattern,
            boundary: outer_circle,
            content: Box::new(content),
            rim,
        }
    }
}

impl LayoutNode for Circle {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        let mut boundary: Vec<Box<dyn OuterShape>> = Vec::with_capacity(self.rim.len() + 1);
        boundary.push(Box::new(self.boundary.clone()));
        boundary.extend(self.rim.iter().map(|n| n.boundary()));
        boundary
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        self.content.translate(amount);
        self.rim.iter_mut().for_each(|n| n.translate(amount));
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        self.rim.iter_mut().for_each(|n| n.rotate(angle));
        self.content.rotate(angle);
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
        self.rim.iter_mut().for_each(|n| n.scale(factor))
    }
}

pub struct RegularPolygon {
    pub sides: usize,
    pub stroke: visual::StrokePattern,
    pub boundary: bounding::RegularPolygon,
    pub child: Box<Node>,
}

impl LayoutNode for RegularPolygon {
    type Boundary = bounding::RegularPolygon;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        self.child.translate(amount);
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
        self.child.scale(factor);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        self.child.rotate(angle);
    }
}

pub struct DecoratedItem {
    pub kind: visual::DecorationKind,
    pub decoration_box: bounding::Rect,
    pub child: Box<Node>,
}

impl LayoutNode for DecoratedItem {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        vec![Box::new(self.decoration_box.clone()), self.child.boundary()]
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.decoration_box.translate(amount);
        self.child.translate(amount);
    }

    fn rotate(&mut self, angle: f64) {
        self.decoration_box.rotate(angle);
        self.child.rotate(angle);
    }

    fn scale(&mut self, factor: f64) {
        self.decoration_box.scale(factor);
        self.child.scale(factor);
    }
}

pub struct Link {
    pub stroke: visual::StrokePattern,
    pub boundary: bounding::Line,
}

impl LayoutNode for Link {
    type Boundary = bounding::Line;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
    }
}

impl LayoutNode for Vec<Node> {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        self.iter().map(|n| n.boundary()).collect::<Vec<_>>()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.iter_mut().for_each(|n| n.translate(amount));
    }

    fn rotate(&mut self, angle: f64) {
        self.iter_mut().for_each(|n| n.rotate(angle));
    }

    fn scale(&mut self, factor: f64) {
        self.iter_mut().for_each(|n| n.scale(factor));
    }
}

pub enum Node {
    Symbol(Symbol),
    Phrase(Phrase),
    Pentagram(Pentagram),
    Circle(Circle),
    RegularPolygon(RegularPolygon),
    Link(Link),
    Arrangement(Vec<Node>),
}

impl Node {
    fn construct(params: &LayoutParams, figure: visual::Figure) -> Self {
        todo!()
    }
}

impl LayoutNode for Node {
    type Boundary = Box<dyn OuterShape>;

    fn boundary(&self) -> Self::Boundary {
        match self {
            Self::Symbol(s) => Box::new(s.boundary()),
            Self::Phrase(p) => Box::new(p.boundary()),
            Self::Pentagram(p) => Box::new(p.boundary()),
            Self::Circle(c) => Box::new(c.boundary()),
            Self::RegularPolygon(p) => Box::new(p.boundary()),
            Self::Link(l) => Box::new(l.boundary()),
            Self::Arrangement(a) => Box::new(a.boundary()),
        }
    }

    fn scale(&mut self, factor: f64) {
        match self {
            Self::Symbol(s) => s.scale(factor),
            Self::Phrase(p) => p.scale(factor),
            Self::Pentagram(p) => p.scale(factor),
            Self::Circle(c) => c.scale(factor),
            Self::RegularPolygon(p) => p.scale(factor),
            Self::Link(l) => l.scale(factor),
            Self::Arrangement(a) => a.scale(factor),
        }
    }

    fn rotate(&mut self, angle: f64) {
        match self {
            Self::Symbol(s) => s.rotate(angle),
            Self::Phrase(p) => p.rotate(angle),
            Self::Pentagram(p) => p.rotate(angle),
            Self::Circle(c) => c.rotate(angle),
            Self::RegularPolygon(p) => p.rotate(angle),
            Self::Link(l) => l.rotate(angle),
            Self::Arrangement(a) => a.rotate(angle),
        }
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        match self {
            Self::Symbol(s) => s.translate(amount),
            Self::Phrase(p) => p.translate(amount),
            Self::Pentagram(p) => p.translate(amount),
            Self::Circle(c) => c.translate(amount),
            Self::RegularPolygon(p) => p.translate(amount),
            Self::Link(l) => l.translate(amount),
            Self::Arrangement(a) => a.translate(amount),
        }
    }
}
