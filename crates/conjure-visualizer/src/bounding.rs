use std::{f64, ops::Range};

use nalgebra::{vector, Rotation2, Vector2};

pub trait ConvexHull {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>);

    fn convex_radius(&self) -> f64;

    fn convex_radius_at(&self, angle: f64) -> f64;
}

pub trait TransformShape {
    fn translate(&mut self, amount: Vector2<f64>);

    fn rotate(&mut self, angle: f64);

    fn scale(&mut self, factor: f64);
}

fn vector_to_line(from: Vector2<f64>, to: Vector2<f64>) -> Vector2<f64> {
    let diff = to - from;
    let factor = (from.magnitude_squared() - from.dot(&to)) / diff.magnitude_squared();
    from + factor * diff
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub start: Vector2<f64>,
    pub end: Vector2<f64>,
}

impl ConvexHull for Line {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>) {
        (
            f64::min(self.start.x, self.end.x)..f64::max(self.start.x, self.end.x),
            f64::min(self.start.y, self.end.y)..f64::min(self.start.y, self.end.y),
        )
    }

    fn convex_radius(&self) -> f64 {
        f64::max(self.start.magnitude_squared(), self.end.magnitude_squared()).sqrt()
    }

    fn convex_radius_at(&self, _angle: f64) -> f64 {
        f64::max(
            0.0,
            vector_to_line(self.start, self.end).magnitude_squared(),
        )
        .sqrt()
    }
}

impl TransformShape for Line {
    fn scale(&mut self, factor: f64) {
        self.start *= factor;
        self.end *= factor;
    }

    fn rotate(&mut self, angle: f64) {
        let rotation = Rotation2::new(angle);
        self.start = rotation * self.start;
        self.end = rotation * self.end;
    }

    fn translate(&mut self, amount: Vector2<f64>) {
        self.start += amount;
        self.end += amount;
    }
}

pub struct PolygonSides<V> {
    first: Option<Vector2<f64>>,
    prev: Option<Vector2<f64>>,
    vertices: V,
}

impl<V> PolygonSides<V> {
    fn new(vertices: V) -> Self {
        Self {
            first: None,
            prev: None,
            vertices,
        }
    }
}

impl<V> Iterator for PolygonSides<V>
where
    V: Iterator<Item = Vector2<f64>>,
{
    type Item = Vector2<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(vertex) = self.vertices.next() else {
                return Some(vector_to_line(self.prev.take()?, self.first.take()?));
            };
            self.first.get_or_insert(vertex);
            if let Some(prev) = self.prev.replace(vertex) {
                return Some(vector_to_line(prev, vertex));
            }
        }
    }
}

pub trait Polygon {
    type Vertices: Iterator<Item = Vector2<f64>>;

    fn vertices(&self) -> Self::Vertices;

    fn sides(&self) -> PolygonSides<Self::Vertices> {
        PolygonSides::new(self.vertices())
    }
}

impl<P: Polygon> ConvexHull for P {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut x_range = 0.0..0.0;
        let mut y_range = 0.0..0.0;
        for vertex in self.vertices() {
            x_range = f64::min(x_range.start, vertex.x)..f64::max(x_range.end, vertex.x);
            y_range = f64::min(y_range.start, vertex.y)..f64::max(y_range.end, vertex.y);
        }
        (x_range, y_range)
    }

    fn convex_radius(&self) -> f64 {
        self.vertices()
            .map(|v| v.magnitude_squared())
            .fold(0.0, f64::max)
            .sqrt()
    }

    fn convex_radius_at(&self, angle: f64) -> f64 {
        let normal = vector![angle.cos(), angle.sin()];
        self.vertices().map(|v| v.dot(&normal)).fold(0.0, f64::max)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    radius: f64,
    offset: Vector2<f64>,
}

impl Circle {
    pub fn new(radius: f64, center: Vector2<f64>) -> Self {
        Self {
            radius,
            offset: -center,
        }
    }

    pub fn from_radius(radius: f64) -> Self {
        Self::new(radius, Vector2::zeros())
    }

