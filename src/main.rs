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
const BASE_ASPECT_RATIO: f64 = 5. / 7.2;

mod export;
use export::export;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/deck.png")]
    output: PathBuf,

    /// The aspect ratio of a single card. Defaults to 5w/7.2h.
    /// 1.0 is square, 2.0 is twice as wide as it is tall, etc.
    #[arg(short, long, default_value_t = BASE_ASPECT_RATIO)]
    aspect_ratio: f64,

    /// The image resolution for the deck. Defaults to 4096x4096.
    /// On non-square aspect ratios, this value determines the longer side,
    /// so the image can never be larger than resolution².
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

    let deck_height;
    let deck_width;
    let card_height;
    let card_width;
    let pix_scale;

    if args.aspect_ratio > BASE_ASPECT_RATIO {
        // The deck is wider than it is tall.
        deck_width = args.resolution;
        let actual_card_width = deck_width as f64 / COLUMNS as f64;
        let actual_card_height = actual_card_width / args.aspect_ratio;
        deck_height = (actual_card_height * ROWS as f64) as u32;
        pix_scale = deck_height as f64 / BASE_RESOLUTION as f64;
        card_width = (BASE_RESOLUTION as f64 / COLUMNS as f64) / pix_scale;
        card_height = card_width / args.aspect_ratio;
    } else if args.aspect_ratio == BASE_ASPECT_RATIO {
        deck_height = args.resolution;
        deck_width = args.resolution;
        pix_scale = args.resolution as f64 / BASE_RESOLUTION as f64;
        card_width = BASE_RESOLUTION as f64 / COLUMNS as f64;
        card_height = BASE_RESOLUTION as f64 / ROWS as f64;
    } else {
        deck_height = args.resolution;
        let actual_card_height = deck_height as f64 / ROWS as f64;
        let actual_card_width = actual_card_height * args.aspect_ratio;
        deck_width = (actual_card_width * COLUMNS as f64) as u32;
        pix_scale = deck_width as f64 / BASE_RESOLUTION as f64;
        card_height = (BASE_RESOLUTION as f64 / ROWS as f64) / pix_scale;
        card_width = card_height * args.aspect_ratio;
    }

    let writer = File::create(args.output)?;
    let mut device = Device::new()?;
    let mut bitmap = device.bitmap_target(deck_width as usize, deck_height as usize, pix_scale)?;
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
        .max_width(card_width)
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
                .max_width(card_width)
                .build()
                .unwrap_or_else(|_| error_text.clone());

            DrawableCard { info, index_text }
        })
        .collect();

    let card_area = Rect::from_origin_size((0., 0.), (card_width, card_height));
    for card in cards {
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((
                (card.info.index % COLUMNS) as f64 * card_width,
                (card.info.index / COLUMNS) as f64 * card_height,
            )));
            let grad = LinearGradient::new(UnitPoint::TOP, UnitPoint::BOTTOM, {
                if card.info.index % 2 == 0 {
                    (Color::WHITE, Color::BLACK)
                } else {
                    (Color::BLACK, Color::WHITE)
                }
            });
            ctx.fill(card_area, &grad);
            ctx.draw_text(&card.index_text, (0., card_height - 64.));
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
