use std::f64;

use nalgebra::{vector, Vector2};

use crate::{
    bounding::{self, OuterShape, ShapeMut},
    font::Font,
    visual::{self},
};

trait LayoutNode {
    type Boundary: OuterShape;

    fn boundary(&self) -> Self::Boundary;

    fn scale(&mut self, factor: f64);

    fn rotate(&mut self, angle: f64);

    fn translate(&mut self, amount: Vector2<f64>);
}

#[derive(Debug)]
pub struct LayoutParams<'a> {
    pub emphasis_rays_radius_ratio: f64,
    pub decoration_hat_relative_width: f64,
    pub decoration_hat_relative_height: f64,
    pub decoration_tilde_relative_width: f64,
    pub decoration_tilde_relative_height: f64,
    pub decoration_position_radius_ratio: f64,
    pub circle_content_scale: f64,
    pub double_stroke_radius_ratio: f64,
    pub circle_max_rim_overlap_ratio: f64,
    pub circle_min_rim_ratio: f64,
    pub circle_max_rim_ratio: f64,
    pub polygon_content_scale: f64,
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
        let mut child = Node::construct(params, *pentagram.content);
        let inner_pentagon =
            bounding::RegularPolygon::wrap(&child.boundary(), 5, Self::INNER_ROTATION);

        let boundary = bounding::RegularPolygon::new(
            5,
            Self::INNER_OUTER_RADIUS_RATIO * inner_pentagon.outer_radius(),
            Self::OUTER_ROTATION,
        );

        child.scale(params.polygon_content_scale);

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
    pub double: bool,
    pub boundary: bounding::Circle,
    pub rim: Vec<Node>,
    pub content: Box<Node>,
}

impl Circle {
    const BASE_RIM_ROTATION: f64 = f64::consts::TAU * -0.25;

    fn construct(params: &LayoutParams, circle: visual::Circle) -> Self {
        let radius_ratio = Self::get_outer_radius_ratio(params, &circle);
        let mut content = Node::construct(params, *circle.content);
        let mut rim = Self::get_rim_nodes(params, circle.rim);

        Self::apply_content_constraints(params, &mut content, &rim);

        let inner_circle = bounding::Circle::wrap(content.boundary());
        let outer_radius = inner_circle.radius() * radius_ratio;
        let mean_radius = (inner_circle.radius() + outer_radius) * 0.5;

        Self::apply_rim_constraints(params, &mut rim, mean_radius);
        Self::position_rim_items(params, &mut rim, mean_radius, inner_circle.radius());

        content.scale(params.circle_content_scale);

        Self {
            stroke: circle.stroke,
            pattern: circle.pattern,
            double: circle.double,
            boundary: bounding::Circle::from_radius(outer_radius),
            rim,
            content: Box::new(content),
        }
    }

    fn apply_content_constraints(params: &LayoutParams, content: &mut Node, rim: &[Node]) {
        let highest_rim_size = rim
            .iter()
            .map(|n| n.boundary().outer_radius())
            .fold(0.0, f64::max);

        let min_content_size = highest_rim_size / params.circle_max_rim_ratio;
        let content_radius = content.boundary().outer_radius();
        if content_radius < min_content_size {
            let factor = min_content_size / content_radius;
            content.scale(factor);
        }
    }

    fn apply_rim_constraints(params: &LayoutParams, rim: &mut [Node], mean_radius: f64) {
        let min_rim_size = params.circle_min_rim_ratio * mean_radius;
        for rim_node in rim {
            let radius = rim_node.boundary().outer_radius();
            if radius < min_rim_size {
                let factor = min_rim_size / radius;
                rim_node.scale(factor);
            }
        }
    }

    fn position_rim_items(
        params: &LayoutParams,
        rim: &mut [Node],
        mean_radius: f64,
        inner_radius: f64,
    ) {
        let num_rim_items = rim.len();
        let max_rim_overlap = params.circle_max_rim_overlap_ratio * inner_radius;
        for (i, rim_node) in rim.iter_mut().enumerate() {
            let orientation = (i as f64) * f64::consts::TAU / (num_rim_items as f64);
            let angle = orientation + Self::BASE_RIM_ROTATION;
            let inward_radius = rim_node
                .boundary()
                .outer_radius_at(angle - 0.5 * f64::consts::TAU);
            let initial_overlap = mean_radius - inner_radius - inward_radius;
            let offset = f64::max(0.0, initial_overlap - max_rim_overlap);
            let translation = (mean_radius + offset) * vector![angle.cos(), angle.sin()];
            rim_node.rotate(orientation);
            rim_node.translate(translation);
        }
    }

    fn get_rim_nodes(params: &LayoutParams, rim: Vec<visual::Figure>) -> Vec<Node> {
        rim.into_iter()
            .map(|f| Node::construct(params, f))
            .collect()
    }

