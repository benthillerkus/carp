use piet_common::kurbo::{Affine, Rect, RoundedRect};
use piet_common::RenderContext;

use crate::{
    artifact::Artifact, dimensions::Dimensions, renderer::Render, Card as CardTrait,
    Deck as DeckTrait, Result, COLUMNS, ROWS,
};

pub struct TTS;

impl TTS {
    pub fn build<'a, Format, Deck, Card>(
        deck: &'a Deck,
        renderer: &'a impl Render<Output = Format>,
    ) -> impl Iterator<Item = Result<Artifact<Format>>> + 'a
    where
        Format: 'a,
        Deck: DeckTrait<Card>,
        Card: CardTrait<Deck = Deck> + 'a,
    {
        let front = TTS::render_sheet(renderer, deck, false);

        let back = if deck.share_back() {
            Box::new((0..1).map(move |_| {
                renderer
                    .create_card(|ctx, dimensions| {
                        deck.cards()
                            .first()
                            .unwrap()
                            .draw_back(deck, ctx, 0, dimensions);
                        Ok(())
                    })
                    .map(|image| Artifact {
                        name: format!("{}-back-single", deck.name()),
                        data: image,
                    })
            }))
        } else {
            Box::new(TTS::render_sheet(renderer, deck, true)) as Box<dyn Iterator<Item = _>>
        };

        front.chain(back)
    }

    fn render_sheet<'a, Format, Deck, Card>(
        renderer: &'a impl Render<Output = Format>,
        deck: &'a Deck,
        backside: bool,
    ) -> impl Iterator<Item = Result<Artifact<Format>>> + 'a
    where
        Format: 'a,
        Deck: DeckTrait<Card>,
        Card: CardTrait<Deck = Deck> + 'a,
    {
        deck.cards()
            .chunks((ROWS * COLUMNS) as usize)
            .enumerate()
            .map(move |(page, chunk)| {
                renderer
                    .create_sheet(|ctx, dimensions| {
                        TTS::draw_sheet(ctx, dimensions, deck, page as u32, chunk, backside)
                    })
                    .map(|image| Artifact {
                        name: if deck.cards().len() <= (ROWS * COLUMNS) as usize {
                            format!("{}-contains-{}", deck.name(), chunk.len())
                        } else {
                            format!("{}-contains-{}-page-{page}", deck.name(), chunk.len())
                        },
                        data: image,
                    })
            })
    }

    fn draw_sheet<Deck, Card>(
        ctx: &mut impl RenderContext,
        dimensions: &Dimensions,
        deck: &Deck,
        page: u32,
        cards: &[Card],
        backside: bool,
    ) -> Result<()>
    where
        Deck: DeckTrait<Card>,
        Card: CardTrait<Deck = Deck>,
    {
        let card_area = Rect::from_origin_size((0., 0.), dimensions.card);
        let border = RoundedRect::from_rect(card_area, 20.);
        for (index, card) in cards.iter().enumerate() {
            ctx.with_save(|ctx| {
                let index = index as u32 + page * (ROWS * COLUMNS);
                ctx.transform(Affine::translate((
                    (index % COLUMNS) as f64 * dimensions.card.width,
                    (index / COLUMNS) as f64 * dimensions.card.height,
                )));
                ctx.clip(border);
                if backside {
                    card.draw_back(deck, ctx, index, dimensions);
                } else {
                    card.draw(deck, ctx, index, dimensions);
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}
