use glam::Vec4;
use wgpu::TextureView;
use winit::window::Window;

use crate::assets::assets::Assets;
use crate::assets::buffer::Vertices;
use crate::assets::shader::ShaderVariant;
use crate::assets::texture::Texture2D;
use crate::context::{Context, VisContext};
use crate::entities::animation2d::Animation2D;
use crate::entities::omniverse::Omniverse;
use crate::entities::sprite::Sprite;
use crate::entities::transform::Transform;
use crate::event::{self, EventSubscriber};
use crate::render::renderer::Renderer;
use crate::static_asset;
use crate::utils::Timestep;

use super::camera::CameraBuffer;
use super::factory::{PipelineFactory, RenderPipelineConfig};
use super::framebuffer::Framebuffer;
use super::material::BackgroundMaterial;
use super::types::{BindGroup, IndexBuffer, VertexBuffer};
use super::utils::create_color_renderpass;

pub enum ImmType<'a> {
    /// A sprite to be rendered immediately, at some position at the screen.
    Sprite,
    /// A texture to be renderer immediately, at the whole screen.
    Background(&'a BackgroundMaterial),
    /// A text to be rendered immediately, at some position at the screen.
    Text,
}

pub struct Renderer2D {
    framebuffer: Framebuffer,
    pipelines: PipelineFactory,
    camera_buffer: Option<CameraBuffer>,
    egui_renderer: egui_wgpu::Renderer,
    background: Option<BackgroundMaterial>,
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

#[profiling::all_functions]
impl Renderer2D {
    pub fn new(context: &Context) -> Self {
        //Renderable setup
        let sample_count = 4;
        let pipelines = PipelineFactory::new();
        let framebuffer = Framebuffer::new(context, sample_count);
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
                let background = BackgroundMaterial::new(context, texture, tint);
                self.background = Some(background);
            }
        }
    }

    pub fn set_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_buffer(context, camera);
        }
    }

    pub fn set_viewport(&mut self, viewport: (f32, f32, f32, f32)) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_viewport(viewport);
        }
    }

    pub fn update(&mut self, context: &VisContext, delta: &Timestep, omni: &mut Omniverse) {
        if let Some(world) = omni.get_mut() {
            for (_entity, (sprite, animation)) in world.query::<(&mut Sprite, &mut Animation2D)>() {
                animation.update(context, delta, sprite);
            }
        }
    }

    /// Render an immediate object to the screen. Immediate in this context means not part of a scene. Or not part of a bigger state.
    /// Although this function caches the created pipeline in the pipeline factory and thus needs a mut ref.
    pub fn draw_imm(&mut self, ctx: &mut Context, view: &TextureView, obj: ImmType) {
        let context = ctx.graphics.as_ref();
        let fbo = &self.framebuffer;

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D: Immediate Draw Encoder"),
        });

        {
            let mut render_pass = create_color_renderpass(&mut encoder, view, fbo, true);

            match obj {
                ImmType::Background(material) => {
                    if let Some(shader) = static_asset!(BackgroundShader) {
                        let shader = ShaderVariant::Single(shader);

                        let config =
                            RenderPipelineConfig::new(&shader, None::<&Vertices>, material, &[]);

                        let pipeline = self.pipelines.get_or_create(context, &config);

                        render_pass.set_pipeline(pipeline);

                        for (i, bind_group) in material.groups().iter().enumerate() {
                            render_pass.set_bind_group(i as u32, bind_group, &[]);
                        }

                        render_pass.draw(0..3, 0..1);
                    }
                }
                _ => {
                    log::warn!("Immediate draw type not supported yet.");
                }
            }
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn draw_egui(&mut self, ctx: &mut Context, view: &TextureView, window: &Window) {
        let context = ctx.graphics.as_ref();
        let fbo = &self.framebuffer;

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D: GUI Draw Encoder"),
        });

        //------------------------------------------------------------------------------------------
        {
            let egui_ctx = ctx.egui.egui_ctx();
            let output = egui_ctx.end_frame();
            let paint_jobs = egui_ctx.tessellate(output.shapes, egui_ctx.pixels_per_point());
            let texture_delta = output.textures_delta;

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
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
                let mut render_pass = create_color_renderpass(&mut encoder, view, fbo, false);
                self.egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
            }

            for id in texture_delta.free {
                self.egui_renderer.free_texture(&id);
            }
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn draw(
        &mut self, assets: &mut Assets, worlds: &mut Worlds, ctx: &mut Context, view: &TextureView,
        window: &Window,
    ) {
        let context = ctx.graphics.as_ref();
        let fbo = &self.framebuffer;
        let _ = assets.update();

        let mut encoder = context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D Render Encoder"),
        });

        if let Some(camera_buffer) = &self.camera_buffer {
            //Background render pass---------------------------------------------------------------------
            {
                let mut render_pass = create_color_renderpass(&mut encoder, view, fbo, true);

                let (x, y, w, h) = camera_buffer.viewport();
                render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

                if let (Some(bg), Some(shader)) =
                    (&self.background, static_asset!(BackgroundShader))
                {
                    let shader: ShaderVariant = shader.into();
                    let config = RenderPipelineConfig::new(&shader, None::<&Vertices>, bg, &[]);

                    let pipeline = self.pipelines.get_or_create(context, &config);

                    render_pass.set_pipeline(pipeline);

                    for (i, bind_group) in bg.groups().iter().enumerate() {
                        render_pass.set_bind_group(i as u32, bind_group, &[]);
                    }

                    render_pass.draw(0..3, 0..1);
                }
            }

            //------------------------------------------------------------------------------------------
            //Prepare World Render Pass--------------------------------------------------------------------------
            if let Some(world) = worlds.get_mut() {
                {
                    let mut renderables = world.query::<(&mut Transform, &mut Sprite)>();

                    entities.sort_by(|(_, (a, _)), (_, (b, _))| {
                        a.position().z.total_cmp(&b.position().z)
                    });

                    //World Render Pass---------------------------------------------------------------------
                    let mut render_pass = create_color_renderpass(&mut encoder, view, fbo, false);

                    //Set viewport
                    let (x, y, w, h) = camera_buffer.viewport();
                    render_pass.set_viewport(x, y, w, h, 0.0, 1.0);

                    for (entity, renderable) in entities.iter_mut() {
                        let (transform, sprite) = renderable;

                        //Update components
                        (*transform).update(context, *entity, world);

                        if let Ok(texture) = assets.try_get(sprite.texture()) {
                            sprite.update(context, texture);
                        }

                        let material = sprite.material();

                        if let Ok(shader) = ShaderVariant::from_material(material, assets) {
                            let config = RenderPipelineConfig::new(
                                &shader,
                                Some(sprite.mesh()),
                                material,
                                &[transform.layout(), CameraBuffer::layout(context)],
                            );

                            let pipeline = self.pipelines.get_or_create(context, &config);

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
                }
                //------------------------------------------------------------------------------------------
            }
        }

        context.queue.submit(std::iter::once(encoder.finish()));

        self.draw_egui(ctx, view, window);
    }
}