    fn get_outer_radius_ratio(params: &LayoutParams, circle: &visual::Circle) -> f64 {
        if circle.double {
            params.double_stroke_radius_ratio
        } else {
            1.0
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

impl RegularPolygon {
    const BASE_ROTATION: f64 = f64::consts::TAU * -0.25;

    fn construct(params: &LayoutParams, polygon: visual::RegularPolygon) -> Self {
        let mut child = Node::construct(params, *polygon.content);
        let boundary =
            bounding::RegularPolygon::wrap(&child.boundary(), polygon.sides, Self::BASE_ROTATION);
        child.scale(params.polygon_content_scale);
        Self {
            sides: polygon.sides,
            stroke: polygon.stroke,
            boundary,
            child: Box::new(child),
        }
    }
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

pub struct Decorated {
    pub kind: visual::DecorationKind,
    pub decoration_rect: bounding::Rect,
    pub child: Box<Node>,
}

struct DecorationParams {
    rect: bounding::Rect,
    angle: f64,
}

impl Decorated {
    fn construct(params: &LayoutParams, decorated: visual::Decorated) -> Self {
        let child = Node::construct(params, *decorated.content);
        let radius = child.boundary().outer_radius();
        let decoration_rect = Self::position_decoration(params, radius, decorated.kind);

        Self {
            kind: decorated.kind,
            decoration_rect,
            child: Box::new(child),
        }
    }

    fn position_decoration(
        params: &LayoutParams,
        radius: f64,
        kind: visual::DecorationKind,
    ) -> bounding::Rect {
        let DecorationParams { mut rect, angle } = Self::get_deco_params(params, radius, kind);
        let position_radius = params.decoration_position_radius_ratio * radius;

        rect.translate(position_radius * vector![angle.cos(), angle.sin()]);
        rect
    }

    fn get_deco_params(
        params: &LayoutParams,
        radius: f64,
        kind: visual::DecorationKind,
    ) -> DecorationParams {
        match kind {
            visual::DecorationKind::Hat => DecorationParams {
                angle: f64::consts::TAU * -0.25,
                rect: bounding::Rect::from_width_height(
                    params.decoration_hat_relative_width * radius,
                    params.decoration_hat_relative_height * radius,
                ),
            },
            visual::DecorationKind::Tilde => DecorationParams {
                angle: f64::consts::TAU * 0.25,
                rect: bounding::Rect::from_width_height(
                    params.decoration_tilde_relative_width * radius,
                    params.decoration_tilde_relative_height * radius,
                ),
            },
        }
    }
}

impl LayoutNode for Decorated {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        vec![
            Box::new(self.decoration_rect.clone()),
            self.child.boundary(),
        ]
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.decoration_rect.translate(amount);
        self.child.translate(amount);
    }

    fn rotate(&mut self, angle: f64) {
        self.decoration_rect.rotate(angle);
        self.child.rotate(angle);
    }

    fn scale(&mut self, factor: f64) {
        self.decoration_rect.scale(factor);
        self.child.scale(factor);
    }
}

pub struct Emphasized {
    pub kind: visual::EmphasisKind,
    pub boundary: bounding::Circle,
    pub child: Box<Node>,
}

impl Emphasized {
    fn construct(params: &LayoutParams, emphasized: visual::Emphasized) -> Self {
        let child = Node::construct(params, *emphasized.content);
        let mut boundary = bounding::Circle::wrap(child.boundary());
        boundary.scale(Self::get_radius_ratio(params, emphasized.kind));

        Self {
            kind: emphasized.kind,
            boundary,
            child: Box::new(child),
        }
    }

    fn get_radius_ratio(params: &LayoutParams, kind: visual::EmphasisKind) -> f64 {
        match kind {
            visual::EmphasisKind::Rays => params.emphasis_rays_radius_ratio,
        }
    }
}

impl LayoutNode for Emphasized {
    type Boundary = bounding::Circle;

    fn boundary(&self) -> Self::Boundary {
        self.boundary.clone()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.boundary.translate(amount);
        self.child.translate(amount);
    }

    fn rotate(&mut self, angle: f64) {
        self.boundary.rotate(angle);
        self.child.rotate(angle);
    }

    fn scale(&mut self, factor: f64) {
        self.boundary.scale(factor);
        self.child.scale(factor)
    }
}

pub struct Link {
    pub stroke: visual::StrokePattern,
    pub segments: Vec<bounding::Line>,
    pub items: Vec<Node>,
}

impl Link {
    const BASE_ANGLE: f64 = f64::consts::TAU * -0.25;

    fn construct(params: &LayoutParams, link: visual::Link) -> Self {
        let mut items: Vec<Node> = Vec::with_capacity(link.items.len());
        let mut segments: Vec<bounding::Line> = Vec::with_capacity(link.items.len() - 1);

        let num_sides: f64 = link.items.len() as f64;
        let start_angle = Self::BASE_ANGLE + 0.5 * (num_sides - 1.0) / num_sides * f64::consts::TAU;

        let mut prev_position: Option<Vector2<f64>>;
        for i in 0..link.items.len() {
            let angle = start_angle + i as f64 / num_sides * f64::consts::TAU;
        }

        todo!()
    }
}

impl LayoutNode for Link {
    type Boundary = Vec<Box<dyn OuterShape>>;

    fn boundary(&self) -> Self::Boundary {
        self.segments
            .iter()
            .map(|s| Box::new(s.clone()) as Box<dyn OuterShape>)
            .chain(self.items.iter().map(|i| i.boundary()))
            .collect()
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.segments.iter_mut().for_each(|s| s.translate(amount));
        self.items.iter_mut().for_each(|i| i.translate(amount));
    }

    fn rotate(&mut self, angle: f64) {
        self.segments.iter_mut().for_each(|s| s.rotate(angle));
        self.items.iter_mut().for_each(|i| i.rotate(angle));
    }

    fn scale(&mut self, factor: f64) {
        self.segments.iter_mut().for_each(|s| s.scale(factor));
        self.items.iter_mut().for_each(|i| i.scale(factor));
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
