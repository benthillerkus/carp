use std::{error::Error, io::Write};

use mtpng::{
    encoder::{Encoder, Options},
    ColorType, Header,
};
use piet_common::ImageBuf;

pub fn export(pixels: &ImageBuf, writer: impl Write) -> Result<(), Box<dyn Error>> {
    let mut header = Header::new();
    header.set_size(pixels.width() as u32, pixels.height() as u32)?;
    header.set_color(ColorType::TruecolorAlpha, 8)?;
    let options = Options::new();
    let mut encoder = Encoder::new(writer, &options);
    encoder.write_header(&header)?;
    encoder.write_image_rows(pixels.raw_pixels())?;
    encoder.finish()?;

    Ok(())
}
