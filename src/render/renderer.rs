use wgpu::{util::DeviceExt, BindGroupLayout, RenderPipeline, TextureView};
use winit::event::{ElementState, VirtualKeyCode};

use crate::{
    assets::manager::{AssetManager, AssetType},
    context::{Context, VisContext},
    event::{self, EventSubscriber},
    render::{material::Skybox, texture::Texture2D},
};

use super::material::Material;
use super::types::Vertex2D;
use super::{camera::CameraBuffer, framebuffer::Framebuffer};

pub struct Renderer {
    framebuffer: Framebuffer,
    render_pipeline: wgpu::RenderPipeline,
    skybox_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: Material,
    camera_buffer: CameraBuffer,
    indices: u32,
    skybox: Skybox,
    egui_render_pass: egui_wgpu_backend::RenderPass,
}

impl EventSubscriber for Renderer {
    fn on_event(&mut self, event: &crate::event::Event, context: &mut Context) -> bool {
        match event {
            event::Event::Resized { width, height } => {
                self.framebuffer.resize(context, *width, *height);
                false
            }
            event::Event::KeyboardInput { keycode, state } => match keycode {
                VirtualKeyCode::Left => {
                    if *state == ElementState::Pressed {
                        self.enable_msaa(context, self.framebuffer.sample_count() * 2);
                    }
                    false
                }
                VirtualKeyCode::Right => {
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

impl Renderer {
    pub fn new(context: &Context, asset_manager: &mut AssetManager) -> Self {
        //Renderable setup
        let sample_count = 4;

        let framebuffer = Framebuffer::new(context, sample_count);

        let texture = Texture2D::error_texture(&context.graphics);

        let material = Material::new(
            &context.graphics,
            vec![texture.view()],
            texture.sampler(),
            "Quad",
        );

        let skybox_guid = asset_manager.request_id("data/skybox.fur");
        let skybox_texture = asset_manager.get_asset(skybox_guid, 0);

        let skybox = if let AssetType::TextureArray(skybox_texture) = skybox_texture {
            Skybox::new(
                &context.graphics,
                &skybox_texture.texture_view(),
                skybox_texture.sampler(),
                "Skybox",
            )
        } else {
            panic!("Failed to load skybox.")
        };

        let skybox_shader =
            context
                .graphics
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Default Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
                });

        let skybox_pipeline = Skybox::create_pipeline(&context.graphics, &skybox_shader);

        let camera_buffer = CameraBuffer::new(&context.graphics, "Default Camera");

        let egui_render_pass = Renderer::recreate_gui(context, 1);

        //Pipeline creation

        let render_pipeline = Renderer::recreate_pipeline(
            context,
            framebuffer.sample_count(),
            vec![material.layout(), camera_buffer.layout()],
        );

        const VERTICES: &[Vertex2D] = &[
            Vertex2D {
                position: [-1.0, -1.0, -0.0],
                texture_coords: [0.0, 1.0],
            },
            Vertex2D {
                position: [1.0, 1.0, -0.0],
                texture_coords: [1.0, 0.0],
            },
            Vertex2D {
                position: [-1.0, 1.0, -0.0],
                texture_coords: [0.0, 0.0],
            },
            Vertex2D {
                position: [1.0, -1.0, -0.0],
                texture_coords: [1.0, 1.0],
            },
        ];

        const INDICES: &[u16] = &[0, 1, 2, 0, 3, 1];

        let vertex_buffer =
            context
                .graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Default VertexBuffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let index_buffer =
            context
                .graphics
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Default IndexBuffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        Renderer {
            framebuffer,
            render_pipeline,
            skybox_pipeline,
            vertex_buffer,
            index_buffer,
            material,
            camera_buffer,
            indices: INDICES.len() as u32,
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

    fn recreate_pipeline(
        context: &Context,
        sample_count: u32,
        bind_group_layouts: Vec<&BindGroupLayout>,
    ) -> RenderPipeline {
        let shader = context
            .graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Default Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("default.wgsl").into()),
            });

        let pipeline_layout =
            context
                .graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        context
            .graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex2D::LAYOUT],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: sample_count,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
    }

    pub fn enable_msaa(&mut self, context: &mut Context, sample_count: u32) -> bool {
        if self.framebuffer.change_sample_count(context, sample_count) {
            self.render_pipeline = Renderer::recreate_pipeline(
                context,
                sample_count,
                vec![self.material.layout(), self.camera_buffer.layout()],
            );

            // self.egui_render_pass = Renderer::recreate_gui(context, sample_count);

            return true;
        }
        false
    }

    pub fn update_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        self.camera_buffer.update_buffer(context, camera);
    }

    pub fn update_skybox_buffer(
        &mut self,
        context: &VisContext,
        view: [[f32; 4]; 4],
        projection: [[f32; 4]; 4],
    ) {
        self.skybox.update_buffer(context, view, projection);
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        view: &TextureView,
        window: &winit::window::Window,
        _asset_manager: &AssetManager,
    ) {
        let framebuffer_view: TextureView = (&self.framebuffer).into();
        let sample_count = self.framebuffer.sample_count();

        let mut encoder =
            context
                .graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        let output = context.egui.end_frame(Some(window));
        let paint_jobs = context.egui.context().tessellate(output.shapes);
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
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.7,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.skybox_pipeline);
            render_pass.set_bind_group(0, self.skybox.bind_group(), &[]);
            render_pass.draw(0..3, 0..1);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.material.bind_group(), &[]);
            render_pass.set_bind_group(1, self.camera_buffer.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices, 0, 0..1);
        }

        {
            let screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
                physical_width: context.surface_config.width,
                physical_height: context.surface_config.height,
                scale_factor: window.scale_factor() as f32,
            };

            self.egui_render_pass
                .add_textures(
                    &context.graphics.device,
                    &context.graphics.queue,
                    &texture_delta,
                )
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

        context
            .graphics
            .queue
            .submit(std::iter::once(encoder.finish()));
    }
}
