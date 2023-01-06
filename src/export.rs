use std::{error::Error, io::Write};

use mtpng::{
    encoder::{Encoder, Options},
    ColorType, Header,
};
use piet_common::BitmapTarget;

use crate::{HEIGHT, WIDTH};

pub fn export(bitmap: &mut BitmapTarget, writer: impl Write) -> Result<(), Box<dyn Error>> {
    let mut header = Header::new();
    header.set_size(WIDTH, HEIGHT)?;
    header.set_color(ColorType::TruecolorAlpha, 8)?;
    let options = Options::new();
    let mut encoder = Encoder::new(writer, &options);
    encoder.write_header(&header)?;
    let buf = bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?;
    encoder.write_image_rows(buf.raw_pixels())?;
    encoder.finish()?;

    Ok(())
}
