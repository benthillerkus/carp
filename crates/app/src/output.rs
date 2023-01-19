//! This module contains some Impls for the [Output] subcommand enum.

use carp::export::{Export, FileExporter};
use carp_export_s3::S3Exporter;
use color_eyre::{eyre::Context, Help, Result};
use s3::{creds::Credentials, Bucket, Region};
use std::{path::PathBuf, str::FromStr};

use crate::Output;

impl Default for Output {
    fn default() -> Self {
        Output::Disk {
            directory: PathBuf::from("export/"),
        }
    }
}

impl Output {
    pub fn exporter(self) -> Result<Box<dyn Export<Data = Vec<u8>, Output = PathBuf>>> {
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

                Ok(Box::new(S3Exporter::new(bucket)))
            }
        }
    }
}
