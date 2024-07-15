use std::{f64, ops::Range};

use anyhow::anyhow;
use nalgebra::{vector, Matrix2, Vector2};

pub trait ShapeOps {
    fn coords_range(&self) -> (Range<f64>, Range<f64>);

    fn outer_radius(&self) -> f64;

    fn radius_at(&self, angle: f64) -> f64;

    fn translate(&mut self, translation: Vector2<f64>);

    fn rotate(&mut self, angle: f64);

    fn scale(&mut self, factor: f64);

    fn wrapping_rect(&self) -> Polygon {
        let (x_range, y_range) = self.coords_range();
        Polygon::new_rect(
            vector![x_range.start, y_range.start],
            vector![x_range.end, y_range.end],
        )
        .unwrap()
    }

    fn containing_circle(&self, padding: f64) -> Circle {
        Circle {
            radius: self.outer_radius() + padding,
            offset: vector![0.0, 0.0],
        }
    }

    fn containing_regular_polygon(&self, sides: usize, angle: f64, padding: f64) -> Polygon {
        let segment_angle = f64::consts::TAU / (sides as f64);
        let unpadded_inner_radius = (0..sides)
            .map(|i| self.radius_at(angle + segment_angle / 2.0 + (i as f64) * segment_angle))
            .reduce(f64::max)
            .unwrap_or_default();
        let inner_radius = dbg!(unpadded_inner_radius) + padding;
        let outer_radius = inner_radius / f64::cos(segment_angle / 2.0);

        Polygon::new_regular(sides, outer_radius, angle)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    radius: f64,
    offset: Vector2<f64>,
}

impl Circle {
    pub fn new(radius: f64) -> Self {
        Self {
            radius,
            offset: vector![0.0, 0.0],
        }
    }

    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    #[inline]
    pub fn offset(&self) -> Vector2<f64> {
        self.offset
    }

    pub fn wrap(shape: &impl ShapeOps, padding: f64) -> Self {
        Self::new(shape.outer_radius() + padding)
    }
}

impl ShapeOps for Circle {
    fn coords_range(&self) -> (Range<f64>, Range<f64>) {
        (
            self.offset.x - self.radius..self.offset.x + self.radius,
            self.offset.y - self.radius..self.offset.y + self.radius,
        )
    }

    fn outer_radius(&self) -> f64 {
        self.radius + self.offset.magnitude()
    }

    fn radius_at(&self, angle: f64) -> f64 {
        self.offset.x * angle.cos() + self.offset.y * angle.sin() + self.radius
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        self.offset += translation;
    }

    fn rotate(&mut self, angle: f64) {
        if self.offset == vector![0.0, 0.0] {
            return;
        }
        let angle_cos = angle.cos();
        let angle_sin = angle.sin();
        let matrix = Matrix2::new(angle_cos, angle_sin, -angle_sin, angle_cos);
        self.offset = matrix * self.offset;
    }

    fn scale(&mut self, factor: f64) {
        self.radius *= factor;
        self.offset *= factor;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    // The polygon must have at least one vertex
    vertices: Vec<Vector2<f64>>,
}

impl Polygon {
    pub fn new_regular(sides: usize, outer_radius: f64, angle: f64) -> Self {
        let segment_angle = f64::consts::TAU / (sides as f64);
        let mut vertices = vec![Vector2::zeros(); sides];
        for (i, vertex) in vertices.iter_mut().enumerate() {
            let vertex_angle = angle + (i as f64) * segment_angle;
            *vertex = vector![
                outer_radius * vertex_angle.cos(),
                outer_radius * vertex_angle.sin()
            ];
        }
        Self { vertices }
    }

    pub fn wrap_regular(sides: usize, angle: f64, shape: &impl ShapeOps, padding: f64) -> Self {
        let segment_angle = f64::consts::TAU / (sides as f64);
        let unpadded_inner_radius = (0..sides)
            .map(|i| shape.radius_at(angle + segment_angle / 2.0 + (i as f64) * segment_angle))
            .reduce(f64::max)
            .unwrap_or_default();
        let inner_radius = dbg!(unpadded_inner_radius) + padding;
        let outer_radius = inner_radius / f64::cos(segment_angle / 2.0);

        Polygon::new_regular(sides, outer_radius, angle)
    }

    pub fn new_rect(corner_1: Vector2<f64>, corner_2: Vector2<f64>) -> anyhow::Result<Self> {
        let diagonal = corner_2 - corner_1;
        let corner_3 = corner_1 + vector![diagonal.x, 0.0];
        let corner_4 = corner_1 + vector![0.0, diagonal.y];
        Ok(Self {
            vertices: vec![corner_1, corner_3, corner_2, corner_4],
        })
    }

    #[inline]
    pub fn vertices(&self) -> &[Vector2<f64>] {
        &self.vertices
    }

    #[inline]
    pub fn num_sides(&self) -> usize {
        self.vertices.len()
    }
}

impl ShapeOps for Polygon {
    fn coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut vertex_iter = self.vertices.iter();
        let first_vertex = vertex_iter
            .next()
            .expect("The polygon should have at least one vertex");

