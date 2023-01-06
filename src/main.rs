use clap::Parser;
use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    Color, Device, FontFamily, LinearGradient, PietTextLayout, RenderContext, Text, TextAlignment,
    TextLayoutBuilder, UnitPoint,
};
use std::{error::Error, fs::File, path::PathBuf};

const WIDTH: u32 = 4096;
const HEIGHT: u32 = 4096;
const ROWS: u32 = 7;
const COLUMNS: u32 = 10;
const NUM_CARDS: u32 = ROWS * COLUMNS;
const CARD_WIDTH: f64 = WIDTH as f64 / COLUMNS as f64;
const CARD_HEIGHT: f64 = HEIGHT as f64 / ROWS as f64;

mod export;
use export::export;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/deck.png")]
    output: PathBuf,
}

struct CardInfo {
    index: u32,
    x: f64,
    y: f64,
}

impl CardInfo {
    fn new(index: u32) -> Self {
        Self {
            index,
            x: (index % COLUMNS) as f64 * CARD_WIDTH,
            y: (index / COLUMNS) as f64 * CARD_HEIGHT,
        }
    }
}

struct DrawableCard {
    info: CardInfo,
    index_text: PietTextLayout,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let writer = File::create(args.output)?;
    let mut device = Device::new()?;
    let mut bitmap = device.bitmap_target(WIDTH as usize, HEIGHT as usize, 1.0)?;
    let mut ctx = bitmap.render_context();

    let font = ctx
        .text()
        .font_family("Bebas Neue")
        .unwrap_or(FontFamily::SERIF);

    let error_text = ctx
        .text()
        .new_text_layout("THIS LAYOUT COULDN'T BE BUILT")
        .font(font.clone(), 24.)
        .alignment(TextAlignment::Justified)
        .text_color(Color::RED)
        .max_width(CARD_WIDTH)
        .build()?;

    let cards: Vec<_> = (0..NUM_CARDS)
        .map(CardInfo::new)
        .map(|info| {
            let index_text = ctx
                .text()
                .new_text_layout(format!("{}", info.index))
                .font(font.clone(), 36.)
                .alignment(TextAlignment::Center)
                .text_color(Color::RED)
                .max_width(CARD_WIDTH)
                .build()
                .unwrap_or_else(|_| error_text.clone());

            DrawableCard { info, index_text }
        })
        .collect();

    for card in cards {
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((card.info.x, card.info.y)));
            let rect = Rect::from_origin_size((0., 0.), (CARD_WIDTH, CARD_HEIGHT));
            let grad = LinearGradient::new(UnitPoint::TOP, UnitPoint::BOTTOM, {
                if card.info.index % 2 == 0 {
                    (Color::WHITE, Color::BLACK)
                } else {
                    (Color::BLACK, Color::WHITE)
                }
            });
            ctx.fill(rect, &grad);
            ctx.draw_text(&card.index_text, (0., CARD_HEIGHT - 64.));
            let border = RoundedRect::from_rect(rect, 21.);
            ctx.stroke(border, &Color::RED, 4.);
            Ok(())
        })?;
    }

    ctx.finish()?;
    drop(ctx);

    println!("Saving to file...");
    export(&mut bitmap, writer)?;
    println!("Done!");
    Ok(())
}