    pub fn wrap(shape: impl ConvexHull) -> Self {
        Self::from_radius(shape.convex_radius())
    }

    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    #[inline]
    pub fn center(&self) -> Vector2<f64> {
        -self.offset
    }
}

impl ConvexHull for Circle {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>) {
        (
            self.offset.x - self.radius..self.offset.x + self.radius,
            self.offset.y - self.radius..self.offset.y + self.radius,
        )
    }

    fn convex_radius(&self) -> f64 {
        self.radius + self.offset.magnitude()
    }

    fn convex_radius_at(&self, angle: f64) -> f64 {
        self.offset.x * angle.cos() + self.offset.y * angle.sin() + self.radius
    }
}

impl TransformShape for Circle {
    fn translate(&mut self, amount: Vector2<f64>) {
        self.offset += amount;
    }

    fn rotate(&mut self, angle: f64) {
        self.offset = Rotation2::new(-angle) * self.offset;
    }

    fn scale(&mut self, factor: f64) {
        self.radius *= factor;
        self.offset *= factor;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    width: f64,
    height: f64,
    offset: Vector2<f64>,
    rotation: f64,
}

impl Rect {
    pub fn new(width: f64, height: f64, rotation: f64, center: Vector2<f64>) -> Self {
        Self {
            width,
            height,
            offset: -center,
            rotation,
        }
    }

    pub fn from_width_height_rotation(width: f64, height: f64, rotation: f64) -> Self {
        Self::new(width, height, rotation, Vector2::zeros())
    }

    pub fn from_width_height(width: f64, height: f64) -> Self {
        Self::from_width_height_rotation(width, height, 0.0)
    }

    pub fn wrap(shape: &impl ConvexHull) -> Self {
        let (x_range, y_range) = shape.convex_coords_range();
        let width = 2.0 * f64::max(x_range.start.abs(), x_range.end.abs());
        let height = 2.0 * f64::max(y_range.start.abs(), y_range.end.abs());
        Self::from_width_height(width, height)
    }

    pub fn wrap_rotated(shape: &impl ConvexHull, rotation: f64) -> Self {
        let width = f64::max(
            shape.convex_radius_at(rotation),
            shape.convex_radius_at(rotation + f64::consts::TAU * 0.5),
        );
        let height = f64::max(
            shape.convex_radius_at(rotation + f64::consts::TAU * 0.25),
            shape.convex_radius_at(rotation + f64::consts::TAU * 0.75),
        );
        Self::from_width_height_rotation(width, height, rotation)
    }

    #[inline]
    pub fn width(&self) -> f64 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> f64 {
        self.height
    }

    #[inline]
    pub fn center(&self) -> Vector2<f64> {
        -self.offset
    }

    #[inline]
    pub fn rotation(&self) -> f64 {
        self.rotation
    }
}

pub struct RectVertices {
    half_width: f64,
    half_height: f64,
    rotation: Rotation2<f64>,
    offset: Vector2<f64>,
    idx: u8,
}

impl RectVertices {
    fn new(rect: &Rect) -> Self {
        Self {
            half_width: rect.width / 2.0,
            half_height: rect.height / 2.0,
            rotation: Rotation2::new(rect.rotation),
            offset: rect.offset,
            idx: 0,
        }
    }
}

impl Iterator for RectVertices {
    type Item = Vector2<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let (s_x, s_y) = match self.idx {
            0 => (-1.0, -1.0),
            1 => (1.0, -1.0),
            2 => (1.0, 1.0),
            3 => (-1.0, 1.0),
            _ => return None,
        };
        self.idx += 1;
        let vertex =
            self.rotation * vector![s_x * self.half_width, s_y * self.half_height] + self.offset;
        Some(vertex)
    }
}

impl Polygon for Rect {
    type Vertices = RectVertices;

    fn vertices(&self) -> Self::Vertices {
        RectVertices::new(self)
    }
}

impl TransformShape for Rect {
    fn translate(&mut self, amount: Vector2<f64>) {
        self.offset += amount;
    }

