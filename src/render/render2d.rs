use glam::Vec4;
use wgpu::TextureView;
use winit::window::Window;

use crate::assets::assets::{Assets, BACKGROUND_SHADER, SPRITE_SHADER};
use crate::assets::buffer::{UniformBuffer, Vertices};
use crate::assets::shader::ShaderVariant;
use crate::assets::texture::{Sampler, Texture2D};
use crate::context::{Context, VisContext};
use crate::entities::animation2d::Animation2D;
use crate::entities::entities::Worlds;
use crate::entities::sprite::Sprite;
use crate::entities::transform2d::Transform2D;
use crate::event::{self, EventSubscriber};
use crate::render::renderer::Renderer;
use crate::utils::Timestep;

use super::camera::CameraBuffer;
use super::factory::{PipelineFactory, RenderPipelineConfig};
use super::framebuffer::Framebuffer;
use super::material::{Background2DMaterial, GenericMaterial};
use super::types::{BindGroup, FragmentShader, IndexBuffer, VertexBuffer, VertexShader};

pub struct Renderer2D {
    framebuffer: Framebuffer,
    pipelines: PipelineFactory,
    camera_buffer: Option<CameraBuffer>,
    egui_renderer: egui_wgpu::Renderer,
    background: Option<Background2DMaterial>,
}

impl EventSubscriber for Renderer2D {
    fn on_event(&mut self, event: &crate::event::Event, context: &mut Context) -> bool {
        match event {
            event::Event::Resized { width, height } => {
                self.framebuffer.resize(context, *width, *height);
                false
            }
            _ => false,
        }
    }
}

impl Renderer2D {
    pub fn new(context: &Context, _assets: &mut Assets) -> Self {
        //Renderable setup
        let sample_count = 4;
        let pipelines = PipelineFactory::new();
        let framebuffer = Framebuffer::new(context, sample_count, Some(16.0 / 9.0));
        let camera_buffer = Some(CameraBuffer::new(&context.graphics, "Default Camera"));
        let egui_renderer = Renderer::recreate_gui(context, sample_count);

        Renderer2D { framebuffer, pipelines, camera_buffer, egui_renderer, background: None }
    }

    pub fn set_background(&mut self, context: &VisContext, texture: &Texture2D, tint: Vec4) {
        match self.background {
            Some(ref mut background) => {
                background.update_texture(context, texture);
                background.update_tint(context, tint);
            }
            None => {
                let background = Background2DMaterial::new(context, texture, tint);
                self.background = Some(background);
            }
        }
    }

