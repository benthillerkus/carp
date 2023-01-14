pub mod artifact;
pub mod device;
pub mod dimensions;
pub mod export;
pub mod renderer;
pub mod tts;

pub const BASE_RESOLUTION: u32 = 4096;
pub const BASE_ASPECT_RATIO: f64 = 5. / 7.2;
pub const ROWS: u32 = 7;
pub const COLUMNS: u32 = 10;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub use {card::Card, deck::Deck};

mod card {
    use super::dimensions::Dimensions;
    use piet_common::RenderContext;

    pub trait Card {
        type Deck;

        fn draw(&self, deck: &Self::Deck, ctx: &mut impl RenderContext, index: u32, dimensions: &Dimensions);

        fn draw_back(
            &self,
            deck: &Self::Deck,
            ctx: &mut impl RenderContext,
            index: u32,
            dimensions: &Dimensions,
        );
    }
}

mod deck {
    use super::Card as CardTrait;

    pub trait Deck<Card: CardTrait<Deck = Self>> {
        fn name(&self) -> &str;

        fn cards(&self) -> &[Card];

        fn share_back(&self) -> bool;
    }
}
