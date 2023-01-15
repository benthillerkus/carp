use crate::{card::Side, Backside};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Content {
    Single,
    Sheet { rows: u16, columns: u16, total: u16 },
}

impl Display for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Single => write!(f, "single"),
            Content::Sheet {
                rows,
                columns,
                total,
            } => write!(f, "r{}c{}t{}", rows, columns, total),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Amount {
    Single,
    Multiple { index: u16, total: u16 },
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Amount::Single => write!(f, "1of1"),
            Amount::Multiple { index, total } => write!(f, "{}of{}", index, total),
        }
    }
}

pub struct Artifact<Format> {
    pub deck: String,
    pub shared: Backside,
    pub data: Format,
    pub side: Side,
    pub content: Content,
    pub amount: Amount,
}

impl<Format> Display for Artifact<Format> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}",
            self.deck, self.side, self.content, self.amount
        )
    }
}
