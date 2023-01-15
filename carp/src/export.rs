use std::{
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use mtpng::{
    encoder::{Encoder, Options},
    ColorType, Header,
};
use piet_common::ImageBuf;

use crate::artifact::Artifact;

pub trait Export {
    type Data;
    type Output;
    fn export(
        &mut self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>>;
}

pub struct PNGExporter {
    pub directory: PathBuf,
}

impl Export for PNGExporter {
    type Data = ImageBuf;
    type Output = PathBuf;
    fn export(
        &mut self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>> {
        if let Some(parent) = self.directory.parent() {
            fs::create_dir_all(parent)?;
        }

        let (pixels, artifact) = artifact.extract_data();

        let path = self
            .directory
            .with_file_name(artifact.to_string())
            .with_extension("png");

        let writer = File::create(&path)?;
        let mut header = Header::new();
        header.set_size(pixels.width() as u32, pixels.height() as u32)?;
        header.set_color(ColorType::TruecolorAlpha, 8)?;
        let options = Options::new();
        let mut encoder = Encoder::new(writer, &options);
        encoder.write_header(&header)?;
        encoder.write_image_rows(pixels.raw_pixels())?;
        encoder.finish()?;

        Ok(artifact.with_data(path))
    }
}
