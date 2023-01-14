use carp::{dimensions::Dimensions, Card as CardTrait};
use piet_common::{kurbo::Point, *};

use crate::{
    format::{self, Card, Deck, StyleAnnotation},
    theme::Theme,
};

const SHY: char = '\u{AD}';

impl<'a> CardTrait for Card<'a> {
    type Deck = Deck<'a>;

    fn draw(
        &self,
        deck: &Self::Deck,
        ctx: &mut impl RenderContext,
        index: u32,
        dimensions: &Dimensions,
    ) {
        let border = Point::new(56.0, 64.0);
        let area = dimensions.card.to_rounded_rect(20.);
        let theme = if deck.theme == format::Theme::Light {
            Theme::light(ctx)
        } else {
            Theme::dark(ctx)
        };

        ctx.fill(area, &theme.background);

        let (source, annotations) = self.styled_segments();

        let mut text = ctx
            .text()
            .new_text_layout(source)
            .font(theme.font.to_owned(), theme.text_size)
            .alignment(TextAlignment::Start)
            .text_color(theme.color)
            .max_width(area.width() - border.x * 2.0);

        let dash = ctx
            .text()
            .new_text_layout("-")
            .font(theme.font.to_owned(), theme.text_size)
            .alignment(TextAlignment::Start)
            .text_color(theme.color)
            .max_width(f64::INFINITY)
            .build()
            .unwrap();

        for annotation in annotations {
            match annotation {
                StyleAnnotation::Italic(range) => {
                    text = text.range_attribute(range, TextAttribute::Style(FontStyle::Italic))
                }
            }
        }

        let text = text.build().unwrap();

        // Place a dash at the end of each line that ends with a SHY character.
        let mut line_number = 0;
        while let (Some(line), Some(metric)) =
            (text.line_text(line_number), text.line_metric(line_number))
        {
            if let Some(last) = line.as_bytes().last() {
                if *last == SHY as u8 {
                    let hit = text.hit_test_text_position(metric.end_offset - 2);
                    ctx.draw_text(&dash, hit.point + (border.x, border.y - metric.baseline));
                }
                line_number += 1;
            }
        }

        ctx.draw_text(&text, border);

        let number = ctx
            .text()
            .new_text_layout(format!("{}", index + 1))
            .font(theme.font.to_owned(), 24.)
            .alignment(TextAlignment::Center)
            .text_color(Color::BLACK)
            .max_width(area.width())
            .build()
            .unwrap();

        ctx.draw_text(&number, (0., area.height() - border.y));

        // TODO: TTS actually distorts the border for rounded rects on non 5/7.2 aspect ratio cards
        // so our border should either get distorted too (do a scale before drawing the border)
        // or we just use the TTS rects and use transparency for the "roundedness"
        ctx.stroke(area, &theme.border_color, theme.border_size);
    }

    fn draw_back(
        &self,
        deck: &Self::Deck,
        ctx: &mut impl RenderContext,
        _index: u32,
        dimensions: &Dimensions,
    ) {
        let area = dimensions.card.to_rounded_rect(20.);
        let theme = if deck.theme == format::Theme::Light {
            Theme::light(ctx)
        } else {
            Theme::dark(ctx)
        };

        let text = ctx
            .text()
            .new_text_layout("schlimm")
            .font(theme.font.to_owned(), 64.)
            .alignment(TextAlignment::Center)
            .text_color(Color::grey(0.8))
            .max_width(area.width())
            .build()
            .unwrap();

        ctx.fill(area, &theme.background);
        ctx.draw_text(
            &text,
            (0.0, area.height() / 2.0 - text.image_bounds().height()),
        );
        ctx.stroke(area, &theme.border_color, theme.border_size);
    }
}
