use piet_common::{kurbo::RoundedRect, RenderContext};
use std::error::Error;

pub mod deck;
pub mod device;
pub mod dimensions;
pub mod export;
pub mod renderer;

pub const BASE_RESOLUTION: u32 = 4096;
pub const BASE_ASPECT_RATIO: f64 = 5. / 7.2;
pub const ROWS: u32 = 7;
pub const COLUMNS: u32 = 10;

pub trait DrawableCard {
    fn index(&self) -> u32;

    fn draw(&self, ctx: &mut impl RenderContext, area: &RoundedRect);

    fn draw_back(&self, ctx: &mut impl RenderContext, area: &RoundedRect);
}

pub trait Import<T: DrawableCard> {
    fn import(&mut self) -> Result<Vec<T>, Box<dyn Error>>;
}