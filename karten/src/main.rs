use carp::{
    artifact::{Amount, Artifact, Content},
    dimensions::Dimensions,
    export::{Export, PNGExporter},
    renderer::ImageRenderer,
    tts::TTS,
    Side, BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use clap::Parser;
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
    if args.output.is_relative() {
        args.output = std::env::current_dir()?.join(args.output);
    }

    let dimensions = Dimensions::new(args.resolution, args.aspect_ratio);

    let prompts = fs::read_to_string("prompts.xml")?;
    let prompts = format::Deck::try_from(prompts.as_ref())?;
    let quips = fs::read_to_string("quips.xml")?;
    let quips = format::Deck::try_from(quips.as_ref())?;

    let renderer = ImageRenderer::new(dimensions);

    let mut exporter = PNGExporter {
        directory: args.output,
    };

    let api = ExternalEditorApi::new();

    let (mut prompts, mut quips): (Vec<_>, _) = TTS::build(&prompts, &renderer)
        .chain(TTS::build(&quips, &renderer))
        .filter_map(Result::ok)
        // .filter(|_| false)
        .map(|deck| exporter.export(deck).unwrap())
        .partition(|artifact| artifact.deck == "Prompts");

    spawn_deck(&api, &mut quips, (-1.2, 0.0, 0.0))?;
    spawn_deck(&api, &mut prompts, (1.2, 0.0, 0.0))?;

    Ok(())
}

fn spawn_deck(
    api: &ExternalEditorApi,
    deck: &mut [Artifact<PathBuf>],
    position: (f32, f32, f32),
) -> Result<(), Box<dyn Error>> {
    if deck.is_empty() {
        return Ok(());
    }

    // Make sure backs and fronts are next to one another
    deck.sort_unstable_by(|a, b| match (a.amount, b.amount) {
        (Amount::Single, Amount::Single) => std::cmp::Ordering::Equal,
        (Amount::Single, Amount::Multiple { .. }) => std::cmp::Ordering::Less,
        (Amount::Multiple { .. }, Amount::Single) => std::cmp::Ordering::Greater,
        (Amount::Multiple { index: ia, .. }, Amount::Multiple { index: ib, .. }) => ia.cmp(&ib),
    });

    let backs = deck
        .iter()
        .filter(|artifact| artifact.side == Side::Back)
        .cycle();

    for (front, back) in deck
        .iter()
        .filter(|artifact| artifact.side == Side::Front)
        .zip(backs)
    {
        let answer = api.execute(spawn_card_or_deck_tts(
            position,
            &front.data,
            &back.data,
            front.content,
            false,
            true,
        ))?;

        println!("{:#?}", answer);
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
