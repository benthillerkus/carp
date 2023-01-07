use std::error::Error;

use karten::DrawableCard;
use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    ImageBuf, RenderContext,
};

use crate::device::Pool;

use self::dimensions::Dimensions;

pub mod dimensions;

const ROWS: u32 = 7;
const COLUMNS: u32 = 10;

pub struct Deck<Card: DrawableCard> {
    cards: Vec<Card>,
    dimensions: Dimensions,
    backside: Option<Card>,
    pub name: String,
}

impl<Card: DrawableCard> Deck<Card> {
    pub fn new(
        dimensions: Dimensions,
        cards: Vec<Card>,
        backside: Option<Card>,
        name: String,
    ) -> Self {
        Self {
            dimensions,
            cards,
            backside,
            name,
        }
    }

    pub fn render(
        &self,
        pool: Pool,
    ) -> impl Iterator<Item = Result<ImageBuf, Box<dyn Error>>> + '_ {
        let pool_front = pool.clone();
        let pool_back = pool.clone();

        let front = self
            .cards
            .chunks((ROWS * COLUMNS) as usize)
            .map(move |chunk| self.render_sheet(pool_front.clone(), chunk, false));

        let back = if let Some(backside) = &self.backside {
            let back = (0..1).map(move |_| {
                let mut device = pool.get().unwrap();
                let mut bitmap = self.dimensions.create_card(&mut device).unwrap();
                let mut ctx = bitmap.render_context();
                backside.draw_back(
                    &mut ctx,
                    &self.dimensions.card.to_rect().to_rounded_rect(20.),
                );
                ctx.finish()?;
                drop(ctx);
                Ok(bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?)
            });

            Box::new(back) as Box<dyn Iterator<Item = Result<ImageBuf, Box<dyn Error>>>>
        } else {
            let back = self
                .cards
                .chunks((ROWS * COLUMNS) as usize)
                .map(move |chunk| self.render_sheet(pool_back.clone(), chunk, true));

            Box::new(back) as Box<dyn Iterator<Item = Result<ImageBuf, Box<dyn Error>>>>
        };

        front.chain(back)
    }

    fn render_sheet<'a>(
        &'a self,
        pool: Pool,
        cards: &'a [Card],
        draw_back: bool,
    ) -> Result<ImageBuf, Box<dyn Error>> {
        let mut device = pool.get()?;
        let mut bitmap = self.dimensions.create_sheet(&mut device)?;
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
                if draw_back {
                    card.draw_back(ctx, &border);
                } else {
                    card.draw(ctx, &border);
                }
                Ok(())
            })?;
        }

        ctx.finish()?;
        drop(ctx);
        Ok(bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?)
    }
}
