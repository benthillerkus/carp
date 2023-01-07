use std::error::Error;

use piet_common::{
    kurbo::{Affine, Rect, RoundedRect},
    RenderContext,
};

use crate::{dimensions::Dimensions, renderer::Render, DrawableCard, COLUMNS, ROWS};

pub struct Deck<Card: DrawableCard> {
    cards: Vec<Card>,
    backside: Option<Card>,
    pub name: String,
}

impl<Card: DrawableCard> Deck<Card> {
    pub fn new(cards: Vec<Card>, backside: Option<Card>, name: String) -> Self {
        Self {
            cards,
            backside,
            name,
        }
    }

    pub fn render<'a, T: 'a>(
        &'a self,
        renderer: &'a impl Render<Output = T>,
    ) -> impl Iterator<Item = Result<T, Box<dyn Error>>> + '_ {
        let front = self
            .cards
            .chunks((ROWS * COLUMNS) as usize)
            .map(move |chunk| {
                renderer.create_sheet(|ctx, dimensions| {
                    Self::render_sheet(ctx, dimensions, chunk, false)
                })
            });

        let back = if let Some(backside) = &self.backside {
            let back = (0..1).map(move |_| {
                renderer.create_card(|ctx, dimensions| {
                    backside.draw_back(ctx, dimensions);
                    Ok(())
                })
            });

            Box::new(back) as Box<dyn Iterator<Item = Result<T, Box<dyn Error>>>>
        } else {
            let back = self
                .cards
                .chunks((ROWS * COLUMNS) as usize)
                .map(move |chunk| {
                    renderer.create_sheet(|ctx, dimensions| {
                        Self::render_sheet(ctx, dimensions, chunk, true)
                    })
                });

            Box::new(back) as Box<dyn Iterator<Item = Result<T, Box<dyn Error>>>>
        };

        front.chain(back)
    }

    fn render_sheet(
        ctx: &mut impl RenderContext,
        dimensions: &Dimensions,
        cards: &[Card],
        draw_back: bool,
    ) -> Result<(), Box<dyn Error>> {
        let card_area = Rect::from_origin_size((0., 0.), dimensions.card);
        let border = RoundedRect::from_rect(card_area, 20.);
        for card in cards {
            ctx.with_save(|ctx| {
                ctx.transform(Affine::translate((
                    (card.index() % COLUMNS) as f64 * dimensions.card.width,
                    (card.index() / COLUMNS) as f64 * dimensions.card.height,
                )));
                ctx.clip(border);
                if draw_back {
                    card.draw_back(ctx, dimensions);
                } else {
                    card.draw(ctx, dimensions);
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}
