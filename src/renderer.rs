use wgpu::{TextureView, RenderPipeline};
use winit::event::{VirtualKeyCode, ElementState};

use crate::{buffer::Framebuffer, context::Context, event::{EventSubscriber, self}};

pub struct Renderer2D {
    framebuffer: Framebuffer,
    render_pipeline: wgpu::RenderPipeline,
}

impl EventSubscriber for Renderer2D {

    fn on_event(&mut self, event: &crate::event::Event, context: &Context) -> bool
    {
        match event {
            event::Event::Resized {width, height} => {
                self.framebuffer.resize(context, *width, *height);
                return false;
            },
            event::Event::KeyboardInput { keycode, state } => {
                match keycode {
                    VirtualKeyCode::Left => {
                        if *state == ElementState::Pressed {
                            self.enable_msaa(context, self.framebuffer.sample_count() * 2);
                        }
                        return false;
                    },
                    VirtualKeyCode::Right => {
                        if *state == ElementState::Pressed {
                            let new_count = self.framebuffer.sample_count() / 2;

                            if new_count != 0 {
                                self.enable_msaa(context, new_count);
                            }
                        }
                        return false;
                    },
                    _ => {false}
                }
            }
            _ => {false}
        }
    }
}

impl Renderer2D {
    pub fn new(context: &Context) -> Self
    {
        let framebuffer = Framebuffer::new(context, 4);
        let render_pipeline = Renderer2D::recreate_pipeline(context, framebuffer.sample_count());

        Renderer2D { framebuffer: framebuffer, render_pipeline }
    }

    fn recreate_pipeline(context: &Context, sample_count: u32) -> RenderPipeline
    {
        let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor
        { 
            label: Some("Default Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("default.wgsl").into()),
        });
            
        let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
        {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[], 
            push_constant_ranges: &[],
        });

       context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor 
        {
            label: Some("Pipeline"), 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
            self.render_pipeline = Renderer2D::recreate_pipeline(context, sample_count);
            return true;
        }
        return false;
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
            render_pass.draw(0..3, 0..1);
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }
}