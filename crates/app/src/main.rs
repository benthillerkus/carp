use carp::{
    artifact::Amount, dimensions::Dimensions, export::Export, renderer::ImageRenderer, tts::TTS,
};
use carp_export_png::PNGExporter;
use clap::Parser;
use color_eyre::Result;
use dotenvy::dotenv;
use log::info;
use std::fs::{self};
use tts_external_api::ExternalEditorApi;

mod cli;
mod deck;
mod draw;
mod format;
mod theme;
mod tts;

fn main() -> Result<()> {
    let start = std::time::Instant::now();
    // Setup stuff
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .issue_filter(|e| match e {
            color_eyre::ErrorKind::Recoverable(_) => false,
            color_eyre::ErrorKind::NonRecoverable(_) => true,
        })
        .install()?;
    dotenv()?;
    env_logger::init();
    let args = cli::Args::parse();

    // Configure pipeline
    let exporter = args.output.unwrap_or_default().exporter()?;
    let dimensions = Dimensions::new(args.resolution, args.aspect_ratio);
    let renderer = ImageRenderer::new(dimensions);
    let pngexporter = PNGExporter;

    // Run pipeline
    let input = fs::read_dir("input")?;
    input
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if let Ok(true) = entry.file_type().map(|t| t.is_file()) {
                    let path = entry.path();

                    if let Some(true) = path.extension().map(|ext| ext == "xml" || ext == "deck") {
                        return Some(path);
                    }
                }
            }
            None
        })
        .map(fs::read_to_string)
        .filter_map(Result::ok)
        .map(|s| {
            let deck = format::Deck::try_from(s.as_ref()).unwrap();

            let deck = TTS::build(&deck, &renderer)
                .map(|e| e.unwrap())
                .map(|artifact| pngexporter.export(artifact).unwrap())
                .map(|a| exporter.export(a).unwrap());

            let mut deck: Vec<_> = deck.collect();

            // Make sure backs and fronts are next to one another
            deck.sort_unstable_by(|a, b| match (a.amount, b.amount) {
                (Amount::Single, Amount::Single) => std::cmp::Ordering::Equal,
                (Amount::Single, Amount::Multiple { .. }) => std::cmp::Ordering::Less,
                (Amount::Multiple { .. }, Amount::Single) => std::cmp::Ordering::Greater,
                (Amount::Multiple { index: ia, .. }, Amount::Multiple { index: ib, .. }) => {
                    ia.cmp(&ib)
                }
            });
            deck
        })
        .enumerate()
        .for_each(|(index, deck)| {
            if args.sync_to_tts {
                let api = ExternalEditorApi::new();
                tts::spawn_deck(&api, &deck, (index as f32 * 2.4, 0.0, 0.0)).unwrap();
            }
        });

    info!("Done in {:.2?}", start.elapsed());

    Ok(())
}