        let mut x_range = first_vertex.x..first_vertex.x;
        let mut y_range = first_vertex.y..first_vertex.y;
        for vertex in vertex_iter {
            x_range.start = f64::min(x_range.start, vertex.x);
            x_range.end = f64::max(x_range.end, vertex.x);
            y_range.start = f64::min(y_range.start, vertex.y);
            y_range.end = f64::max(y_range.end, vertex.y);
        }
        (x_range, y_range)
    }

    fn outer_radius(&self) -> f64 {
        self.vertices
            .iter()
            .map(Vector2::magnitude_squared)
            .reduce(f64::max)
            .expect("The polygon should have at least one vertex")
            .sqrt()
    }

    fn radius_at(&self, angle: f64) -> f64 {
        let normal = vector![angle.cos(), angle.sin()];
        self.vertices
            .iter()
            .map(|v| v.dot(&normal))
            .reduce(f64::max)
            .expect("The polygon should have at least one vertex")
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        self.vertices.iter_mut().for_each(|v| *v += translation)
    }

    fn rotate(&mut self, angle: f64) {
        let angle_cos = angle.cos();
        let angle_sin = angle.sin();
        let matrix = Matrix2::new(angle_cos, angle_sin, -angle_sin, angle_cos);

        self.vertices.iter_mut().for_each(|v| *v = matrix * *v)
    }

    fn scale(&mut self, factor: f64) {
        self.vertices.iter_mut().for_each(|v| *v *= factor);
    }
}

pub struct CompositeShape {
    // This vec must not be empty
    parts: Vec<Shape>,
}

impl CompositeShape {
    pub fn new(parts: Vec<Shape>) -> anyhow::Result<Self> {
        if parts.is_empty() {
            return Err(anyhow!("Composite shapes must have at least one part!"));
        }
        Ok(Self { parts })
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.parts.push(shape)
    }
}

impl ShapeOps for CompositeShape {
    fn coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut parts_iter = self.parts.iter();
        let first_part = parts_iter
            .next()
            .expect("Composite shape should have at least one part");

        let (mut x_range, mut y_range) = first_part.coords_range();
        for part in parts_iter {
            let (part_x_range, part_y_range) = part.coords_range();
            x_range.start = f64::min(x_range.start, part_x_range.start);
            x_range.end = f64::max(x_range.end, part_x_range.end);
            y_range.start = f64::min(y_range.start, part_y_range.start);
            y_range.end = f64::max(y_range.end, part_y_range.end);
        }
        (x_range, y_range)
    }

    fn outer_radius(&self) -> f64 {
        self.parts
            .iter()
            .map(|p| p.outer_radius())
            .reduce(f64::max)
            .expect("Composite shape should have at least one part")
    }

    fn radius_at(&self, angle: f64) -> f64 {
        self.parts
            .iter()
            .map(|p| p.radius_at(angle))
            .reduce(f64::max)
            .expect("Composite shape should have at least one part")
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        self.parts.iter_mut().for_each(|p| p.translate(translation));
    }

    fn rotate(&mut self, angle: f64) {
        self.parts.iter_mut().for_each(|p| p.rotate(angle))
    }

    fn scale(&mut self, factor: f64) {
        self.parts.iter_mut().for_each(|p| p.scale(factor))
    }
}

pub enum Shape {
    Circle(Circle),
    Polygon(Polygon),
    Composite(CompositeShape),
}

impl ShapeOps for Shape {
    fn coords_range(&self) -> (Range<f64>, Range<f64>) {
        match self {
            Self::Circle(c) => c.coords_range(),
            Self::Polygon(p) => p.coords_range(),
            Self::Composite(c) => c.coords_range(),
        }
    }

    fn outer_radius(&self) -> f64 {
        match self {
            Self::Circle(c) => c.outer_radius(),
            Self::Polygon(p) => p.outer_radius(),
            Self::Composite(c) => c.outer_radius(),
        }
    }

    fn radius_at(&self, angle: f64) -> f64 {
        match self {
            Self::Circle(c) => c.radius_at(angle),
            Self::Polygon(p) => p.radius_at(angle),
            Self::Composite(c) => c.radius_at(angle),
        }
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        match self {
            Self::Circle(c) => c.translate(translation),
            Self::Polygon(p) => p.translate(translation),
            Self::Composite(c) => c.translate(translation),
        }
    }

    fn rotate(&mut self, angle: f64) {
        match self {
            Self::Circle(c) => c.rotate(angle),
            Self::Polygon(p) => p.rotate(angle),
            Self::Composite(c) => c.rotate(angle),
        }
    }

    fn scale(&mut self, factor: f64) {
        match self {
            Self::Circle(c) => c.scale(factor),
            Self::Polygon(p) => p.scale(factor),
            Self::Composite(c) => c.scale(factor),
        }
    }
}
