use clap::Parser;
use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    Color, Device, FontFamily, LinearGradient, PietTextLayout, RenderContext, Text, TextAlignment,
    TextLayoutBuilder, UnitPoint,
};
use std::{error::Error, fs::File, path::PathBuf};

const ROWS: u32 = 7;
const COLUMNS: u32 = 10;
const NUM_CARDS: u32 = ROWS * COLUMNS;
const BASE_RESOLUTION: u32 = 4096;
const CARD_HEIGHT: f64 = BASE_RESOLUTION as f64 / ROWS as f64;

mod export;
use export::export;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/deck.png")]
    output: PathBuf,

    #[arg(short, long, default_value_t = 5./7.2)]
    aspect_ratio: f64,

    #[arg(short, long, default_value_t = BASE_RESOLUTION)]
    resolution: u32,
}

struct CardInfo {
    index: u32,
    number: u32,
}

impl CardInfo {
    fn new(index: u32) -> Self {
        Self {
            index,
            number: index + 1,
        }
    }
}

struct DrawableCard {
    info: CardInfo,
    index_text: PietTextLayout,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let deck_height = args.resolution;
    let actual_card_height = args.resolution as f64 / ROWS as f64;
    let actual_card_width = actual_card_height * args.aspect_ratio;
    let deck_width = (actual_card_width * COLUMNS as f64) as u32;
    #[allow(non_snake_case)]
    let CARD_WIDTH = CARD_HEIGHT * args.aspect_ratio;

    let writer = File::create(args.output)?;
    let mut device = Device::new()?;
    let mut bitmap = device.bitmap_target(
        deck_width as usize,
        deck_height as usize,
        args.resolution as f64 / BASE_RESOLUTION as f64,
    )?;
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
                .new_text_layout(format!("{}", info.number))
                .font(font.clone(), 36.)
                .alignment(TextAlignment::Center)
                .text_color(Color::RED)
                .max_width(CARD_WIDTH)
                .build()
                .unwrap_or_else(|_| error_text.clone());

            DrawableCard { info, index_text }
        })
        .collect();

    let card_area = Rect::from_origin_size((0., 0.), (CARD_WIDTH, CARD_HEIGHT));
    for card in cards {
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((
                (card.info.index % COLUMNS) as f64 * CARD_WIDTH,
                (card.info.index / COLUMNS) as f64 * CARD_HEIGHT,
            )));
            let grad = LinearGradient::new(UnitPoint::TOP, UnitPoint::BOTTOM, {
                if card.info.index % 2 == 0 {
                    (Color::WHITE, Color::BLACK)
                } else {
                    (Color::BLACK, Color::WHITE)
                }
            });
            ctx.fill(card_area, &grad);
            ctx.draw_text(&card.index_text, (0., CARD_HEIGHT - 64.));
            let border = RoundedRect::from_rect(card_area, 20.);
            // TODO: TTS actually distorts the border for rounded rects on non 5/7.2 aspect ratio cards
            // so our border should either get distorted too (do a scale before drawing the border)
            // or we just use the TTS rects and use transparency for the "roundedness"
            ctx.stroke(border, &Color::RED, 8.);
            Ok(())
        })?;
    }

    ctx.finish()?;
    drop(ctx);

    println!("Saving to file...");
    export(&mut bitmap, deck_width, deck_height, writer)?;
    println!("Done!");
    Ok(())
}
