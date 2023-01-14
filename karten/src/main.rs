use clap::Parser;
use carp::{
    dimensions::Dimensions,
    export::{Export, PNGExporter},
    renderer::ImageRenderer,
    tts::TTS,
    BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use std::fs::{self, File};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

mod format;
mod deck;
mod karte;
mod theme;

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

    let prompts = fs::read_to_string("prompts.xml")?;
    let prompts = format::Deck::try_from(prompts.as_ref())?;
    let quips = fs::read_to_string("quips.xml")?;
    let quips = format::Deck::try_from(quips.as_ref())?;

    let renderer = ImageRenderer::new(dimensions);

    let mut exporter = PNGExporter {
        directory: args.output,
    };

    for artifact in TTS::build(&prompts, &renderer)
        .chain(TTS::build(&quips, &renderer))
        .filter_map(Result::ok)
    {
        exporter.export(artifact)?;
    }

    Ok(())
}
