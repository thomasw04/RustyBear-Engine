use std::default::Default;

use wgpu::TextureView;

use crate::{
    assets::{
        assets::{AssetType, Assets},
        buffer::Vertices,
        shader::{Shader, ShaderVariant},
    },
    context::{Context, VisContext},
    event::{self, EventSubscriber},
    utils::Guid,
};

use super::camera::CameraBuffer;
use super::factory::{PipelineFactory, RenderPipelineConfig};
use super::framebuffer::Framebuffer;
use super::material::SkyboxMaterial;
use super::types::{BindGroup, FragmentShader, IndexBuffer, VertexBuffer, VertexShader};

pub struct Renderer {
    framebuffer: Framebuffer,
    assets: Assets,
    pipelines: PipelineFactory,
    camera_buffer: CameraBuffer,
    skybox: Option<SkyboxMaterial>,
    egui_renderer: egui_wgpu::Renderer,
}

impl EventSubscriber for Renderer {
    fn on_event(&mut self, event: &event::Event, context: &mut Context) -> bool {
        match event {
            event::Event::Resized { width, height } => {
                self.framebuffer.resize(context, *width, *height);
                false
            }
            _ => false,
        }
    }
}

impl Renderer {
    pub fn new(context: &Context, mut assets: Assets) -> Self {
        //Renderable setup
        let sample_count = 4;

        let pipelines = PipelineFactory::new();

        let framebuffer = Framebuffer::new(context, sample_count);

        let sky_shader = assets.consume_asset(
            AssetType::Shader(
                Shader::new(
                    &context.graphics,
                    Guid::dead(),
                    wgpu::ShaderSource::Wgsl(include_str!("../assets/skybox.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            ),
            None::<&str>,
        );

        //TODO add capability to what for material asset type containing textures, shaders, etc.
        let sky_tex = assets.request_asset("data/skybox.fur", 0);

        let skybox = assets
            .get(&sky_tex)
            .map(|sky_tex| SkyboxMaterial::new(&context.graphics, sky_shader, sky_shader, sky_tex));

        let camera_buffer = CameraBuffer::new(&context.graphics, "Default Camera");

        let egui_renderer = Renderer::recreate_gui(context, sample_count);

        Renderer { framebuffer, assets, pipelines, camera_buffer, skybox, egui_renderer }
    }

    pub(crate) fn recreate_gui(context: &Context, sample_count: u32) -> egui_wgpu::Renderer {
        egui_wgpu::Renderer::new(
            &context.graphics.device,
            context.surface_config.format,
            None,
            sample_count,
        )
    }

    pub fn enable_msaa(&mut self, context: &mut Context, sample_count: u32) -> bool {
        if self.framebuffer.change_sample_count(context, sample_count) {
            //TODO
            return true;
        }
        false
    }

    pub fn update_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        self.camera_buffer.update_buffer(context, camera);
    }

    pub fn update_skybox_buffer(
        &mut self, context: &VisContext, view: [[f32; 4]; 4], projection: [[f32; 4]; 4],
    ) {
        if let Some(ref mut skybox) = &mut self.skybox {
            skybox.update_buffer(context, view, projection);
        }
    }

    pub fn render(
        &mut self, context: &mut Context, view: &TextureView, window: &winit::window::Window,
    ) {
        let gpu = context.graphics.as_ref();
        let assets = &mut self.assets;

        let _ = assets.update();
        let framebuffer_view: TextureView = (&self.framebuffer).into();
        let sample_count = self.framebuffer.sample_count();

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: match sample_count {
                        1 => view,
                        _ => &framebuffer_view,
                    },
                    resolve_target: match sample_count {
                        1 => None,
                        _ => Some(view),
                    },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.3, g: 0.7, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            if let Some(skybox) = &self.skybox {
                let shader = ShaderVariant::Double(
                    assets.try_get(VertexShader::ptr(skybox)).unwrap(),
                    assets.try_get(FragmentShader::ptr(skybox)).unwrap(),
                );

                let sky_config = RenderPipelineConfig::new(&shader, None::<&Vertices>, skybox, &[]);
                let sky_pipeline = self.pipelines.get_or_create(gpu, &sky_config);

                render_pass.set_pipeline(sky_pipeline);

                BindGroup::groups(skybox).iter().enumerate().for_each(|(i, group)| {
                    render_pass.set_bind_group(i as u32, group, &[]);
                });

                render_pass.draw(0..3, 0..1);
            }
        }
        {
            let egui_ctx = context.egui.egui_ctx();
            let output = egui_ctx.end_frame();
            let paint_jobs = egui_ctx.tessellate(output.shapes, egui_ctx.pixels_per_point());
            let texture_delta = output.textures_delta;

            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [context.surface_config.width, context.surface_config.height],
                pixels_per_point: window.scale_factor() as f32,
            };

            self.egui_renderer.update_buffers(
                &gpu.device,
                &gpu.queue,
                &mut (&mut encoder),
                &paint_jobs,
                &screen_descriptor,
            );

            for (id, delta) in texture_delta.set {
                self.egui_renderer.update_texture(&gpu.device, &gpu.queue, id, &delta);
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GUI RenderPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: match sample_count {
                            1 => view,
                            _ => &framebuffer_view,
                        },
                        resolve_target: match sample_count {
                            1 => None,
                            _ => Some(view),
                        },
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });
                self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
            }

            for id in texture_delta.free {
                self.egui_renderer.free_texture(&id);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}
