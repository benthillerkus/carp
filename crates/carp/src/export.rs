use crate::artifact::Artifact;
use std::{error::Error, fs::File, io::Write, path::PathBuf};

pub trait Export {
    type Data;
    type Output;
    fn export(
        &self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>>;
}

/// An exporter that writes files to disk.
/// It takes Bytes and writes them as files in the given directory.
pub struct FileExporter {
    /// The directory in which files will be placed.
    pub directory: PathBuf,
}

impl Export for FileExporter {
    type Data = Vec<u8>;
    type Output = PathBuf;

    fn export(
        &self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>> {
        let mut path = self.directory.join(artifact.to_string());

        if let Some(ref fileformat) = artifact.extension {
            path = path.with_extension(fileformat);
        }

        let mut writer = File::create(&path)?;

        writer.write_all(&artifact.data)?;

        Ok(Artifact {
            aspect_ratio: artifact.aspect_ratio,
            ..artifact.with_data(path)
        })
    }
}
