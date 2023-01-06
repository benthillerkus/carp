use clap::Parser;
use karten::Import;
use piet_common::Device;
use std::{error::Error, fs::File, path::PathBuf};

const BASE_RESOLUTION: u32 = 4096;
const BASE_ASPECT_RATIO: f64 = 5. / 7.2;

mod card;
mod export;
use export::export;
mod import;
use import::Importer;
mod deck;
use deck::dimensions::Dimensions;

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let dimensions = Dimensions::new(args.resolution, args.aspect_ratio);

    let mut device = Device::new()?;

    let cards = Importer::new().import()?;
    let sheets = deck::Deck::new(dimensions, cards).render(&mut device)?;

    eprintln!("Saving to file...");
    for sheet in sheets {
        let writer = File::create(&args.output)?;
        export(&sheet, writer)?;
    }
    eprintln!("Done!");
    Ok(())
}
