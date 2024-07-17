use nalgebra::Vector2;

use crate::{
    bounding::{self, OuterShape, ShapeMut},
    font::Font,
    visual,
};

trait LayoutNode {
    type Boundary: OuterShape;

    fn boundary(&self) -> Self::Boundary;

    fn scale(&mut self, factor: f64);

    fn rotate(&mut self, angle: f64);

    fn translate(&mut self, amount: Vector2<f64>);
}

pub struct LayoutParams<'a> {
    pub font: &'a Font,
    pub phrase_font_size: f32,
    pub symbol_font_size: f32,
}

pub struct Symbol {
    pub name: String,
    pub boundary: bounding::Rect,
}

impl Symbol {
    fn construct(params: &LayoutParams, symbol: visual::Symbol) -> Self {
        let boundary = params.font.measure(&symbol.0, params.symbol_font_size);
        Self {
            name: symbol.0,
            boundary,
        }
    }
}

impl LayoutNode for Symbol {
    type Boundary = bounding::Rect;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
    }
}

pub struct Phrase {
    pub text: String,
    pub boundary: bounding::Rect,
}

impl Phrase {
    fn construct(params: &LayoutParams, phrase: visual::Symbol) -> Self {
        let boundary = params.font.measure(&phrase.0, params.phrase_font_size);
        Self {
            text: phrase.0,
            boundary,
        }
    }
}

impl LayoutNode for Phrase {
    type Boundary = bounding::Rect;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
    }
}

pub struct Pentagram {
    pub boundary: bounding::RegularPolygon,
    pub child: Option<Box<Node>>,
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
        if let Some(child) = &mut self.child {
            child.rotate(angle)
        }
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        if let Some(child) = &mut self.child {
            child.translate(amount)
        }
    }
}

pub struct Circle {
    pub stroke: visual::StrokePattern,
    pub pattern: visual::CirclePattern,
    pub boundary: bounding::Circle,
    pub rim: Vec<Node>,
    pub content: Option<Box<Node>>,
}

impl LayoutNode for Circle {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        let mut boundary: Vec<Box<dyn OuterShape>> = Vec::with_capacity(self.rim.len() + 1);
        boundary.push(Box::new(self.boundary.clone()));
        boundary.extend(self.content.iter().map(|n| n.boundary()));
        boundary
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        self.rim.iter_mut().for_each(|n| n.translate(amount));
        if let Some(content) = &mut self.content {
            content.translate(amount)
        }
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        self.rim.iter_mut().for_each(|n| n.rotate(angle));
        if let Some(content) = &mut self.content {
            content.rotate(angle)
        }
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
    pub child: Option<Box<Node>>,
}

impl LayoutNode for RegularPolygon {
    type Boundary = bounding::RegularPolygon;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        if let Some(child) = &mut self.child {
            child.translate(amount);
        }
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
        if let Some(child) = &mut self.child {
            child.scale(factor);
        }
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        if let Some(child) = &mut self.child {
            child.rotate(angle);
        }
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
