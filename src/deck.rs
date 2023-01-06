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

    pub fn render<'a>(
        &'a self,
        device: &'a mut Device,
    ) -> impl Iterator<Item = Result<ImageBuf, Box<dyn Error>>> + '_ {
        self.cards
            .chunks((ROWS * COLUMNS) as usize)
            .map(|chunk| self.render_sheet(device, chunk))
    }

    fn render_sheet<'a>(
        &'a self,
        device: &'a mut Device,
        cards: &'a [Card],
    ) -> Result<ImageBuf, Box<dyn Error>> {
        let mut bitmap = device.bitmap_target(
            self.dimensions.width as usize,
            self.dimensions.height as usize,
            self.dimensions.pix_scale,
        )?;
        let mut ctx = bitmap.render_context();

        let card_area = Rect::from_origin_size((0., 0.), self.dimensions.card);
        let border = RoundedRect::from_rect(card_area, 20.);
        for card in cards {
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
        Ok(bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?)
    }
}
