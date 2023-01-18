use carp::{
    artifact::{Amount, Artifact, Content},
    dimensions::{AspectRatio, Dimensions},
    export::{Export, PNGExporter},
    renderer::ImageRenderer,
    tts::TTS,
    Side, BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use clap::Parser;
use color_eyre::{eyre::Context, Help, Result};
use dotenvy::dotenv;
use log::info;
use s3::{creds::Credentials, Bucket, Region};
use std::{error::Error, path::PathBuf, str::FromStr};
use std::{
    fs::{self},
    path::Path,
};
use tts_external_api::ExternalEditorApi;
use ulid::Ulid;

mod deck;
mod format;
mod karte;
mod theme;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value = "export/")]
    output: PathBuf,

    /// The aspect ratio of a single card.
    ///
    /// The aspect ratio is defined as width / height.
    /// 1.0 is square, 2.0 is twice as wide as it is tall, etc.
    /// The default is the 5/7.2 ratio prefered by Tabletop Simulator.
    #[arg(short, long, default_value_t = BASE_ASPECT_RATIO)]
    aspect_ratio: AspectRatio,

    /// The image resolution for the deck.
    /// On non-square aspect ratios, this value determines the longer side,
    /// so the image can never be larger than resolution².
    #[arg(short, long, default_value_t = BASE_RESOLUTION)]
    resolution: u32,

    /// Whether to sync the deck into the Tabletop Simulator.
    #[arg(short, long, default_value_t = false)]
    sync_to_tts: bool,

    /// The name of the S3 bucket to export to.
    ///
    /// The bucket must exist and you must have the folliwng permissions:
    /// - `s3:GetBucketLocation`
    /// - `s3:PutObject`
    ///
    /// Credentials are read from the environment variables `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`.
    #[arg(long, env)]
    s3_bucket: String,

    #[arg(long, env)]
    s3_region: String,

    /// The S3 endpoint to use. If not set, the default endpoint for the region is used.
    /// In Minio this setting is called "Server Location".
    #[arg(long, env)]
    s3_endpoint: Option<String>,

    /// Whether to use the path style or the subdomain style for S3 URLs.
    ///
    /// Minio uses the path style per default, AWS uses the subdomain style.
    #[arg(long, env, default_value_t = false)]
    s3_path_style: bool,
}

fn main() -> Result<()> {
    let start = std::time::Instant::now();
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .install()?;
    dotenv()?;
    env_logger::init();
    let mut args = Args::parse();
    args.output.push("nofile");
    if args.output.is_relative() {
        args.output = std::env::current_dir()?.join(args.output);
    }

    let dimensions = Dimensions::new(args.resolution, args.aspect_ratio);
    let renderer = ImageRenderer::new(dimensions);

    let exporter = PNGExporter {
        directory: args.output,
    };

    let bucket = Bucket::new(
        &args.s3_bucket,
        if let Some(endpoint) = args.s3_endpoint {
            Region::Custom {
                region: args.s3_region,
                endpoint,
            }
        } else {
            Region::from_str(&args.s3_region)
                .with_context(|| format!("couldn't parse a S3 Region from {}", args.s3_region))?
        },
        Credentials::from_env().with_context(|| "couldn't build credentials from env vars")?,
    )?;

    let bucket = if args.s3_path_style {
        bucket.with_path_style()
    } else {
        bucket
    };

    bucket
        .location()
        .with_context(|| format!("couldn't get bucket location: {}", bucket.host()))
        .with_suggestion(|| format!("does {} exist in {}?", bucket.host(), bucket.region,))
        .with_suggestion(|| {
            format!(
                "the bucket is configured with {}. Maybe {} would work?",
                if bucket.is_path_style() {
                    "path style"
                } else {
                    "subdomain style"
                },
                if bucket.is_path_style() {
                    "subdomain style"
                } else {
                    "path style"
                },
            )
        })
        .with_suggestion(|| {
            format!(
                "does {:?} have the permission `s3:GetBucketLocation`?",
                bucket.credentials().access_key
            )
        })?;

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
        .map(|deck| {
            deck.iter()
                .map(|artifact| {
                    let ulid = format!("{}.png", Ulid::new());
                    bucket
                        .put_object(ulid.clone(), &fs::read(artifact.data.clone()).unwrap())
                        .unwrap();
                    let artifact = artifact.clone();
                    artifact.with_data(Path::new(&bucket.url()).join(ulid))
                })
                .collect::<Vec<_>>()
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
    let mut face = face.display().to_string().replace('\\', "//");
    if !face.starts_with("http") {
        face = format!("file://{face}");
    }
    let mut back = back.display().to_string().replace('\\', "//");
    if !back.starts_with("http") {
        back = format!("file://{back}");
    }

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
            face = "{face}",
            back = "{back}",
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
        face = "{face}",
        back = "{back}",
        sideways = {sideways},
    }})
end
}})"#,
            position.0, position.1, position.2,
        ),
    }
}
