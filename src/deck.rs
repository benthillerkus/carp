use std::error::Error;

use karten::DrawableCard;
use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    Device, ImageBuf, RenderContext,
};

use self::dimensions::Dimensions;

pub mod dimensions;

const ROWS: u32 = 7;
const COLUMNS: u32 = 10;

pub struct Deck<Card: DrawableCard> {
    cards: Vec<Card>,
    dimensions: Dimensions,
}

impl<Card: DrawableCard> Deck<Card> {
    pub fn new(dimensions: Dimensions, cards: Vec<Card>) -> Self {
        Self { dimensions, cards }
    }

    pub fn render(self, device: &mut Device) -> Result<Vec<ImageBuf>, Box<dyn Error>> {
        let mut sheets = Vec::new();
        for chunk in self.cards.chunks((ROWS * COLUMNS) as usize) {
            let mut bitmap = device.bitmap_target(
                self.dimensions.width as usize,
                self.dimensions.height as usize,
                self.dimensions.pix_scale,
            )?;
            let mut ctx = bitmap.render_context();

            let card_area = Rect::from_origin_size((0., 0.), self.dimensions.card);
            let border = RoundedRect::from_rect(card_area, 20.);
            for card in chunk {
                ctx.with_save(|ctx| {
                    ctx.transform(Affine::translate((
                        (card.index() % COLUMNS) as f64 * self.dimensions.card.width,
                        (card.index() / COLUMNS) as f64 * self.dimensions.card.height,
                    )));
                    ctx.clip(border);
                    card.draw(ctx, &border);
                    Ok(())
                })?;
            }

            ctx.finish()?;
            drop(ctx);
            sheets.push(bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?);
        }
        Ok(sheets)
    }
}
