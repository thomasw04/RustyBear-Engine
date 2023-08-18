use wgpu::{TextureView, RenderPipeline, util::DeviceExt, BindGroupLayout};
use winit::event::{VirtualKeyCode, ElementState};

use crate::{context::Context, event::{EventSubscriber, self}, render::texture::Texture2D};

use super::{framebuffer::Framebuffer, camera::CameraBuffer};
use super::material::Material;
use super::types::Vertex2D;

pub struct Renderer {
    framebuffer: Framebuffer,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    material: Material,
    camera_buffer: CameraBuffer,
    indices: u32,
}

impl EventSubscriber for Renderer {

    fn on_event(&mut self, event: &crate::event::Event, context: &Context) -> bool
    {
        match event {
            event::Event::Resized {width, height} => {
                self.framebuffer.resize(context, *width, *height);
                false
            },
            event::Event::KeyboardInput { keycode, state } => {
                match keycode {
                    VirtualKeyCode::Left => {
                        if *state == ElementState::Pressed {
                            self.enable_msaa(context, self.framebuffer.sample_count() * 2);
                        }
                        false
                    },
                    VirtualKeyCode::Right => {
                        if *state == ElementState::Pressed {
                            let new_count = self.framebuffer.sample_count() / 2;

                            if new_count != 0 {
                                self.enable_msaa(context, new_count);
                            }
                        }
                        false
                    },
                    _ => {false}
                }
            }
            _ => {false}
        }
    }
}

impl Renderer {
    pub fn new(context: &Context) -> Self
    {
        //Renderable setup
        let framebuffer = Framebuffer::new(context, 4);

        let texture = Texture2D::error_texture(context);
        let material = Material::new(context, vec![texture.view()], texture.sampler(), "Quad");

        let camera_buffer = CameraBuffer::new(context, "Default Camera");

        //Pipeline creation

        let render_pipeline = Renderer::recreate_pipeline(context, framebuffer.sample_count(), vec![material.layout(), camera_buffer.layout()]);

        const VERTICES: &[Vertex2D] = &[
            Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [0.0, 1.0] },
            Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [1.0, 0.0] },
            Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [0.0, 0.0] },
            Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [1.0, 1.0] },
        ];

        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 3, 1,
        ];

        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Default VertexBuffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX, 
            }
        );

        let index_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Default IndexBuffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        

        Renderer { framebuffer, render_pipeline, vertex_buffer, index_buffer, material, camera_buffer, indices: INDICES.len() as u32}
    }

    fn recreate_pipeline(context: &Context, sample_count: u32, bind_group_layouts: Vec<&BindGroupLayout>) -> RenderPipeline
    {
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor
        { 
            label: Some("Default Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("default.wgsl").into()),
        });
            
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts, 
            push_constant_ranges: &[],
        });

        context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor 
        {
            label: Some("Pipeline"), 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex2D::LAYOUT,
                ],
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
                    format: context.config.format, 
                    blend: Some(wgpu::BlendState::REPLACE), 
                    write_mask: wgpu::ColorWrites::ALL 
                })], 
            }), 
            multiview: None, 
        })
    }

    pub fn enable_msaa(&mut self, context: &Context, sample_count: u32) -> bool
    {
        if self.framebuffer.change_sample_count(context, sample_count) {
            self.render_pipeline = Renderer::recreate_pipeline(context, sample_count, vec![self.material.layout()]);
            return true;
        }
        false
    }

    pub fn update_camera_buffer(&mut self, context: &Context, camera: [[f32; 4]; 4])
    {
        self.camera_buffer.update_buffer(context, camera);
    }

    pub fn render(&mut self, context: &mut Context, view: TextureView)
    {
        let framebuffer_view: TextureView = (&self.framebuffer).into();
        let sample_count = self.framebuffer.sample_count();

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: match sample_count
                    {
                        1 => &view,
                        _ => &framebuffer_view
                    },
                    resolve_target: match sample_count
                    {
                        1 => None,
                        _ => Some(&view)
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.material.bind_group(), &[]);
            render_pass.set_bind_group(1, &self.camera_buffer.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.indices, 0, 0..1);
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }
}