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

pub use {
    card::Card,
    deck::{Backside, Deck},
};

mod card {
    use std::fmt::Display;

    use super::dimensions::Dimensions;
    use piet_common::RenderContext;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Side {
        Front,
        Back,
    }

    impl Display for Side {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Side::Front => write!(f, "front"),
                Side::Back => write!(f, "back"),
            }
        }
    }

    pub trait Card {
        type Deck;

        fn draw(
            &self,
            deck: &Self::Deck,
            ctx: &mut impl RenderContext,
            index: u32,
            dimensions: &Dimensions,
        );

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
    use std::fmt::Display;

    use super::Card as CardTrait;

    /// The back of a card can be the same across a [Deck] ([Backside::Shared]) or each [Card] can have its own one ([Backside::Unique]).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum Backside {
        #[default]
        Shared,
        Unique,
    }

    impl Display for Backside {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Backside::Shared => write!(f, "shared"),
                Backside::Unique => write!(f, "unique"),
            }
        }
    }

    pub trait Deck<Card: CardTrait<Deck = Self>> {
        fn name(&self) -> &str;

        fn cards(&self) -> &[Card];

        fn share_back(&self) -> Backside;
    }
}
