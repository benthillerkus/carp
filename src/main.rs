use card::Card;
use clap::Parser;
use karten::{
    deck::Deck, dimensions::Dimensions, export::export, renderer::ImageRenderer,
    Import, BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use std::{error::Error, fs::File, path::PathBuf};

mod card;
mod import_csv;
use import_csv::CsvImporter;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/FFFF")]
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

    let cards = CsvImporter::new("prompts.csv").import()?;
    let deck = Deck::new(dimensions, cards, Some(Card::default()), "deck".to_string());

    let renderer = ImageRenderer::new(dimensions);

    for (index, sheet) in deck.render(&renderer).filter_map(Result::ok).enumerate() {
        let writer = File::create(
            args.output
                .with_file_name(format!("{}-{index}.png", deck.name)),
        )?;
        export(&sheet, writer)?;
    }

    Ok(())
}
