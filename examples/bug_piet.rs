use std::error::Error;

use druid::RenderContext;

use piet_common::kurbo::Size;
use piet_common::kurbo::Rect;

pub fn main() -> Result<(), Box<dyn Error>> {

    let size = Size::new(400.0, 200.0);

    let mut device = piet_common::Device::new()?;
    let mut target = device.bitmap_target(size.width as usize, size.height as usize, 1.0)?;
    {
        let mut ctx = target.render_context();

        ctx.clear(size.to_rect(), piet_common::Color::grey(0.1));

        let image_data: Vec<u8> = make_image_data(10, 3, [0.1, 0.2, 0.3, 0.5]);
        let image = ctx.make_image(1, 1, &image_data, piet_common::ImageFormat::RgbaSeparate)?;

        let rect = Rect::new(10.0, 10.0, 110.0, 110.0);
        ctx.draw_image(&image, rect, piet_common::InterpolationMode::NearestNeighbor);
        
        let rect = Rect::new(rect.x0 + 110.0, rect.y0, rect.x1 + 310.145, rect.y1 - 65.568898);
        ctx.draw_image(&image, rect, piet_common::InterpolationMode::Bilinear);

        ctx.finish()?;
    }

    target.save_to_file("test.png")?;

    println!("Done!");

    Ok(())
}

fn make_image_data(width: usize, height: usize, color: [f32; 4]) -> Vec<u8> {
    let color = color.map(|x| (x * 255.0) as u8);
    let mut data = vec![0; width * height * 4];
    for i in 0..height {
        for j in 0..width {
            for k in 0..4 {
                data[i * width * 4 + j * 4 + k] = color[k];
            }
        }
    }

    data
}