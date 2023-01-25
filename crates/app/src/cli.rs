//! This module contains some Impls for the [Output] subcommand enum.

use carp::{
    dimensions::AspectRatio,
    export::{Export, FileExporter},
};
use carp::{BASE_ASPECT_RATIO, BASE_RESOLUTION};
use carp_export_s3::S3Exporter;
use clap::{Parser, Subcommand};
use color_eyre::{
    eyre::{eyre, Context},
    Help, Result,
};
use s3::{creds::Credentials, Bucket, Region};
use std::{fs, path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub(crate) struct Args {
    /// The aspect ratio of a single card.
    ///
    /// The aspect ratio is defined as width / height.
    /// 1.0 is square, 2.0 is twice as wide as it is tall, etc.
    /// The default is the 5/7.2 ratio prefered by Tabletop Simulator.
    #[arg(short, long, env, default_value_t = BASE_ASPECT_RATIO)]
    pub aspect_ratio: AspectRatio,

    /// The image resolution for the deck.
    /// On non-square aspect ratios, this value determines the longer side,
    /// so the image can never be larger than resolution².
    #[arg(short, long, env, default_value_t = BASE_RESOLUTION)]
    pub resolution: u32,

    /// Whether to sync the deck into the Tabletop Simulator.
    #[arg(short, long, default_value_t = false)]
    pub sync_to_tts: bool,

    /// The deck template .xml files to load.
    #[arg(short, long, env)]
    pub input: Vec<PathBuf>,

    /// Where should the deck be exported to?
    #[command(subcommand)]
    pub output: Option<Output>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Output {
    /// Export the deck to a directory.
    Disk {
        #[arg(short, long, default_value = "export", env)]
        directory: PathBuf,

        /// Whether to create the directory if it doesn't exist.
        #[arg(long)]
        create: bool,
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
            directory: PathBuf::from("export"),
            create: true,
        }
    }
}

impl Output {
    pub fn exporter(self) -> Result<Box<dyn Export<Data = Vec<u8>, Output = PathBuf>>> {
        match self {
            Output::Disk { directory, create } => {
                if create {
                    fs::create_dir_all(&directory)?;
                } else if !directory.is_dir() {
                    Err(eyre!(
                        "the directory {:?} doesn't exist {}. Use --create to create it",
                        directory,
                        if directory.is_relative() {
                            format!("in {}", std::env::current_dir()?.display())
                        } else {
                            String::new()
                        }
                    ))?
                }

                let directory = directory.canonicalize()?;

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

                Ok(Box::new(S3Exporter::new(bucket)))
            }
        }
    }
}
