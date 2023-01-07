use piet_common::{kurbo::RoundedRect, RenderContext};
use std::error::Error;

pub trait DrawableCard {
    fn index(&self) -> u32;

    fn draw(&self, ctx: &mut impl RenderContext, area: &RoundedRect);

    fn draw_back(&self, ctx: &mut impl RenderContext, area: &RoundedRect);
}

pub trait Import<T: DrawableCard> {
    fn import(&mut self) -> Result<Vec<T>, Box<dyn Error>>;
}
