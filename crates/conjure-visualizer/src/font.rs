use anyhow::Context;
use nalgebra::vector;

use crate::bounding::{self, ShapeMut};

const ZEYADA_REGULAR: &[u8] = include_bytes!("../assets/Zeyada-Regular.ttf");

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BuiltinFont {
    Zeyada,
}

pub struct Font(rusttype::Font<'static>);

impl Font {
    pub fn builtin(font: BuiltinFont) -> anyhow::Result<Self> {
        let font_data = match font {
            BuiltinFont::Zeyada => ZEYADA_REGULAR,
        };
        let font = rusttype::Font::try_from_bytes(font_data)
            .context("Failed to load builtin font: {font:?}")?;
        Ok(Self(font))
    }

    pub fn measure(&self, text: &str, size: f32) -> Vec<bounding::Rect> {
        let lines: Vec<_> = text.lines().collect();

        let scale = rusttype::Scale::uniform(size);
        let v_metrics = self.0.v_metrics(scale);

        let text_height = f64::from(v_metrics.ascent - v_metrics.descent);
        let line_height = text_height + f64::from(v_metrics.line_gap);
        let line_widths = lines.iter().map(|line| self.measure_line(line, scale));

        let line_rects = line_widths.enumerate().map(|(i, width)| {
            let v_offset = (i as f64 - 0.5 * (lines.len() as f64 - 1.0)) * line_height;
            let mut rect = bounding::Rect::from_width_height(width, text_height);
            rect.translate(vector![0.0, v_offset]);
            rect
        });

        line_rects.collect()
    }

    fn measure_line(&self, line: &str, scale: rusttype::Scale) -> f64 {
        let glyphs = self.0.layout(line, scale, rusttype::point(0.0, 0.0));

        glyphs
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .last()
            .unwrap_or(0.0)
            .into()
    }
}
