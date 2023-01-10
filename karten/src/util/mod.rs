use piet_common::RenderContext;
use self::dimensions::Dimensions;

pub mod deck;
pub mod device;
pub mod dimensions;
pub mod renderer;
pub mod artifact;
pub mod export;

pub const BASE_RESOLUTION: u32 = 4096;
pub const BASE_ASPECT_RATIO: f64 = 5. / 7.2;
pub const ROWS: u32 = 7;
pub const COLUMNS: u32 = 10;

pub trait Card {
    fn index(&self) -> u32;

    fn draw(&self, ctx: &mut impl RenderContext, dimensions: &Dimensions);

    fn draw_back(&self, ctx: &mut impl RenderContext, dimensions: &Dimensions);
}
