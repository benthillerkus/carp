use carp::{Backside, Deck};

use crate::format;

impl<'a> Deck<format::Card<'a>> for format::Deck<'a> {
    /// The name of the Deck can be used in exporters to name the file.
    fn name(&self) -> &str {
        &self.name
    }

    /// A [Deck] is a collection of [Card]s.
    fn cards(&self) -> &[format::Card<'a>] {
        &self.cards
    }

    /// This is just a getter for the type of [Backside] the [Deck] uses.
    /// 
    /// The back of a card can either be the same across a [Deck] -> [`Backside::Shared`]
    /// or each [Card] can have its own backside -> [`Backside::Unique`].
    fn share_back(&self) -> Backside {
        self.back
    }
}
