use wgpu::TextureView;
use winit::{event::ElementState, keyboard::KeyCode};

use crate::{
    assets::{
        assets::Assets,
        buffer::{Indices, Vertices},
        shader::ShaderVariant,
        texture::{Sampler, Texture2D},
    },
    context::{Context, VisContext},
    event::{self, EventSubscriber},
    render::{material::GenericMaterial, types::BindGroupEntry},
};

use super::{
    camera::CameraBuffer,
    factory::PipelineFactory,
    framebuffer::Framebuffer,
    mesh::GenericMesh,
    types::{BindGroup, IndexBuffer, VertexBuffer},
};
use super::{material::SkyboxMaterial, types::Vertex2D};

pub struct Renderer<'a> {
    framebuffer: Framebuffer,
    assets: Assets,
    pipelines: PipelineFactory,
    material: GenericMaterial,
    mesh: GenericMesh<'a>,
    camera_buffer: CameraBuffer,
    skybox: Option<SkyboxMaterial>,
    egui_render_pass: egui_wgpu_backend::RenderPass,
}

impl<'a> EventSubscriber for Renderer<'a> {
    fn on_event(&mut self, event: &crate::event::Event, context: &mut Context) -> bool {
        match event {
            event::Event::Resized { width, height } => {
                self.framebuffer.resize(context, *width, *height);
                false
            }
            event::Event::KeyboardInput { keycode, state } => match keycode {
                KeyCode::ArrowLeft => {
                    if *state == ElementState::Pressed {
                        self.enable_msaa(context, self.framebuffer.sample_count() * 2);
                    }
                    false
                }
                KeyCode::ArrowRight => {
                    if *state == ElementState::Pressed {
                        let new_count = self.framebuffer.sample_count() / 2;

                        if new_count != 0 {
                            self.enable_msaa(context, new_count);
                        }
                    }
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }
}

impl<'a> Renderer<'a> {
    pub fn new(context: &Context, mut assets: Assets) -> Self {
        //Renderable setup
        let sample_count = 4;

        let pipelines = PipelineFactory::new();

        let framebuffer = Framebuffer::new(context, sample_count);

        let texture = Texture2D::error_texture(&context.graphics);
        let sampler = Sampler::two_dim(&context.graphics);

        //TODO add capability to what for material asset type containing textures, shaders, etc.
        let sky_tex = assets.request_asset("data/skybox.fur", 0);
        let sky_shader = assets.request_asset("static:skybox.wgsl", 0);

        let skybox = assets.get(&sky_tex).map(|sky_tex| {
            SkyboxMaterial::new(&context.graphics, ShaderVariant::Single(sky_shader), sky_tex)
        });

        let camera_buffer = CameraBuffer::new(&context.graphics, "Default Camera");

        let egui_render_pass = Renderer::recreate_gui(context, 1);

        let material = GenericMaterial::new(
            &context.graphics,
            ShaderVariant::Single(assets.request_asset("static:default.wgsl", 0)),
            &[texture.layout_entry(0), sampler.layout_entry(1)],
            &[texture.group_entry(0), sampler.group_entry(1)],
        );

        const VERTICES: &[Vertex2D] = &[
            Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [0.0, 1.0] },
            Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [1.0, 0.0] },
            Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [0.0, 0.0] },
            Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [1.0, 1.0] },
        ];

        const INDICES: &[u16] = &[0, 1, 2, 0, 3, 1];

        let vertices =
            Vertices::new(&context.graphics, bytemuck::cast_slice(VERTICES), Vertex2D::LAYOUT);

        let indices = Indices::new(
            &context.graphics,
            bytemuck::cast_slice(INDICES),
            wgpu::IndexFormat::Uint16,
        );

        let mesh = GenericMesh::new(vertices, indices, 6);

        Renderer {
            framebuffer,
            assets,
            pipelines,
            material,
            mesh,
            camera_buffer,
            skybox,
            egui_render_pass,
        }
    }

    fn recreate_gui(context: &Context, sample_count: u32) -> egui_wgpu_backend::RenderPass {
        egui_wgpu_backend::RenderPass::new(
            &context.graphics.device,
            context.surface_config.format,
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
        let _ = self.assets.update();
        let framebuffer_view: TextureView = (&self.framebuffer).into();
        let sample_count = self.framebuffer.sample_count();

        let mut encoder =
            context.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let output = context.egui.end_frame(Some(window));
        let paint_jobs = context
            .egui
            .context()
            .tessellate(output.shapes, context.egui.context().pixels_per_point());
        let texture_delta = output.textures_delta;

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
                let pipeline = self.pipelines.for_object(
                    &context.graphics,
                    &mut self.assets,
                    skybox,
                    None::<&GenericMesh>,
                    None,
                    None,
                    true,
                );

                render_pass.set_pipeline(pipeline);

                BindGroup::groups(skybox).iter().enumerate().for_each(|(i, group)| {
                    render_pass.set_bind_group(i as u32, group, &[]);
                });

                render_pass.draw(0..3, 0..1);
            }
        }

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
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            let pipeline = self.pipelines.for_object(
                &context.graphics,
                &mut self.assets,
                &self.material,
                Some(&self.mesh),
                None,
                Some(self.camera_buffer.layout()),
                true,
            );

            render_pass.set_pipeline(pipeline);

            BindGroup::groups(&self.material).iter().enumerate().for_each(|(i, group)| {
                render_pass.set_bind_group(i as u32, group, &[]);
            });

            render_pass.set_bind_group(1, self.camera_buffer.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, VertexBuffer::buffer(&self.mesh).unwrap().slice(..));
            let (buffer, format) = IndexBuffer::buffer(&self.mesh).unwrap();
            render_pass.set_index_buffer(buffer.slice(..), format);
            render_pass.draw_indexed(0..self.mesh.num_indices(), 0, 0..1);
        }

        {
            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: context.surface_config.width,
                physical_height: context.surface_config.height,
                scale_factor: window.scale_factor() as f32,
            };

            self.egui_render_pass
                .add_textures(&context.graphics.device, &context.graphics.queue, &texture_delta)
                .expect("[EGUI] Failed to add textures.");

            self.egui_render_pass.update_buffers(
                &context.graphics.device,
                &context.graphics.queue,
                &paint_jobs,
                &screen_descriptor,
            );

            self.egui_render_pass
                .execute(&mut encoder, view, &paint_jobs, &screen_descriptor, None)
                .unwrap();
        }

        context.graphics.queue.submit(std::iter::once(encoder.finish()));
    }
}
