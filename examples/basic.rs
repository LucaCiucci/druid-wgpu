
use std::borrow::Cow;

use druid::{Size, WindowDesc, LocalizedString, AppLauncher, RenderContext};



fn main() {
    println!("Hello, world!");

    let adapter = pollster::block_on(
        druid_wgpu_utils::default_adapter()
    );

    let mut renderer = pollster::block_on(
        druid_wgpu_utils::WgpuRenderer::from_adapter(adapter)
    );

    let my_widget = MyWidget {
        wgpu_renderer: renderer,
        my_renderer: MyRenderer::default(),
    };

    let window = WindowDesc::new(my_widget)
        .title(LocalizedString::new("WGPU!"))
        .window_size((400.0, 300.0));
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(())
        .expect("launch failed");
}

struct MyWidget {
    wgpu_renderer: druid_wgpu_utils::WgpuRenderer,
    my_renderer: MyRenderer,
}

impl MyWidget {
}

impl druid::Widget<()> for MyWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut (), env: &druid::Env) {
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &(), env: &druid::Env) {
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &(), data: &(), env: &druid::Env) {
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &(), env: &druid::Env) -> druid::Size {
        if bc.is_width_bounded() && bc.is_height_bounded() {
            bc.max()
        } else {
            let size = Size::new(100.0, 100.0);
            bc.constrain(size)
        }
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &(), env: &druid::Env) {
        let size = ctx.size();
        ctx.clear(size.to_rect(), druid::Color::grey(0.1));
        self.wgpu_renderer.ensure_surface_for_size((size.width as u32, size.height as u32));
        self.wgpu_renderer.render(
            |size, tools| {
                self.my_renderer.render(size, tools);
            },
            |frame_data, size, row_pitch| {
                druid_wgpu_utils::default_paint(ctx, frame_data, size, row_pitch);
            }
        ).unwrap();
    }
}

trait Renderer {
    fn render(
        &mut self,
        size: (u32, u32),
        tools: &mut druid_wgpu_utils::RenderTools,
    );
}

struct MyRendererState {
    triangle_shader: Option<wgpu::ShaderModule>,
    triangle_pipeline: Option<wgpu::RenderPipeline>,
}

impl Default for MyRendererState {
    fn default() -> Self {
        Self {
            triangle_shader: None,
            triangle_pipeline: None,
        }
    }
}

struct MyRenderer {
    state: MyRendererState,
}

impl Default for MyRenderer {
    fn default() -> Self {
        Self {
            state: MyRendererState::default(),
        }
    }
}

impl MyRenderer {

    fn get_triangle_shader(&mut self, device: &wgpu::Device) -> &wgpu::ShaderModule {
        if self.state.triangle_shader.is_none() {
            self.state.triangle_shader = Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            }));
        }
        self.state.triangle_shader.as_ref().unwrap()
    }

    fn get_triangle_pipeline(&mut self, device: &wgpu::Device, texture_desc: &wgpu::TextureDescriptor) -> &wgpu::RenderPipeline {

        if self.state.triangle_pipeline.is_none() {
            let shader = self.get_triangle_shader(device);

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: texture_desc.format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

            self.state.triangle_pipeline = Some(render_pipeline);
        }

        self.state.triangle_pipeline.as_ref().unwrap()
    }
}

impl Renderer for MyRenderer {
    fn render(
        &mut self,
        size: (u32, u32),
        tools: &mut druid_wgpu_utils::RenderTools,
    ) {
        let desc = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: tools.texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 0.5,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };
        let mut render_pass = tools.encoder.begin_render_pass(&desc);

        let render_pipeline = self.get_triangle_pipeline(tools.device, tools.texture_desc);

        render_pass.set_pipeline(&render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}