    fn rotate(&mut self, angle: f64) {
        self.rotation += angle;
        self.offset = Rotation2::new(-angle) * self.offset;
    }

    fn scale(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
        self.offset *= factor;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegularPolygon {
    num_sides: usize,
    outer_radius: f64,
    rotation: f64,
    offset: Vector2<f64>,
}

impl RegularPolygon {
    pub fn new(num_sides: usize, outer_radius: f64, rotation: f64) -> Self {
        Self {
            num_sides,
            outer_radius,
            rotation,
            offset: vector![0.0, 0.0],
        }
    }

    pub fn wrap(shape: &impl ConvexHull, num_sides: usize, rotation: f64) -> Self {
        let segment_angle = f64::consts::TAU / (num_sides as f64);
        let unpadded_inner_radius = (0..num_sides)
            .map(|i| {
                shape.convex_radius_at(rotation + segment_angle / 2.0 + (i as f64) * segment_angle)
            })
            .reduce(f64::max)
            .unwrap_or_default();
        let inner_radius = unpadded_inner_radius;
        let outer_radius = inner_radius / f64::cos(segment_angle / 2.0);
        Self::new(num_sides, outer_radius, rotation)
    }

    #[inline]
    pub fn num_sides(&self) -> usize {
        self.num_sides
    }

    #[inline]
    pub fn center(&self) -> Vector2<f64> {
        -self.offset
    }

    #[inline]
    pub fn rotation(&self) -> f64 {
        self.rotation
    }
}

pub struct RegularPolygonVertices {
    outer_radius: f64,
    base_angle: f64,
    idx_iter: Range<usize>,
}

impl RegularPolygonVertices {
    fn new(polygon: &RegularPolygon) -> Self {
        Self {
            outer_radius: polygon.outer_radius,
            base_angle: f64::consts::TAU / (polygon.num_sides as f64),
            idx_iter: 0..polygon.num_sides,
        }
    }
}

impl Iterator for RegularPolygonVertices {
    type Item = Vector2<f64>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx_iter.next()?;
        let angle = (idx as f64) * self.base_angle;
        let vertex = self.outer_radius * vector![angle.cos(), angle.sin()];
        Some(vertex)
    }
}

impl Polygon for RegularPolygon {
    type Vertices = RegularPolygonVertices;

    fn vertices(&self) -> Self::Vertices {
        RegularPolygonVertices::new(self)
    }
}

impl TransformShape for RegularPolygon {
    fn translate(&mut self, amount: Vector2<f64>) {
        self.offset += amount;
    }

    fn rotate(&mut self, angle: f64) {
        self.rotation += angle;
        self.offset = Rotation2::new(-angle) * self.offset;
    }

    fn scale(&mut self, factor: f64) {
        self.outer_radius *= factor;
        self.offset *= factor;
    }
}

impl ConvexHull for Box<dyn ConvexHull> {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>) {
        (**self).convex_coords_range()
    }

    fn convex_radius(&self) -> f64 {
        (**self).convex_radius()
    }

    fn convex_radius_at(&self, angle: f64) -> f64 {
        (**self).convex_radius_at(angle)
    }
}

impl<S: ConvexHull> ConvexHull for Vec<S> {
    fn convex_coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut x_range = 0.0..0.0;
        let mut y_range = 0.0..0.0;
        for shape in self {
            let (shape_x_range, shape_y_range) = shape.convex_coords_range();
            x_range = f64::min(x_range.start, shape_x_range.start)
                ..f64::max(x_range.end, shape_x_range.end);
            y_range = f64::min(y_range.start, shape_y_range.start)
                ..f64::max(y_range.end, shape_y_range.end);
        }
        (x_range, y_range)
    }

    fn convex_radius(&self) -> f64 {
        self.iter().map(|s| s.convex_radius()).fold(0.0, f64::max)
    }

    fn convex_radius_at(&self, angle: f64) -> f64 {
        self.iter()
            .map(|s| s.convex_radius_at(angle))
            .fold(0.0, f64::max)
    }
}
