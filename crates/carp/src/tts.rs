use piet_common::kurbo::{Affine, Rect, RoundedRect};
use piet_common::RenderContext;

use crate::card::Side;
use crate::deck::Backside;
use crate::{
    artifact::{Amount, Artifact, Content},
    dimensions::Dimensions,
    renderer::Render,
    Card as CardTrait, Deck as DeckTrait, Result, COLUMNS, ROWS,
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
        let front = TTS::render_sheet(renderer, deck, Side::Front);

        let back = if deck.share_back() == Backside::Shared {
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
                        deck: deck.name().into(),
                        data: image,
                        side: Side::Back,
                        shared: deck.share_back(),
                        content: Content::Single,
                        amount: Amount::Single,
                        aspect_ratio: None,
                        extension: None,
                    })
            }))
        } else {
            Box::new(TTS::render_sheet(renderer, deck, Side::Back)) as Box<dyn Iterator<Item = _>>
        };

        front.chain(back)
    }

    fn render_sheet<'a, Format, Deck, Card>(
        renderer: &'a impl Render<Output = Format>,
        deck: &'a Deck,
        side: Side,
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
                        TTS::draw_sheet(ctx, dimensions, deck, page as u32, chunk, side)
                    })
                    .map(|image| Artifact {
                        deck: deck.name().into(),
                        data: image,
                        side,
                        shared: deck.share_back(),
                        aspect_ratio: Default::default(),
                        content: Content::Sheet {
                            rows: ROWS as u16,
                            columns: COLUMNS as u16,
                            total: chunk.len() as u16,
                        },
                        amount: {
                            let in_deck = deck.cards().len() as u32;
                            let per_page = ROWS * COLUMNS;

                            if in_deck <= per_page {
                                Amount::Single
                            } else {
                                Amount::Multiple {
                                    index: page as u16 + 1,
                                    total: {
                                        let total = in_deck / per_page;
                                        let remainder = in_deck % per_page;
                                        total as u16 + u16::from(remainder > 0)
                                    },
                                }
                            }
                        },
                        extension: Default::default(),
                    })
            })
    }

    fn draw_sheet<Deck, Card>(
        ctx: &mut impl RenderContext,
        dimensions: &Dimensions,
        deck: &Deck,
        page: u32,
        cards: &[Card],
        side: Side,
    ) -> Result<()>
    where
        Deck: DeckTrait<Card>,
        Card: CardTrait<Deck = Deck>,
    {
        let card_area = Rect::from_origin_size((0., 0.), dimensions.card);
        let border = RoundedRect::from_rect(card_area, 20.);
        for (index, card) in cards.iter().enumerate() {
            ctx.with_save(|ctx| {
                let sheet_index = index as u32;
                ctx.transform(Affine::translate((
                    f64::from(sheet_index % COLUMNS) * dimensions.card.width,
                    f64::from(sheet_index / COLUMNS) * dimensions.card.height,
                )));
                ctx.clip(border);
                let deck_index = page * (ROWS * COLUMNS);
                if side == Side::Back {
                    card.draw_back(deck, ctx, deck_index, dimensions);
                } else {
                    card.draw(deck, ctx, deck_index, dimensions);
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}
