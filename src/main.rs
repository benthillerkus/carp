use clap::Parser;
use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    Device, FontFamily, RenderContext, Text,
};
use std::{error::Error, fs::File, path::PathBuf};

const ROWS: u32 = 7;
const COLUMNS: u32 = 10;
const BASE_RESOLUTION: u32 = 4096;
const BASE_ASPECT_RATIO: f64 = 5. / 7.2;

mod card;
mod export;
use export::export;
mod import;
use import::Importer;

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

trait DrawableCard {
    fn index(&self) -> u32;

    fn draw(self, ctx: &mut impl RenderContext, area: &RoundedRect);
}

trait Import<T: DrawableCard> {
    fn import(&mut self) -> Result<Vec<T>, Box<dyn Error>>;
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
        // Ensure power of 2 texture for default aspect ratio
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

    let bebas_neue = ctx
        .text()
        .font_family("Bebas Neue")
        .unwrap_or(FontFamily::MONOSPACE);

    let cards = Importer::new(bebas_neue).import()?;

    let card_area = Rect::from_origin_size((0., 0.), (card_width, card_height));
    let border = RoundedRect::from_rect(card_area, 20.);
    for card in cards {
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((
                (card.index() % COLUMNS) as f64 * card_width,
                (card.index() / COLUMNS) as f64 * card_height,
            )));
            ctx.clip(border);
            card.draw(ctx, &border);
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
