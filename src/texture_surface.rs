
/// The output of the renderer
pub struct TextureSurface {
    pub texture: wgpu::Texture,
    pub texture_desc: wgpu::TextureDescriptor<'static>,
    pub output_buffer: wgpu::Buffer,
    pub row_pitch: u32,
}

impl TextureSurface {
    pub fn new(device: &wgpu::Device, sizes: (u32, u32)) -> Self {
        let (texture, texture_desc) = new_texture(device, sizes);

        let bytes_per_row_alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        // TODO copilot ??? let row_pitch = (U32_SIZE * sizes.0 + bytes_per_row_alignment - 1) & !(bytes_per_row_alignment - 1);
        let row_pitch = (U32_SIZE * sizes.0 + bytes_per_row_alignment - 1) / bytes_per_row_alignment * bytes_per_row_alignment;

        let output_buffer_size = (row_pitch * sizes.1) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };

        let output_buffer = device.create_buffer(&output_buffer_desc);

        Self {
            texture,
            texture_desc,
            output_buffer,
            row_pitch,
        }
    }
}

pub const U32_SIZE: u32 = std::mem::size_of::<u32>() as u32;

fn new_texture(device: &wgpu::Device, sizes: (u32, u32)) -> (wgpu::Texture, wgpu::TextureDescriptor<'static>) {

    let size = wgpu::Extent3d {
        width: sizes.0,
        height: sizes.1,
        depth_or_array_layers: 1,
    };

    let texture_desc = wgpu::TextureDescriptor::<'static> {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[], // ???
    };

    let texture = device.create_texture(&texture_desc);
    //let texture_view = texture.create_view(&Default::default());

    (texture, texture_desc)
}