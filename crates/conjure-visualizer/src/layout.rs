use crate::visual;

#[derive(Debug, Clone, PartialEq)]
struct BoundingCircle {
    radius: f64,
}

impl BoundingCircle {
    fn wrap_around(geometry: &BoundingGeometry, padding: f64) -> Self {
        let inner_radius = match geometry {
            BoundingGeometry::None => 0.0,
            BoundingGeometry::Circle(circle) => circle.radius,
            BoundingGeometry::Rect(rect) => {
                f64::sqrt(rect.width.powi(2) + rect.height.powi(2)) / 2.0
            }
            BoundingGeometry::RegularPolygon(polygon) => polygon.outer_radius,
        };
        Self {
            radius: inner_radius + padding,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BoundingRect {
    width: f64,
    height: f64,
    orientation: f64,
}

impl BoundingRect {
    fn wrap_around(geometry: &BoundingGeometry, orientation: f64, padding: f64) -> Self {
        let (inner_width, inner_height) = match geometry {
            BoundingGeometry::None => (0.0, 0.0),
            BoundingGeometry::Circle(circle) => (2.0 * circle.radius, 2.0 * circle.radius),
            BoundingGeometry::Rect(rect) => {
                let norm_orient = rect.orientation - orientation;
                let norm_sin = f64::sin(norm_orient);
                let norm_cos = f64::cos(norm_orient);

                let width_1 = f64::abs(rect.width * norm_cos + rect.height + norm_sin);
                let width_2 = f64::abs(rect.width * norm_cos - rect.height + norm_sin);
                let width = f64::max(width_1, width_2);

                let height_1 = f64::abs(rect.width * norm_sin + rect.height * norm_cos);
                let height_2 = f64::abs(rect.width * norm_sin - rect.height * norm_cos);
                let height = f64::max(height_1, height_2);

                (width, height)
            }
            _ => todo!(),
        };
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct BoundingRegularPolygon {
    sides: usize,
    outer_radius: f64,
    orientation: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum BoundingGeometry {
    None,
    Circle(BoundingCircle),
    Rect(BoundingRect),
    RegularPolygon(BoundingRegularPolygon),
}

#[derive(Debug, Clone)]
struct LayoutParams {
    base_size: f64,
    symbol_font_size: f64,
    symbol_line_height: f64,
    phrase_font_size: f64,
    phrase_line_height: f64,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            base_size: 16.0,
            symbol_font_size: 3.0,
            symbol_line_height: 3.6,
            phrase_font_size: 1.0,
            phrase_line_height: 1.2,
        }
    }
}

trait LayoutItem {
    fn smallest_bounding_geometry(&self, params: &LayoutParams) -> BoundingGeometry;
}

impl LayoutItem for visual::Symbol {
    fn smallest_bounding_geometry(&self, params: &LayoutParams) -> BoundingGeometry {
        BoundingGeometry::Rect(BoundingRect {
            width: params.base_size * params.symbol_font_size * self.0.len() as f64,
            height: params.base_size * params.symbol_line_height,
        })
    }
}

impl LayoutItem for visual::Phrase {
    fn smallest_bounding_geometry(&self, params: &LayoutParams) -> BoundingGeometry {
        BoundingGeometry::Rect(BoundingRect {
            width: params.base_size * params.phrase_font_size * (self.0.len() + 2) as f64,
            height: params.base_size * params.phrase_line_height,
        })
    }
}
