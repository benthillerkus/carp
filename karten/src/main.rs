use carp::{
    artifact::{Amount, Artifact},
    dimensions::{AspectRatio, Dimensions},
    export::{Export, FileExporter, PNGExporter},
    renderer::ImageRenderer,
    tts::TTS,
    BASE_ASPECT_RATIO, BASE_RESOLUTION,
};
use clap::{Parser, Subcommand};
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
mod tts;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    output: Output,

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
}

#[derive(Subcommand, Debug)]
enum Output {
    /// Export the deck to a directory.
    Disk {
        #[arg(short, long, default_value = "export/")]
        directory: PathBuf,
    },
    /// Upload the deck into an S3 (compatible) bucket.
    S3 {
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
    },
}

impl Default for Output {
    fn default() -> Self {
        Output::Disk {
            directory: PathBuf::from("export/"),
        }
    }
}

struct S3Exporter {
    pub bucket: Bucket,
}

impl Export for S3Exporter {
    type Data = Vec<u8>;
    type Output = PathBuf;

    fn export(
        &self,
        artifact: Artifact<Self::Data>,
    ) -> std::result::Result<Artifact<Self::Output>, Box<dyn Error>> {
        let filename = if let Some(ref extension) = artifact.extension {
            format!("{}.{extension}", Ulid::new())
        } else {
            Ulid::new().to_string()
        };
        self.bucket.put_object(filename.clone(), &artifact.data)?;
        let artifact = artifact.clone();
        Ok(artifact.with_data(Path::new(&self.bucket.url()).join(filename)))
    }
}

impl Output {
    fn exporter(self) -> Result<Box<dyn Export<Data = Vec<u8>, Output = PathBuf>>> {
        match self {
            Output::Disk { mut directory } => {
                directory.push("nofile");
                let directory = if directory.is_relative() {
                    std::env::current_dir()?.join(directory)
                } else {
                    directory
                };

                Ok(Box::new(FileExporter { directory }))
            }
            Output::S3 {
                s3_bucket,
                s3_region,
                s3_endpoint,
                s3_path_style,
            } => {
                let bucket = Bucket::new(
                    &s3_bucket,
                    if let Some(endpoint) = s3_endpoint {
                        Region::Custom {
                            region: s3_region,
                            endpoint,
                        }
                    } else {
                        Region::from_str(&s3_region).with_context(|| {
                            format!("couldn't parse a S3 Region from {}", s3_region)
                        })?
                    },
                    Credentials::from_env()
                        .with_context(|| "couldn't build credentials from env vars")?,
                )?;

                let bucket = if s3_path_style {
                    bucket.with_path_style()
                } else {
                    bucket
                };

                bucket
                    .location()
                    .with_context(|| format!("couldn't get bucket location: {}", bucket.host()))
                    .with_suggestion(|| {
                        format!("does {} exist in {}?", bucket.host(), bucket.region,)
                    })
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

                Ok(Box::new(S3Exporter { bucket }))
            }
        }
    }
}

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
    let args = Args::parse();

    // Configure pipeline
    let exporter = args.output.exporter()?;
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

    info!("Done in {:?}", start.elapsed());

    Ok(())
}
