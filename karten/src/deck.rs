use carp::Deck;

use crate::format;

impl<'a> Deck<format::Card<'a>> for format::Deck<'a> {
    fn name(&self) -> &str {
        &self.name
    }

    fn cards(&self) -> &[format::Card<'a>] {
        &self.cards
    }

    fn share_back(&self) -> bool {
        self.back == format::Back::Shared
    }
}
