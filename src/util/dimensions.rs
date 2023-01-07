use std::error::Error;

use piet_common::{kurbo::Size, BitmapTarget, Device};

use crate::{BASE_ASPECT_RATIO, BASE_RESOLUTION, COLUMNS, ROWS};

#[derive(Clone, Copy)]
pub struct Dimensions {
    pub height: u32,
    pub width: u32,
    pub card: Size,
    pub pix_scale: f64,
}

impl Dimensions {
    pub fn new(max_side: u32, card_aspect_ratio: f64) -> Self {
        let deck_height;
        let deck_width;
        let card_height;
        let card_width;
        let pix_scale;

        if card_aspect_ratio > BASE_ASPECT_RATIO {
            // The deck is wider than it is tall.
            deck_width = max_side;
            let actual_card_width = deck_width as f64 / COLUMNS as f64;
            let actual_card_height = actual_card_width / card_aspect_ratio;
            deck_height = (actual_card_height * ROWS as f64) as u32;
            pix_scale = deck_height as f64 / BASE_RESOLUTION as f64;
            card_width = (BASE_RESOLUTION as f64 / COLUMNS as f64) / pix_scale;
            card_height = card_width / card_aspect_ratio;
        } else if card_aspect_ratio == BASE_ASPECT_RATIO {
            // Ensure power of 2 texture for default aspect ratio
            deck_height = max_side;
            deck_width = max_side;
            pix_scale = max_side as f64 / BASE_RESOLUTION as f64;
            card_width = BASE_RESOLUTION as f64 / COLUMNS as f64;
            card_height = BASE_RESOLUTION as f64 / ROWS as f64;
        } else {
            deck_height = max_side;
            let actual_card_height = deck_height as f64 / ROWS as f64;
            let actual_card_width = actual_card_height * card_aspect_ratio;
            deck_width = (actual_card_width * COLUMNS as f64) as u32;
            pix_scale = deck_width as f64 / BASE_RESOLUTION as f64;
            card_height = (BASE_RESOLUTION as f64 / ROWS as f64) / pix_scale;
            card_width = card_height * card_aspect_ratio;
        };

        Self {
            height: deck_height,
            width: deck_width,
            card: Size::new(card_width, card_height),
            pix_scale,
        }
    }

    pub fn create_sheet<'a>(
        &'a self,
        device: &'a mut Device,
    ) -> Result<BitmapTarget, Box<dyn Error>> {
        Ok(device.bitmap_target(self.width as usize, self.height as usize, self.pix_scale)?)
    }

    pub fn create_card<'a>(
        &'a self,
        device: &'a mut Device,
    ) -> Result<BitmapTarget, Box<dyn Error>> {
        Ok(device.bitmap_target(
            self.card.width as usize,
            self.card.height as usize,
            self.pix_scale,
        )?)
    }
}