    pub fn update_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_buffer(context, camera);
        }
    }

    pub fn update_viewport(&mut self, viewport: (f32, f32, f32, f32)) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_viewport(viewport);
        }
    }

    pub fn update_animations(
        &mut self, context: &VisContext, delta: &Timestep, worlds: &mut Worlds,
    ) {
        if let Some(world) = worlds.get_mut() {
            for (_entity, (sprite, animation)) in
                world.query_mut::<(&mut Sprite, &mut Animation2D)>()
            {
                animation.update(context, delta, sprite);
            }
        }
    }

    pub fn render(
        &mut self, assets: &mut Assets, worlds: &mut Worlds, ctx: &mut Context, view: &TextureView,
        window: &Window,
    ) {
        let context = ctx.graphics.as_ref();
        let fbo = &self.framebuffer;
        let fbo_view: TextureView = (&self.framebuffer).into();
        let sample_count = fbo.sample_count();
        let _ = assets.update();

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D Render Encoder"),
        });

        if let Some(camera_buffer) = &self.camera_buffer {
            //Background render pass---------------------------------------------------------------------
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Background Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: match fbo.sample_count() {
                            1 => &view,
                            _ => &fbo_view,
                        },
                        resolve_target: match fbo.sample_count() {
                            1 => None,
                            _ => Some(&view),
                        },
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });

                let (x, y, w, h) = camera_buffer.viewport();
                render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

                if let Some(background) = &self.background {
                    let shader = ShaderVariant::Single(assets.try_get(&BACKGROUND_SHADER).unwrap());

                    let config =
                        RenderPipelineConfig::new(&shader, None::<&Vertices>, background, &[]);

                    let pipeline = self.pipelines.get_or_create(context, &config);

                    render_pass.set_pipeline(pipeline);

                    for (i, bind_group) in background.groups().iter().enumerate() {
                        render_pass.set_bind_group(i as u32, bind_group, &[]);
                    }

                    render_pass.draw(0..3, 0..1);
                }
            }

            //------------------------------------------------------------------------------------------
            //Prepare World Render Pass--------------------------------------------------------------------------
            if let Some(world) = worlds.get_mut() {
                let mut config_keys = Vec::new();

                //Iterate over all entities with a transform component but do not borrow.
                for (entity, _) in world.query::<()>().with::<&Transform2D>().iter() {
                    if let Ok(mut transform) = world.get::<&mut Transform2D>(entity) {
                        transform.update(context, entity, world);
                    }
                }

                for (_, (transform, sprite)) in
                    world.query::<(&mut Transform2D, &mut Sprite)>().iter()
                {
                    if let Some(texture) = assets.try_get(&sprite.texture()) {
                        sprite.update(context, texture);
                    }

                    let material = sprite.material();
                    let vertex = assets.try_get(VertexShader::ptr(material)).unwrap();
                    let fragment = assets.try_get(FragmentShader::ptr(material)).unwrap();
                    let shader = ShaderVariant::Double(vertex, fragment);

                    let config = RenderPipelineConfig::new(
                        &shader,
                        Some(sprite.mesh()),
                        material,
                        &[transform.layout(), CameraBuffer::layout(context)],
                    );

                    self.pipelines.prepare(context, &config);
                    config_keys.push(config.key());
                }

                {
                    let mut renderables = world.query::<(&Transform2D, &Sprite)>();
                    let mut entities: Vec<(hecs::Entity, (&Transform2D, &Sprite<'_>))> =
                        renderables.iter().collect();
                    entities.sort_by(|(_, (a, _)), (_, (b, _))| {
                        a.position()
                            .z
                            .partial_cmp(&b.position().z)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    //World Render Pass---------------------------------------------------------------------
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("World Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: match fbo.sample_count() {
                                1 => &view,
                                _ => &fbo_view,
                            },
                            resolve_target: match fbo.sample_count() {
                                1 => None,
                                _ => Some(&view),
                            },
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

                    //Set viewport
                    let (x, y, w, h) = camera_buffer.viewport();
                    render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

                    for (i, renderable) in entities.iter().enumerate() {
                        let (transform, sprite) = renderable.1;

                        let material = sprite.material();

                        let pipeline = self
                            .pipelines
                            .get_key(unsafe { config_keys.get_unchecked(i) })
                            .unwrap();

                        render_pass.set_pipeline(pipeline);

                        //Set material
                        for (i, bind_group) in material.groups().iter().enumerate() {
                            render_pass.set_bind_group(i as u32, bind_group, &[]);
                        }

                        //Set transform buffer
                        render_pass.set_bind_group(1, transform.group(), &[]);

                        //Set camera buffer
                        render_pass.set_bind_group(2, camera_buffer.bind_group(), &[]);

                        //Set vertex buffer
                        render_pass.set_vertex_buffer(
                            0,
                            VertexBuffer::buffer(sprite.mesh()).unwrap().slice(..),
                        );

                        //Set index buffer
                        let (buffer, format) = IndexBuffer::buffer(sprite.mesh()).unwrap();
                        render_pass.set_index_buffer(buffer.slice(..), format);

                        //Draw the quad.
                        render_pass.draw_indexed(0..sprite.mesh().num_indices(), 0, 0..1);
                    }
                }
                //------------------------------------------------------------------------------------------
            }
        }

        //------------------------------------------------------------------------------------------

        {
            let egui_ctx = ctx.egui.egui_ctx();
            let output = egui_ctx.end_frame();
            let paint_jobs = egui_ctx.tessellate(output.shapes, egui_ctx.pixels_per_point());
            let texture_delta = output.textures_delta;

            let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [ctx.surface_config.width, ctx.surface_config.height],
                pixels_per_point: window.scale_factor() as f32,
            };

            let device = &ctx.graphics.device;

            let queue = &ctx.graphics.queue;
            self.egui_renderer.update_buffers(
                device,
                queue,
                &mut encoder,
                &paint_jobs,
                &screen_descriptor,
            );

            for (id, delta) in texture_delta.set {
                self.egui_renderer.update_texture(device, queue, id, &delta);
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("GUI RenderPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: match sample_count {
                            1 => view,
                            _ => &fbo_view,
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
        context.queue.submit(std::iter::once(encoder.finish()));
    }
}
