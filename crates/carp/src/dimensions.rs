use std::str::FromStr;

use piet_common::kurbo::Size;

use crate::{BASE_ASPECT_RATIO, BASE_RESOLUTION, COLUMNS, ROWS};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatio(pub f64);

impl AspectRatio {
    #[must_use] pub fn new(width: f64, height: f64) -> Self {
        AspectRatio(width / height)
    }

    #[must_use] pub fn is_landscape(&self) -> bool {
        self.0 > 1.0
    }

    #[must_use] pub fn is_portrait(&self) -> bool {
        self.0 <= 1.0
    }

    #[must_use] pub fn is_square(&self) -> bool {
        self.0 == 1.0
    }

    #[must_use] pub fn is_wider_than(&self, other: AspectRatio) -> bool {
        self.0 > other.0
    }

    #[must_use] pub fn is_taller_than(&self, other: AspectRatio) -> bool {
        self.0 < other.0
    }
}

impl FromStr for AspectRatio {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        match parts.len() {
            1 => parts[0]
                .trim()
                .parse::<f64>()
                .map(AspectRatio)
                .map_err(|_| format!("invalid aspect ratio: {s} (could not parse as float)")),
            2 => {
                let width = parts[0]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid aspect ratio: {s} (could not parse width)"))?;
                let height = parts[1]
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid aspect ratio: {s} (could not parse height)"))?;
                Ok(AspectRatio::new(width, height))
            }
            _ => Err(format!("cannot parse an aspec ratio from this: {s}")),
        }
    }
}

impl std::fmt::Display for AspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "1/{:.2}", 1.0 / self.0)
    }
}

#[derive(Clone, Copy)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
    pub card: Size,
    pub pix_scale: f64,
}

impl Dimensions {
    #[must_use] pub fn new(max_side: u32, card_aspect_ratio: AspectRatio) -> Self {
        let deck_height;
        let deck_width;
        let card_height;
        let card_width;
        let pix_scale;

        if card_aspect_ratio.is_wider_than(BASE_ASPECT_RATIO) {
            // The deck is wider than it is tall.
            deck_width = max_side;
            let actual_card_width = f64::from(deck_width) / f64::from(COLUMNS);
            let actual_card_height = actual_card_width / card_aspect_ratio.0;
            deck_height = (actual_card_height * f64::from(ROWS)) as u32;
            pix_scale = f64::from(deck_height) / f64::from(BASE_RESOLUTION);
            card_width = (f64::from(BASE_RESOLUTION) / f64::from(COLUMNS)) / pix_scale;
            card_height = card_width / card_aspect_ratio.0;
        } else if card_aspect_ratio == BASE_ASPECT_RATIO {
            // Ensure power of 2 texture for default aspect ratio
            deck_height = max_side;
            deck_width = max_side;
            pix_scale = f64::from(max_side) / f64::from(BASE_RESOLUTION);
            card_width = f64::from(BASE_RESOLUTION) / f64::from(COLUMNS);
            card_height = f64::from(BASE_RESOLUTION) / f64::from(ROWS);
        } else {
            deck_height = max_side;
            let actual_card_height = f64::from(deck_height) / f64::from(ROWS);
            let actual_card_width = actual_card_height * card_aspect_ratio.0;
            deck_width = (actual_card_width * f64::from(COLUMNS)) as u32;
            pix_scale = f64::from(deck_width) / f64::from(BASE_RESOLUTION);
            card_height = (f64::from(BASE_RESOLUTION) / f64::from(ROWS)) / pix_scale;
            card_width = card_height * card_aspect_ratio.0;
        };

        Self {
            height: deck_height,
            width: deck_width,
            card: Size::new(card_width, card_height),
            pix_scale,
        }
    }
}
