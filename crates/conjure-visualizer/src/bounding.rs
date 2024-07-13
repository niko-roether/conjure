use std::{f64, ops::Add};

use anyhow::anyhow;
use nalgebra::{vector, Vector2};

pub trait Shape {
    fn min_x(&self) -> f64;

    fn min_y(&self) -> f64;

    fn max_x(&self) -> f64;

    fn max_y(&self) -> f64;

    fn center_of_mass(&self) -> Vector2<f64>;

    fn highest_offset(&self) -> f64;

    fn highest_offset_at_angle(&self, angle: f64) -> f64;

    fn translate(&mut self, translation: Vector2<f64>);

    fn rotate(&mut self, angle: f64);

    fn wrapping_rect(&self) -> Polygon {
        Polygon::new_rect(
            vector![self.min_x(), self.min_y()],
            vector![self.max_x(), self.max_y()],
        )
        .unwrap()
    }

    fn containing_circle(&self, padding: f64) -> Circle {
        Circle {
            radius: self.highest_offset() + padding,
            offset: vector![0.0, 0.0],
        }
    }

    fn containing_regular_polygon(&self, sides: usize, angle: f64, padding: f64) -> Polygon {
        let segment_angle = f64::consts::TAU / (sides as f64);
        let unpadded_inner_radius = (0..sides)
            .map(|i| {
                self.highest_offset_at_angle(
                    angle + segment_angle / 2.0 + (i as f64) * segment_angle,
                )
            })
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
    // the center magnitude must be lower than the radius!
    offset: Vector2<f64>,
}

impl Shape for Circle {
    fn max_x(&self) -> f64 {
        self.offset.x + self.radius
    }

    fn min_x(&self) -> f64 {
        self.offset.x - self.radius
    }

    fn max_y(&self) -> f64 {
        self.offset.y + self.radius
    }

    fn min_y(&self) -> f64 {
        self.offset.y - self.radius
    }

    fn center_of_mass(&self) -> Vector2<f64> {
        self.offset
    }

    fn highest_offset(&self) -> f64 {
        self.radius + self.offset.magnitude()
    }

    fn highest_offset_at_angle(&self, angle: f64) -> f64 {
        self.offset.x * angle.cos() + self.offset.y * angle.sin() + self.radius
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        self.offset += translation;
    }

    fn rotate(&mut self, _angle: f64) {}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    // The polygon defined by these vertices must contain the origin!
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

    pub fn new_rect(corner_1: Vector2<f64>, corner_2: Vector2<f64>) -> anyhow::Result<Self> {
        if (corner_1.x.signum() * corner_1.y.signum())
            != (corner_2.x.signum() * corner_2.y.signum())
        {
            return Err(anyhow!(
                "Rectangle defined by {corner_1} and {corner_2} doesn't contain the origin!"
            ));
        }
        let diagonal = corner_2 - corner_1;
        let corner_3 = corner_1 + vector![diagonal.x, 0.0];
        let corner_4 = corner_1 + vector![0.0, diagonal.y];
        Ok(Self {
            vertices: vec![corner_1, corner_3, corner_2, corner_4],
        })
    }

    pub fn num_sides(&self) -> usize {
        self.vertices.len()
    }
}

impl Shape for Polygon {
    fn max_x(&self) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.x)
            .reduce(f64::max)
            .unwrap_or_default()
    }

    fn min_x(&self) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.x)
            .reduce(f64::min)
            .unwrap_or_default()
    }

    fn max_y(&self) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.y)
            .reduce(f64::max)
            .unwrap_or_default()
    }

    fn min_y(&self) -> f64 {
        self.vertices
            .iter()
            .map(|v| v.y)
            .reduce(f64::min)
            .unwrap_or_default()
    }

    fn center_of_mass(&self) -> Vector2<f64> {
        self.vertices
            .iter()
            .copied()
            .reduce(Vector2::add)
            .unwrap_or_default()
            / (self.num_sides() as f64)
    }

    fn highest_offset(&self) -> f64 {
        self.vertices
            .iter()
            .map(Vector2::magnitude_squared)
            .reduce(f64::max)
            .unwrap_or_default()
            .sqrt()
    }

    fn highest_offset_at_angle(&self, angle: f64) -> f64 {
        let normal = vector![angle.cos(), angle.sin()];
        self.vertices
            .iter()
            .map(|v| v.dot(&normal))
            .reduce(f64::max)
            .unwrap_or_default()
    }

    fn translate(&mut self, translation: Vector2<f64>) {
        self.vertices.iter_mut().for_each(|v| *v += translation)
    }

    fn rotate(&mut self, angle: f64) {
        self.vertices.iter_mut().for_each(|v| {
            let current_angle = f64::atan2(v.y, v.x);
            let new_angle = current_angle + angle;
            let new_v = v.magnitude() * Vector2::new(f64::cos(new_angle), f64::sin(new_angle));
            *v = new_v
        })
    }
}
