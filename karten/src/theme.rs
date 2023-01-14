use once_cell::sync::OnceCell;
use piet_common::*;

#[derive(Clone)]
pub struct Theme {
    pub font: FontFamily,
    pub text_size: f64,
    pub color: Color,
    pub background: Color,
    pub border_size: f64,
    pub border_color: Color,
}

static LIGHT_THEME: OnceCell<Theme> = OnceCell::new();
static DARK_THEME: OnceCell<Theme> = OnceCell::new();

impl Theme {
    pub fn light(ctx: &mut impl RenderContext) -> &'static Theme {
        LIGHT_THEME.get_or_init(|| Theme {
            font: ctx
                .text()
                .font_family("Comic Sans MS")
                .unwrap_or(FontFamily::MONOSPACE),
            text_size: 36.0,
            color: Color::BLACK,
            background: Color::WHITE,
            border_size: 16.0,
            border_color: Color::grey(0.95),
        })
    }

    pub fn dark(ctx: &mut impl RenderContext) -> &'static Theme {
        DARK_THEME.get_or_init(|| Theme {
            color: Color::WHITE,
            background: Color::BLACK,
            border_color: Color::grey(0.1),
            ..Theme::light(ctx).clone()
        })
    }
}
