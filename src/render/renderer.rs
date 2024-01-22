use wgpu::TextureView;
use winit::{event::ElementState, keyboard::KeyCode};

use crate::{
    assets::{
        assets::{AssetType, Assets, Ptr},
        buffer::{Indices, UniformBuffer, Vertices},
        shader::{Shader, ShaderVariant},
        texture::{Sampler, Texture2D},
    },
    context::{Context, VisContext},
    entity::{
        components::{Sprite, Transformation},
        entities::Worlds,
    },
    event::{self, EventSubscriber},
    render::{material::GenericMaterial, types::BindGroupEntry},
    utils::Guid,
};

use super::{
    camera::CameraBuffer,
    factory::{BindGroupConfig, BindGroupFactory, PipelineFactory, RenderPipelineConfig},
    framebuffer::Framebuffer,
    material::GenericMaterialLayout,
    mesh::GenericMesh,
    types::{BindGroup, FragmentShader, IndexBuffer, VertexBuffer, VertexShader},
};
use super::{material::SkyboxMaterial, types::Vertex2D};

pub struct RenderData<'a> {
    pub ctx: &'a Context,
    pub view: &'a TextureView,
    pub window: &'a winit::window::Window,
}

pub struct Renderer2D {
    framebuffer: Framebuffer,
    pipelines: PipelineFactory,
    bind_groups: BindGroupFactory,
    sprite_shader: Ptr<Shader>,
    sprite_sampler: Ptr<Sampler>,
    sprite_layout: GenericMaterialLayout,
    sprite_mesh: GenericMesh<'static>,
    camera_buffer: Option<CameraBuffer>,
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
    pub fn new(context: &Context, assets: &mut Assets) -> Self {
        //Renderable setup
        let sample_count = 4;
        let pipelines = PipelineFactory::new();
        let bind_groups = BindGroupFactory::new();
        let framebuffer = Framebuffer::new(context, sample_count);

        let sprite_shader = assets.consume_asset(
            AssetType::Shader(
                Shader::new(
                    &context.graphics,
                    Guid::dead(),
                    wgpu::ShaderSource::Wgsl(include_str!("../assets/sprite.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            ),
            "static:sprite.wgsl",
        );

        let sprite_sampler = assets.consume_asset(
            AssetType::Sampler(Sampler::two_dim(&context.graphics)),
            "static:default.sampler",
        );

        let sprite_layout = GenericMaterialLayout::new(
            &context.graphics,
            sprite_shader,
            sprite_shader,
            &[UniformBuffer::layout_entry(0), Texture2D::layout_entry(1), Sampler::layout_entry(2)],
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

        let sprite_mesh = GenericMesh::new(vertices, indices, 6);

        let camera_buffer = Some(CameraBuffer::new(&context.graphics, "Default Camera"));

        Renderer2D {
            framebuffer,
            pipelines,
            bind_groups,
            sprite_shader,
            sprite_sampler,
            sprite_layout,
            sprite_mesh,
            camera_buffer,
        }
    }

    pub fn update_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_buffer(context, camera);
        }
    }

    pub fn render<'a>(&mut self, data: RenderData<'a>, assets: &mut Assets, worlds: &mut Worlds) {
        let gpu = data.ctx.graphics.as_ref();
        let fbo = &self.framebuffer;
        let fbo_view: TextureView = (&self.framebuffer).into();

        let _ = assets.update();

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Renderer2D Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("World Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: match fbo.sample_count() {
                        1 => data.view,
                        _ => &fbo_view,
                    },
                    resolve_target: match fbo.sample_count() {
                        1 => None,
                        _ => Some(data.view),
                    },
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.3, g: 0.7, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            if let Some(world) = worlds.get() {
                if let Some(camera) = &self.camera_buffer {
                    let camera_layout = CameraBuffer::layout(gpu);
                    let mesh = &self.sprite_mesh;

                    let mut renderables = world.query::<(&Transformation, &Sprite)>();

                    //Create everything necessary to render the quad.
                    for (_entity, (_transform, sprite)) in renderables.iter() {
                        self.bind_groups.prepare(
                            gpu,
                            assets,
                            &BindGroupConfig::new(&[
                                sprite.texture.into(),
                                self.sprite_sampler.into(),
                            ]),
                        );
                    }

                    //Now only request things.
                    let sprite_shader =
                        ShaderVariant::Single(assets.try_get(&self.sprite_shader).unwrap());

                    //Pipline config for our quad.
                    let config = RenderPipelineConfig::new(
                        &sprite_shader,
                        Some(&self.sprite_mesh),
                        &self.sprite_layout,
                        &[camera_layout],
                    );

                    //Create/Get pipeline for our quad.
                    let pipeline = self.pipelines.get_or_create(gpu, &config);

                    for renderable in renderables.iter() {
                        let (_transform, sprite) = renderable.1;

                        //Get bind group for the texture
                        if let Some(bind_group) =
                            self.bind_groups.try_get(&BindGroupConfig::new(&[
                                sprite.texture.into(),
                                self.sprite_sampler.into(),
                            ]))
                        {
                            render_pass.set_pipeline(pipeline);

                            //Set material
                            render_pass.set_bind_group(0, bind_group, &[]);

                            //Set camera buffer
                            render_pass.set_bind_group(1, camera.bind_group(), &[]);

                            //Set vertex buffer
                            render_pass.set_vertex_buffer(
                                0,
                                VertexBuffer::buffer(mesh).unwrap().slice(..),
                            );

                            //Set index buffer
                            let (buffer, format) = IndexBuffer::buffer(mesh).unwrap();
                            render_pass.set_index_buffer(buffer.slice(..), format);

                            //Draw the quad.
                            render_pass.draw_indexed(0..mesh.num_indices(), 0, 0..1);
                        }
                    }
                }
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}

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

        let default_shader = assets.consume_asset(
            AssetType::Shader(
                Shader::new(
                    &context.graphics,
                    Guid::dead(),
                    wgpu::ShaderSource::Wgsl(include_str!("../assets/sprite.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            ),
            "static:sprite.wgsl",
        );

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
            "static:skybox.wgsl",
        );

        let texture = Texture2D::error_texture(&context.graphics);
        let sampler = Sampler::two_dim(&context.graphics);

        //TODO add capability to what for material asset type containing textures, shaders, etc.
        let sky_tex = assets.request_asset("data/skybox.fur", 0);

        let skybox = assets
            .get(&sky_tex)
            .map(|sky_tex| SkyboxMaterial::new(&context.graphics, sky_shader, sky_shader, sky_tex));

        let camera_buffer = CameraBuffer::new(&context.graphics, "Default Camera");

        let egui_render_pass = Renderer::recreate_gui(context, 1);

        let material = GenericMaterial::new(
            &context.graphics,
            default_shader,
            default_shader,
            &[Texture2D::layout_entry(0), Sampler::layout_entry(1)],
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
        let gpu = context.graphics.as_ref();
        let assets = &mut self.assets;

        let _ = assets.update();
        let framebuffer_view: TextureView = (&self.framebuffer).into();
        let sample_count = self.framebuffer.sample_count();

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                let shader = ShaderVariant::Double(
                    assets.try_get(VertexShader::ptr(skybox)).unwrap(),
                    assets.try_get(FragmentShader::ptr(skybox)).unwrap(),
                );

                let sky_config = RenderPipelineConfig::new(&shader, None::<&Vertices>, skybox, &[]);
                let sky_pipeline = self.pipelines.get_or_create(&gpu, &sky_config);

                render_pass.set_pipeline(sky_pipeline);

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

            let shader = ShaderVariant::Double(
                assets.try_get(VertexShader::ptr(&self.material)).unwrap(),
                assets.try_get(FragmentShader::ptr(&self.material)).unwrap(),
            );

            let config = RenderPipelineConfig::new(
                &shader,
                Some(&self.mesh),
                &self.material,
                &[CameraBuffer::layout(gpu)],
            );

            let pipeline = self.pipelines.get_or_create(gpu, &config);

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
                .add_textures(&gpu.device, &gpu.queue, &texture_delta)
                .expect("[EGUI] Failed to add textures.");

            self.egui_render_pass.update_buffers(
                &gpu.device,
                &gpu.queue,
                &paint_jobs,
                &screen_descriptor,
            );

            self.egui_render_pass
                .execute(&mut encoder, view, &paint_jobs, &screen_descriptor, None)
                .unwrap();
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
    }
}
