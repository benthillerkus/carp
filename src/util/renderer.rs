use crate::{device::Pool, dimensions::Dimensions};
use piet_common::D2DRenderContext;
use piet_common::ImageBuf;
use piet_common::RenderContext;
use std::error::Error;

pub trait Render: Clone {
    type Context<'a>: RenderContext;
    type Output;
    fn create_sheet<F: FnOnce(&mut Self::Context<'_>, &Dimensions) -> Result<(), Box<dyn Error>>>(
        &self,
        draw: F,
    ) -> Result<Self::Output, Box<dyn Error>>;
    fn create_card<F: FnOnce(&mut Self::Context<'_>, &Dimensions) -> Result<(), Box<dyn Error>>>(
        &self,
        draw: F,
    ) -> Result<Self::Output, Box<dyn Error>>;
}

pub struct ImageRenderer<T: RenderContext> {
    device_pool: Pool,
    dimensions: Dimensions,
    __marker: std::marker::PhantomData<T>,
}

impl<T: RenderContext> Clone for ImageRenderer<T> {
    fn clone(&self) -> Self {
        Self {
            device_pool: self.device_pool.clone(),
            dimensions: self.dimensions,
            __marker: std::marker::PhantomData,
        }
    }
}

impl<T: RenderContext> ImageRenderer<T> {
    pub fn new(dimensions: Dimensions) -> Self {
        Self {
            device_pool: Pool::default(),
            dimensions,
            __marker: std::marker::PhantomData,
        }
    }
}

#[cfg(target_os = "windows")]
impl Render for ImageRenderer<D2DRenderContext<'_>> {
    type Context<'a> = D2DRenderContext<'a>;
    type Output = ImageBuf;

    fn create_sheet<
        F: FnOnce(&mut Self::Context<'_>, &Dimensions) -> Result<(), Box<dyn Error>>,
    >(
        &self,
        draw: F,
    ) -> Result<Self::Output, Box<dyn Error>> {
        let mut device = self.device_pool.get()?;
        let mut bitmap = device.bitmap_target(
            self.dimensions.width as usize,
            self.dimensions.height as usize,
            self.dimensions.pix_scale,
        )?;

        let mut ctx: Self::Context<'_> = bitmap.render_context();

        draw(&mut ctx, &self.dimensions)?;
        ctx.finish()?;
        drop(ctx);
        let image = bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?;
        Ok(image)
    }

    fn create_card<F: FnOnce(&mut Self::Context<'_>, &Dimensions) -> Result<(), Box<dyn Error>>>(
        &self,
        draw: F,
    ) -> Result<Self::Output, Box<dyn Error>> {
        let mut device = self.device_pool.get()?;
        let mut bitmap = device.bitmap_target(
            self.dimensions.card.width as usize,
            self.dimensions.card.height as usize,
            self.dimensions.pix_scale,
        )?;

        let mut ctx: Self::Context<'_> = bitmap.render_context();

        draw(&mut ctx, &self.dimensions)?;
        ctx.finish()?;
        drop(ctx);
        let image = bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?;
        Ok(image)
    }
}
