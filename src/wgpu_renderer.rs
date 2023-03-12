use std::{ops::Deref, error::Error};
use crate::texture_surface::TextureSurface;


pub struct WgpuRenderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    my_surface: Option<TextureSurface>,
}

/// Provides the tools needed to render a scene
pub struct RenderTools<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub device: &'a mut wgpu::Device,
    pub queue: &'a mut wgpu::Queue,
    pub texture_view: &'a wgpu::TextureView,
    pub texture_desc: &'a wgpu::TextureDescriptor<'a>,
}

impl WgpuRenderer {
    /// Creates a new renderer from an adapter
    pub async fn from_adapter(adapter: wgpu::Adapter) -> Self {
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();

        Self {
            device,
            queue,
            my_surface: None,
        }
    }

    pub fn ensure_surface_for_size(&mut self, sizes: (u32, u32)) {
        let to_rebuild = if let Some(surface) = &self.my_surface {
            // if the surface is the wrong size, rebuild it
            surface.texture.size().width != sizes.0 || surface.texture.size().height != sizes.1
        } else {
            // if there is no surface, rebuild it
            true
        };
        
        if to_rebuild {
            self.my_surface = Some(TextureSurface::new(&self.device, sizes));
        }
    }

    pub fn has_surface(&self) -> bool {
        self.my_surface.is_some()
    }

    pub fn current_surface_size(&self) -> Option<(u32, u32)> {
        self.my_surface.as_ref().map(|surface| (surface.texture.size().width, surface.texture.size().height))
    }

    pub fn clear_surface(&mut self) {
        self.my_surface = None;
    }

    pub fn render(
        &mut self,
        render_pass_fn: impl FnOnce((u32, u32), &mut RenderTools),
        paint: impl FnOnce(&[u8], (u32, u32), u32),
    ) -> Result<(), Box<dyn Error>> {
        // get surface, return error if there is no surface
        let surface = self.my_surface.as_mut().ok_or("No surface to render to")?;
        let size = surface.texture.size();
        let sizes = (size.width, size.height);
        let texture_view = surface.texture.create_view(&Default::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        render_pass_fn(
            sizes,
            &mut RenderTools {
                encoder: &mut encoder,
                device: &mut self.device,
                queue: &mut self.queue,
                texture_view: &texture_view,
                texture_desc: &surface.texture_desc
            }
        );

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &surface.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &surface.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(surface.row_pitch),
                    rows_per_image: std::num::NonZeroU32::new(sizes.1),
                },
            },
            surface.texture_desc.size,
        );
        self.queue.submit(Some(encoder.finish()));

        {
            let data = pollster::block_on(Self::read_buffer(&self.device, &surface.output_buffer));

            paint(data.deref(), sizes, surface.row_pitch);
        }
        surface.output_buffer.unmap();

        Ok(())
    }

    async fn read_buffer<'a>(device: &wgpu::Device, output_buffer: &'a wgpu::Buffer) -> wgpu::BufferView<'a> {
        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        rx.receive().await.unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        data
    }
}