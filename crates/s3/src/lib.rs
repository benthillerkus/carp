use std::{path::{PathBuf, Path}, error::Error};

use carp::{export::Export, artifact::Artifact};
use s3::Bucket;
use ulid::Ulid;

pub struct S3Exporter {
    pub bucket: Bucket,
}

impl S3Exporter {
    #[must_use] pub fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }
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
        let artifact = artifact;
        Ok(artifact.with_data(Path::new(&self.bucket.url()).join(filename)))
    }
}
