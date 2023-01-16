use carp::{
    artifact::{Amount, Artifact, Content},
    dimensions::{AspectRatio, Dimensions},
    export::{Export, PNGExporter},
    renderer::ImageRenderer,
    tts::TTS,
    Side, BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use clap::Parser;
use log::info;
use std::{error::Error, path::PathBuf};
use std::{
    fs::{self},
    path::Path,
};
use tts_external_api::ExternalEditorApi;

mod deck;
mod format;
mod karte;
mod theme;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/")]
    output: PathBuf,

    /// The aspect ratio of a single card. Defaults to 5w/7.2h.
    /// 1.0 is square, 2.0 is twice as wide as it is tall, etc.
    #[arg(short, long, default_value_t = BASE_ASPECT_RATIO.0)]
    aspect_ratio: f64,

    /// The image resolution for the deck. Defaults to 4096x4096.
    /// On non-square aspect ratios, this value determines the longer side,
    /// so the image can never be larger than resolution².
    #[arg(short, long, default_value_t = BASE_RESOLUTION)]
    resolution: u32,

    /// Whether to sync the deck into the Tabletop Simulator.
    ///
    sync_to_tts: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let start = std::time::Instant::now();
    env_logger::init();
    let mut args = Args::parse();
    args.output.push("nofile");
    if args.output.is_relative() {
        args.output = std::env::current_dir()?.join(args.output);
    }

    let dimensions = Dimensions::new(args.resolution, AspectRatio(args.aspect_ratio));
    let renderer = ImageRenderer::new(dimensions);

    let exporter = PNGExporter {
        directory: args.output,
    };

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

            let mut artifacts: Vec<_> = TTS::build(&deck, &renderer)
                .map(|e| e.unwrap())
                .map(|artifact| exporter.export(artifact).unwrap())
                .collect();

            // Make sure backs and fronts are next to one another
            artifacts.sort_unstable_by(|a, b| match (a.amount, b.amount) {
                (Amount::Single, Amount::Single) => std::cmp::Ordering::Equal,
                (Amount::Single, Amount::Multiple { .. }) => std::cmp::Ordering::Less,
                (Amount::Multiple { .. }, Amount::Single) => std::cmp::Ordering::Greater,
                (Amount::Multiple { index: ia, .. }, Amount::Multiple { index: ib, .. }) => {
                    ia.cmp(&ib)
                }
            });

            artifacts
        })
        .enumerate()
        .for_each(|(index, deck)| {
            if args.sync_to_tts {
                let api = ExternalEditorApi::new();
                spawn_deck(&api, &deck, (index as f32 * 2.4, 0.0, 0.0)).unwrap();
            }
        });

    info!("Done in {:?}", start.elapsed());

    Ok(())
}

fn spawn_deck(
    api: &ExternalEditorApi,
    deck: &[Artifact<PathBuf>],
    position: (f32, f32, f32),
) -> Result<(), Box<dyn Error>> {
    if deck.is_empty() {
        return Ok(());
    }

    let backs = deck
        .iter()
        .filter(|artifact| artifact.side == Side::Back)
        .cycle();

    for (front, back) in deck
        .iter()
        .filter(|artifact| artifact.side == Side::Front)
        .zip(backs)
    {
        let _ = api.execute(spawn_card_or_deck_tts(
            position,
            &front.data,
            &back.data,
            front.content,
            front.aspect_ratio.map_or(false, |a| a.is_landscape()),
            true,
        ))?;
    }
    Ok(())
}

fn spawn_card_or_deck_tts(
    position: (f32, f32, f32),
    face: &Path,
    back: &Path,
    content: Content,
    sideways: bool,
    back_is_hidden: bool,
) -> String {
    let face = face.display().to_string().replace('\\', "//");
    let back = back.display().to_string().replace('\\', "//");

    match content {
        Content::Sheet {
            columns,
            rows,
            total,
        } => {
            format!(
                r#"spawnObject({{
    type = "DeckCustom",
    position = {{{}, {}, {}}},
    snap_to_grid = true,
    callback_function = function(spawned_object)
        spawned_object.setCustomObject({{
            face = "file://{face}",
            back = "file://{back}",
            width = {columns},
            height = {rows},
            number = {total},
            sideways = {sideways},
            back_is_hidden = {back_is_hidden},
        }})
    end
}})"#,
                position.0, position.1, position.2,
            )
        }
        Content::Single => format!(
            r#"spawnObject({{
type = "CardCustom",
position = {{{}, {}, {}}},
snap_to_grid = true,
callback_function = function(spawned_object)
    spawned_object.setCustomObject({{
        face = "file://{face}",
        back = "file://{back}",
        sideways = {sideways},
    }})
end
}})"#,
            position.0, position.1, position.2,
        ),
    }
}
