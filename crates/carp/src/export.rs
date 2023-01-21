use std::{error::Error, fs::File, io::Write, path::PathBuf};

use mtpng::{
    encoder::{Encoder, Options},
    ColorType, Header,
};
use piet_common::ImageBuf;

use log::trace;

use crate::{artifact::Artifact, dimensions::AspectRatio};

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

pub struct PNGExporter;

impl Export for PNGExporter {
    type Data = ImageBuf;
    type Output = Vec<u8>;
    fn export(
        &self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>> {
        let start = std::time::Instant::now();

        let (pixels, artifact) = artifact.extract_data();
        let aspect_ratio = Some(AspectRatio::new(
            pixels.width() as f64,
            pixels.height() as f64,
        ));

        let mut header = Header::new();
        header.set_size(pixels.width() as u32, pixels.height() as u32)?;
        header.set_color(ColorType::TruecolorAlpha, 8)?;
        let options = Options::new();
        let mut encoder = Encoder::new(Vec::new(), &options);
        encoder.write_header(&header)?;
        encoder.write_image_rows(pixels.raw_pixels())?;
        let buf = encoder.finish()?;

        trace!(
            "Exported {:?}.png in {:?}",
            artifact.to_string(),
            start.elapsed()
        );

        Ok(Artifact {
            aspect_ratio,
            extension: Some("png".into()),
            ..artifact.with_data(buf)
        })
    }
}
