use std::error::Error;

use carp::{export::Export, artifact::Artifact, dimensions::AspectRatio, piet_common::ImageBuf};
use log::trace;
use mtpng::{Header, encoder::{Options, Encoder}, ColorType};

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
