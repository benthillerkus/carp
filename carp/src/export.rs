use std::{
    error::Error,
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
};

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

pub struct PNGExporter {
    pub directory: PathBuf,
}

impl Export for PNGExporter {
    type Data = ImageBuf;
    type Output = PathBuf;
    fn export(
        &self,
        artifact: Artifact<Self::Data>,
    ) -> Result<Artifact<Self::Output>, Box<dyn Error>> {
        let start = std::time::Instant::now();
        trace!("Exporting artifact {:?}", artifact.to_string());

        if let Some(parent) = self.directory.parent() {
            fs::create_dir_all(parent)?;
        }

        let (pixels, artifact) = artifact.extract_data();
        let aspect_ratio = Some(AspectRatio::new(
            pixels.width() as f64,
            pixels.height() as f64,
        ));

        let path = self
            .directory
            .with_file_name(artifact.to_string())
            .with_extension("png");

        let writer = File::create(&path)?;
        let writer = BufWriter::new(writer);
        let mut header = Header::new();
        header.set_size(pixels.width() as u32, pixels.height() as u32)?;
        header.set_color(ColorType::TruecolorAlpha, 8)?;
        let options = Options::new();
        let mut encoder = Encoder::new(writer, &options);
        encoder.write_header(&header)?;
        encoder.write_image_rows(pixels.raw_pixels())?;
        encoder.finish()?;

        trace!("Exported artifact in {:?}", start.elapsed());

        Ok(Artifact {
            aspect_ratio,
            ..artifact.with_data(path)
        })
    }
}
