use std::{error::Error, path::PathBuf};

use karten::Import;

use crate::card::Card;

pub struct CsvImporter {
    pub path: PathBuf,
}

impl CsvImporter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl Import<Card> for CsvImporter {
    fn import(&mut self) -> Result<Vec<Card>, Box<dyn Error>> {
        let mut reader = csv::Reader::from_path(&self.path)?;
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
