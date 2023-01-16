use crate::{device::Pool, dimensions::Dimensions};
use log::trace;
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
        let trace_function_start = std::time::Instant::now();
        let mut device = self.device_pool.get()?;
        let mut bitmap = device.bitmap_target(
            self.dimensions.width as usize,
            self.dimensions.height as usize,
            self.dimensions.pix_scale,
        )?;

        let mut ctx: Self::Context<'_> = bitmap.render_context();

        let trace_draw_start = std::time::Instant::now();
        draw(&mut ctx, &self.dimensions)?;
        let took = trace_draw_start.elapsed();
        ctx.finish()?;
        drop(ctx);

        let trace_convert_start = std::time::Instant::now();
        let image = bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?;

        let trace_function_end = std::time::Instant::now();
        let trace_full_d = trace_function_end.duration_since(trace_function_start);
        let trace_full_df32 = trace_full_d.as_secs_f32();
        trace!(
            "Rendered sheet in {:?} of which was {}% waiting for device, {}% drawing, {}% copying",
            trace_full_d,
            trace_draw_start
                .duration_since(trace_function_start)
                .as_secs_f32()
                / trace_full_df32
                * 100.0,
            took.as_secs_f32() / trace_full_df32 * 100.0,
            trace_function_end
                .duration_since(trace_convert_start)
                .as_secs_f32()
                / trace_full_df32
                * 100.0,
        );
        Ok(image)
    }

    fn create_card<F: FnOnce(&mut Self::Context<'_>, &Dimensions) -> Result<(), Box<dyn Error>>>(
        &self,
        draw: F,
    ) -> Result<Self::Output, Box<dyn Error>> {
        let trace_function_start = std::time::Instant::now();
        let mut device = self.device_pool.get()?;
        let mut bitmap = device.bitmap_target(
            self.dimensions.card.width as usize,
            self.dimensions.card.height as usize,
            self.dimensions.pix_scale,
        )?;

        let mut ctx: Self::Context<'_> = bitmap.render_context();

        let trace_draw_start = std::time::Instant::now();
        draw(&mut ctx, &self.dimensions)?;
        let took = trace_draw_start.elapsed();
        ctx.finish()?;
        drop(ctx);

        let trace_convert_start = std::time::Instant::now();
        let image = bitmap.to_image_buf(piet_common::ImageFormat::RgbaPremul)?;

        let trace_function_end = std::time::Instant::now();
        let trace_full_d = trace_function_end.duration_since(trace_function_start);
        let trace_full_df32 = trace_full_d.as_secs_f32();
        trace!(
            "Rendered card in {:?} of which was {}% waiting for device, {}% drawing, {}% copying",
            trace_full_d,
            trace_draw_start
                .duration_since(trace_function_start)
                .as_secs_f32()
                / trace_full_df32
                * 100.0,
            took.as_secs_f32() / trace_full_df32 * 100.0,
            trace_function_end
                .duration_since(trace_convert_start)
                .as_secs_f32()
                / trace_full_df32
                * 100.0,
        );
        Ok(image)
    }
}
