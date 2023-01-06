use std::error::Error;

use piet_common::FontFamily;

use crate::{card::Card, Import};

pub struct Importer {}

impl Importer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Import<Card> for Importer {
    fn import(&mut self) -> Result<Vec<Card>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path("prompts.csv")?;
        let cards = reader
            .records()
            .filter_map(|r| r.ok())
            .enumerate()
            .map(|(index, r)| Card {
                index: index as u32,
                text: r
                    .get(0)
                    .map_or_else(|| String::from("COULDN'T IMPORT"), |s| s.to_owned()),
            })
            .collect();

        Ok(cards)
    }
}
