use karten::{dimensions::Dimensions, Card};
use once_cell::sync::OnceCell;
use piet_common::{kurbo::Point, *};

#[derive(Default, Clone)]
pub struct Karte {
    pub index: u32,
    pub text: String,
    pub alternate_style: bool,
}

pub mod theme {
    use super::*;

    #[derive(Clone)]
    pub struct Theme {
        pub font: FontFamily,
        pub color: Color,
        pub background: Color,
        pub border_size: f64,
        pub border_color: Color,
    }

    static LIGHT_THEME: OnceCell<Theme> = OnceCell::new();
    static DARK_THEME: OnceCell<Theme> = OnceCell::new();

    impl Theme {
        fn light(ctx: &mut impl RenderContext) -> &'static Theme {
            LIGHT_THEME.get_or_init(|| Theme {
                font: ctx
                    .text()
                    .font_family("Comic Sans MS")
                    .unwrap_or(FontFamily::MONOSPACE),
                color: Color::BLACK,
                background: Color::WHITE,
                border_size: 16.0,
                border_color: Color::grey(0.95),
            })
        }

        fn dark(ctx: &mut impl RenderContext) -> &'static Theme {
            DARK_THEME.get_or_init(|| Theme {
                color: Color::WHITE,
                background: Color::BLACK,
                border_color: Color::grey(0.1),
                ..Theme::light(ctx).clone()
            })
        }
    }

    impl Karte {
        pub fn theme(&self, ctx: &mut impl RenderContext) -> &'static Theme {
            if self.alternate_style {
                Theme::dark(ctx)
            } else {
                Theme::light(ctx)
            }
        }
    }
}

impl Card for Karte {
    fn index(&self) -> u32 {
        self.index
    }

    fn draw(&self, ctx: &mut impl RenderContext, dimensions: &Dimensions) {
        let border = Point::new(64.0, 64.0);
        let area = dimensions.card.to_rounded_rect(20.);
        let theme = self.theme(ctx);

        ctx.fill(area, &theme.background);
        let text = ctx
            .text()
            .new_text_layout(self.text.clone())
            .font(theme.font.to_owned(), 42.)
            .alignment(TextAlignment::Start)
            .text_color(theme.color)
            .max_width(area.width() - border.x * 2.0)
            .build()
            .unwrap();

        ctx.draw_text(&text, border);

        let number = ctx
            .text()
            .new_text_layout(format!("{}", self.index + 1))
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

    fn draw_back(&self, ctx: &mut impl RenderContext, dimensions: &Dimensions) {
        let area = dimensions.card.to_rounded_rect(20.);
        let theme = self.theme(ctx);

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
