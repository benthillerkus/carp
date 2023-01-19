use carp::{dimensions::Dimensions, Card as CardTrait};
use piet_common::{kurbo::Point, *};

use crate::{
    format::{self, Card, Deck, Style},
    theme::Theme,
};

mod break_shy;
use break_shy::BreakShyWithDash;

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

        let texts: Vec<_> = vec![Some(self.annotated_top()), self.annotated_bottom()]
            .iter()
            .filter_map(|e| e.as_ref())
            .fuse()
            .map(|(source, annotations)| {
                let mut text = ctx
                    .new_text_layout(source.to_owned())
                    .font(theme.font.to_owned(), theme.text_size)
                    .alignment(TextAlignment::Start)
                    .text_color(theme.color)
                    .max_width(area.width() - border.x * 2.0);

                for annotation in annotations {
                    match &annotation.style {
                        Style::Italic => {
                            text = text.range_attribute(
                                annotation.range.clone(),
                                TextAttribute::Style(FontStyle::Italic),
                            )
                        }
                        Style::Font(family) => {
                            if let Some(font) = ctx.text().font_family(&family) {
                                text = text.range_attribute(
                                    annotation.range.clone(),
                                    TextAttribute::FontFamily(font),
                                )
                            }
                        }
                        Style::Size(factor) => {
                            text = text.range_attribute(
                                annotation.range.clone(),
                                TextAttribute::FontSize(theme.text_size * factor),
                            )
                        }
                    }
                }

                text.build().unwrap()
            })
            .collect();

        ctx.draw_breaking_text(&texts[0], border);

        if let Some(text) = texts.get(1) {
            ctx.draw_breaking_text(
                &text,
                (
                    border.x,
                    dimensions.card.height - border.y - text.image_bounds().height(),
                ),
            );
        }

        let number = ctx
            .text()
            .new_text_layout(format!("{}", index + 1))
            .font(theme.font.to_owned(), 24.)
            .alignment(TextAlignment::Center)
            .text_color(theme.color)
            .max_width(f64::INFINITY)
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
