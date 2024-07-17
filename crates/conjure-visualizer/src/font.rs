use anyhow::Context;

use crate::bounding;

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

    // TODO: this should probably return a Vec<bounding::Rect>
    pub fn measure(&self, text: &str, size: f32) -> bounding::Rect {
        let scale = rusttype::Scale::uniform(size);
        let v_metrics = self.0.v_metrics(scale);
        let glyphs = self.0.layout(text, scale, rusttype::point(0.0, 0.0));

        let width = glyphs
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .last()
            .unwrap_or(0.0);
        let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

        bounding::Rect::from_width_height(width.into(), height.into())
    }
}
