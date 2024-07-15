use std::{f64, ops::Range};

use nalgebra::{vector, Isometry2, Rotation2, Vector2};

pub trait OuterShape {
    fn outer_coords_range(&self) -> (Range<f64>, Range<f64>);

    fn outer_radius(&self) -> f64;

    fn outer_radius_at(&self, angle: f64) -> f64;
}

pub trait InnerShape {
    fn inner_coords_range(&self) -> (Range<f64>, Range<f64>);

    fn inner_radius(&self) -> f64;

    fn inner_radius_at(&self, angle: f64) -> f64;
}

pub trait ShapeMut {
    fn set_center(&mut self, center: Vector2<f64>);

    fn rotate(&mut self, angle: f64);

    fn scale(&mut self, factor: f64);
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

    fn get_vector_for(from: Vector2<f64>, to: Vector2<f64>) -> Vector2<f64> {
        let diff = to - from;
        from + (from.magnitude_squared() - from.dot(&to)) / diff.magnitude_squared() * diff
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
                return Some(Self::get_vector_for(self.first.take()?, self.prev.take()?));
            };
            self.first.get_or_insert(vertex);
            if let Some(prev) = self.prev {
                return Some(Self::get_vector_for(prev, vertex));
            }
            self.prev.replace(vertex);
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

impl<P: Polygon> OuterShape for P {
    fn outer_coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut x_range = 0.0..0.0;
        let mut y_range = 0.0..0.0;
        for vertex in self.vertices() {
            x_range = f64::min(x_range.start, vertex.x)..f64::max(x_range.end, vertex.x);
            y_range = f64::min(y_range.start, vertex.y)..f64::max(y_range.end, vertex.y);
        }
        (x_range, y_range)
    }

    fn outer_radius(&self) -> f64 {
        self.vertices()
            .map(|v| v.magnitude_squared())
            .fold(0.0, f64::max)
            .sqrt()
    }

    fn outer_radius_at(&self, angle: f64) -> f64 {
        let normal = vector![angle.cos(), angle.sin()];
        self.vertices().map(|v| v.dot(&normal)).fold(0.0, f64::max)
    }
}

impl<P: Polygon> InnerShape for P {
    fn inner_coords_range(&self) -> (Range<f64>, Range<f64>) {
        let mut x_range = f64::NEG_INFINITY..f64::INFINITY;
        let mut y_range = f64::NEG_INFINITY..f64::INFINITY;
        for to_side in self.sides() {
            x_range = f64::max(x_range.start, to_side.x)..f64::min(x_range.end, to_side.x);
            y_range = f64::max(y_range.start, to_side.y)..f64::min(y_range.end, to_side.y);
        }
        (x_range, y_range)
    }

    fn inner_radius(&self) -> f64 {
        self.sides()
            .map(|s| s.magnitude_squared())
            .fold(f64::INFINITY, f64::min)
            .sqrt()
    }

    fn inner_radius_at(&self, angle: f64) -> f64 {
        let normal = vector![angle.cos(), angle.sin()];
        self.sides()
            .map(|s| s.magnitude_squared() / s.dot(&normal))
            .filter(|d| *d >= 0.0)
            .fold(f64::INFINITY, f64::min)
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

    pub fn wrap(shape: impl OuterShape, padding: f64) -> Self {
        Self::from_radius(shape.outer_radius() + padding)
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

impl OuterShape for Circle {
    fn outer_coords_range(&self) -> (Range<f64>, Range<f64>) {
        (
            self.offset.x - self.radius..self.offset.x + self.radius,
            self.offset.y - self.radius..self.offset.y + self.radius,
        )
    }

    fn outer_radius(&self) -> f64 {
        self.radius + self.offset.magnitude()
    }

    fn outer_radius_at(&self, angle: f64) -> f64 {
        self.offset.x * angle.cos() + self.offset.y * angle.sin() + self.radius
    }
}

impl InnerShape for Circle {
    fn inner_coords_range(&self) -> (Range<f64>, Range<f64>) {
        (-self.radius..self.radius, -self.radius..self.radius)
    }

    fn inner_radius(&self) -> f64 {
        self.radius
    }

    fn inner_radius_at(&self, _angle: f64) -> f64 {
        self.radius
    }
}

impl ShapeMut for Circle {
    fn set_center(&mut self, center: Vector2<f64>) {
        self.offset = -center;
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

    pub fn wrap(shape: &impl OuterShape, padding: f64) -> Self {
        let (x_range, y_range) = shape.outer_coords_range();
        let width = 2.0 * f64::max(x_range.start.abs(), x_range.end.abs());
        let height = 2.0 * f64::max(y_range.start.abs(), y_range.end.abs());
        Self::from_width_height(width + 2.0 * padding, height + 2.0 * padding)
    }

    pub fn wrap_rotated(shape: &impl OuterShape, rotation: f64, padding: f64) -> Self {
        let width = f64::max(
            shape.outer_radius_at(rotation),
            shape.outer_radius_at(rotation + f64::consts::TAU * 0.5),
        );
        let height = f64::max(
            shape.outer_radius_at(rotation + f64::consts::TAU * 0.25),
            shape.outer_radius_at(rotation + f64::consts::TAU * 0.75),
        );
        Self::from_width_height_rotation(width + 2.0 * padding, height + 2.0 * padding, rotation)
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
        let vertex = self.rotation
            * (vector![s_x * self.half_width, s_y * self.half_height] - self.offset)
            + self.offset;
        Some(dbg!(vertex))
    }
}

impl Polygon for Rect {
    type Vertices = RectVertices;

    fn vertices(&self) -> Self::Vertices {
        RectVertices::new(self)
    }
}

impl ShapeMut for Rect {
    fn set_center(&mut self, center: Vector2<f64>) {
        self.offset = -center;
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
pub struct RegularPolygon<const N: usize> {
    outer_radius: f64,
    rotation: f64,
    offset: Vector2<f64>,
}

impl<const N: usize> RegularPolygon<N> {
    pub fn new(outer_radius: f64, rotation: f64) -> Self {
        Self {
            outer_radius,
            rotation,
            offset: vector![0.0, 0.0],
        }
    }

    pub fn wrap(shape: &impl OuterShape, rotation: f64, padding: f64) -> Self {
        let segment_angle = f64::consts::TAU / (N as f64);
        let unpadded_inner_radius = (0..N)
            .map(|i| {
                shape.outer_radius_at(rotation + segment_angle / 2.0 + (i as f64) * segment_angle)
            })
            .reduce(f64::max)
            .unwrap_or_default();
        let inner_radius = dbg!(unpadded_inner_radius) + padding;
        let outer_radius = inner_radius / f64::cos(segment_angle / 2.0);
        Self::new(outer_radius, rotation)
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
    fn new<const N: usize>(polygon: &RegularPolygon<N>) -> Self {
        Self {
            outer_radius: polygon.outer_radius,
            base_angle: f64::consts::TAU / (N as f64),
            idx_iter: 0..N,
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

impl<const N: usize> Polygon for RegularPolygon<N> {
    type Vertices = RegularPolygonVertices;

    fn vertices(&self) -> Self::Vertices {
        RegularPolygonVertices::new(self)
    }
}

impl<const N: usize> ShapeMut for RegularPolygon<N> {
    fn set_center(&mut self, center: Vector2<f64>) {
        self.offset = -center;
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