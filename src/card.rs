use crate::DrawableCard;
use piet_common::{
    kurbo::{Point, Rect, RoundedRect, Size},
    *,
};

pub struct Card {
    pub index: u32,
    pub text: String,
}

impl DrawableCard for Card {
    fn index(&self) -> u32 {
        self.index
    }

    fn draw(self, ctx: &mut impl RenderContext, area: &RoundedRect) {
        let bebas_neue = ctx
            .text()
            .font_family("Bebas Neue")
            .unwrap_or(FontFamily::MONOSPACE);

        let border = Point::new(24.0, 64.0);

        ctx.fill(area, &Color::WHITE);
        let text = ctx
            .text()
            .new_text_layout(self.text)
            .font(bebas_neue.clone(), 42.)
            .alignment(TextAlignment::Center)
            .text_color(Color::BLACK)
            .max_width(area.width() - border.x * 2.0)
            .build()
            .unwrap();

        ctx.draw_text(&text, border);

        let number = ctx
            .text()
            .new_text_layout(format!("{}", self.index + 1))
            .font(bebas_neue, 24.)
            .alignment(TextAlignment::Center)
            .text_color(Color::BLACK)
            .max_width(area.width())
            .build()
            .unwrap();

        ctx.draw_text(&number, (0., area.height() - border.y));

        // TODO: TTS actually distorts the border for rounded rects on non 5/7.2 aspect ratio cards
        // so our border should either get distorted too (do a scale before drawing the border)
        // or we just use the TTS rects and use transparency for the "roundedness"
        ctx.stroke(area, &Color::BLACK, 8.);
    }
}
