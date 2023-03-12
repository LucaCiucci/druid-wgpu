


// see https://github.com/gfx-rs/wgpu/blob/master/wgpu/examples/hello-triangle/main.rs
// see https://sotrh.github.io/learn-wgpu/showcase/windowless/#so-what-do-we-need-to-do

pub mod wgpu_renderer;
pub mod texture_surface;

pub use wgpu_renderer::{
    WgpuRenderer,
    RenderTools,
};

pub use texture_surface::{
    TextureSurface
};

use druid::RenderContext;

/// Creates an adapter for the default backend
pub async fn default_adapter() -> wgpu::Adapter {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Failed to find an appropriate adapter");
    return adapter;
}

/// Default render function
pub fn default_paint(ctx: &mut druid::PaintCtx, frame_data: &[u8], size: (u32, u32), row_pitch: u32) {
    let ctx_size = ctx.size();
    let data = crate::tmp_to_image_data(frame_data, size, row_pitch);
    let image = ctx.make_image(
        size.0 as usize,
        size.1 as usize,
        data.as_slice(),
        druid::piet::ImageFormat::RgbaSeparate,
    ).unwrap();
    ctx.draw_image(
        &image,
        ctx_size.to_rect(),
        //druid::piet::InterpolationMode::Bilinear
        druid::piet::InterpolationMode::NearestNeighbor
    );
}

/// Converts a temporary buffer to an image data buffer
pub fn tmp_to_image_data(data: &[u8], size: (u32, u32), row_pitch: u32) -> Vec<u8> {
    let mut image_data = Vec::new();
    image_data.resize((size.0 * size.1 * 4) as usize, 0);

    let row_size = size.0 * 4;

    for y in 0..size.1 {
        let src_offset = (y * row_pitch) as isize;
        let dst_offset = (y * size.0 * 4) as isize;

        let src_row_slice = &data[src_offset as usize..(src_offset + row_size as isize) as usize];
        let dst_row_slice = &mut image_data[dst_offset as usize..(dst_offset + row_size as isize) as usize];

        dst_row_slice.copy_from_slice(src_row_slice);
    }

    image_data
}