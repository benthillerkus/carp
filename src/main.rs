use clap::Parser;
use karte::Karte;
use karten::{
    deck::Deck,
    dimensions::Dimensions,
    export::{Export, PNGExporter},
    renderer::ImageRenderer,
    Import, BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use std::{error::Error, path::PathBuf};

mod import_csv;
mod karte;
use import_csv::CsvImporter;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/")]
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
    let mut args = Args::parse();
    args.output.push("nofile");

    let dimensions = Dimensions::new(args.resolution, args.aspect_ratio);

    let prompts = CsvImporter::new("prompts.csv").import()?;
    let prompts = Deck::new(prompts, Some(Karte::default()), "prompts".to_string());
    let quips = CsvImporter::new("quips.csv").import()?;
    let quips = Deck::new(quips, Some(Karte::default()), "quips".to_string());
    let renderer = ImageRenderer::new(dimensions);

    let mut exporter = PNGExporter {
        directory: args.output,
    };
    for artifact in prompts
        .render(&renderer)
        .chain(quips.render(&renderer))
        .filter_map(Result::ok)
    {
        exporter.export(artifact)?;
    }

    Ok(())
}
