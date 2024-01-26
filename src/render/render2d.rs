use std::mem::size_of;

use glam::{Vec2, Vec3, Vec4};
use wgpu::TextureView;
use what::ShaderStages;

use crate::{
    assets::{
        assets::{AssetType, Assets, GenPtr, Ptr},
        buffer::{Indices, UniformBuffer, Vertices},
        shader::{Shader, ShaderVariant},
        texture::{Sampler, Texture2D},
    },
    context::{Context, VisContext},
    entity::{desc::Transform2D, entities::Worlds},
    event::{self, EventSubscriber},
    render::material::GenericMaterial,
    utils::Guid,
};

use super::types::{BindGroup, BindGroupEntry, BindLayout, FragmentShader, Vertex2D, VertexShader};
use super::{
    camera::CameraBuffer,
    factory::{BindGroupConfig, BindGroupFactory, PipelineFactory, RenderPipelineConfig},
    framebuffer::Framebuffer,
    mesh::GenericMesh,
    types::{IndexBuffer, VertexBuffer},
};

//Descriptors for this system.
pub struct Transform2DDesc {
    transform: Transform2D,
    uniform: Option<UniformBuffer>,
    dirty: bool,
}

impl Transform2DDesc {
    pub fn new(transform: Transform2D) -> Self {
        Self { transform, uniform: None, dirty: true }
    }

    pub fn set_transform(&mut self, transform: Transform2D) {
        if self.transform != transform {
            self.transform = transform;
            self.dirty = true;
        }
    }

    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    pub fn set_position(&mut self, position: Vec3) {
        if self.transform.position != position {
            self.transform.position = position;
            self.dirty = true;
        }
    }

    pub fn position(&self) -> &Vec3 {
        &self.transform.position
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        if self.transform.rotation != rotation {
            self.transform.rotation = rotation;
            self.dirty = true;
        }
    }

    pub fn rotation(&self) -> &f32 {
        &self.transform.rotation
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        if self.transform.scale != scale {
            self.transform.scale = scale;
            self.dirty = true;
        }
    }

    pub fn scale(&self) -> &Vec2 {
        &self.transform.scale
    }
}

pub struct SpriteDesc {
    texture: Ptr<Texture2D>,
    sampler: Ptr<Sampler>,
    tint: Vec4,
    buffer: Option<UniformBuffer>,
    material: Option<GenericMaterial>,
    dirty: bool,
}

impl SpriteDesc {
    pub fn new(texture: Ptr<Texture2D>, tint: Vec4, sampler: Option<Ptr<Sampler>>) -> Self {
        Self {
            texture,
            sampler: sampler.unwrap_or(Ptr::dead()),
            tint,
            buffer: None,
            material: None,
            dirty: true,
        }
    }

    pub fn set_texture(&mut self, texture: Ptr<Texture2D>) {
        if self.texture != texture {
            self.texture = texture;
            self.dirty = true;
        }
    }

    pub fn set_sampler(&mut self, sampler: Ptr<Sampler>) {
        if self.sampler != sampler {
            self.sampler = sampler;
            self.dirty = true;
        }
    }

    pub fn set_tint(&mut self, tint: Vec4) {
        if self.tint != tint {
            self.tint = tint;
            self.dirty = true;
        }
    }

    pub fn texture(&self) -> &Ptr<Texture2D> {
        &self.texture
    }

    pub fn sampler(&self) -> &Ptr<Sampler> {
        &self.sampler
    }

    pub fn tint(&self) -> &Vec4 {
        &self.tint
    }
}

//--------------------------------------------------------------------------------------------------

pub struct RenderData<'a> {
    pub ctx: &'a Context,
    pub view: &'a TextureView,
    pub window: &'a winit::window::Window,
}

pub struct Renderer2D {
    framebuffer: Framebuffer,
    pipelines: PipelineFactory,
    sprite_shader: Ptr<Shader>,
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
        let framebuffer = Framebuffer::new(context, sample_count);

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

        let sprite_shader = assets.consume_asset(
            AssetType::Shader(
                Shader::new(
                    &context.graphics,
                    Guid::dead(),
                    wgpu::ShaderSource::Wgsl(include_str!("../assets/sprite.wgsl").into()),
                    ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                )
                .unwrap(),
            ),
            None::<&str>,
        );

        let camera_buffer = Some(CameraBuffer::new(&context.graphics, "Default Camera"));

        Renderer2D { framebuffer, pipelines, sprite_shader, sprite_mesh, camera_buffer }
    }

    pub fn update_camera_buffer(&mut self, context: &VisContext, camera: [[f32; 4]; 4]) {
        if let Some(camera_buffer) = &mut self.camera_buffer {
            camera_buffer.update_buffer(context, camera);
        }
    }

    fn update_transform(context: &VisContext, desc: &mut Transform2DDesc) {
        let uniform = if let Some(uniform) = desc.uniform.as_mut() {
            uniform
        } else {
            desc.uniform = Some(UniformBuffer::new(context, size_of::<[[f32; 4]; 4]>()));
            desc.dirty = true;
            return;
        };

        if desc.dirty {
            let transform: [[f32; 4]; 4] = glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(desc.transform.scale.x, desc.transform.scale.y, 1.0),
                glam::Quat::from_rotation_z(desc.transform.rotation),
                glam::Vec3::new(desc.transform.position.x, desc.transform.position.y, 0.0),
            )
            .to_cols_array_2d();

            uniform.update_buffer(context, bytemuck::cast_slice(&transform));
            desc.dirty = false;
        }
    }

    fn update_material(
        &self, context: &VisContext, transform: &UniformBuffer, desc: &mut SpriteDesc,
        assets: &mut Assets,
    ) {
        if !assets.exist(&GenPtr::from(desc.sampler)) {
            desc.sampler =
                assets.consume_asset(AssetType::Sampler(Sampler::two_dim(context)), None::<&str>);
        }

        let texture = match assets.try_get(&desc.texture) {
            Some(texture) => texture,
            None => Texture2D::error_texture(context),
        };

        let sampler = assets.try_get(&desc.sampler);

        if desc.buffer.is_none() {
            desc.buffer = Some(UniformBuffer::new(context, size_of::<[f32; 4]>()));
            desc.dirty = true;
        }

        if desc.material.is_none() || desc.dirty {
            if let (Some(sampler), Some(buffer)) = (sampler, &mut desc.buffer) {
                buffer.update_buffer(context, bytemuck::cast_slice(&desc.tint.to_array()));

                desc.material = Some(GenericMaterial::new(
                    context,
                    self.sprite_shader,
                    self.sprite_shader,
                    &[
                        UniformBuffer::layout_entry(0),
                        UniformBuffer::layout_entry(1),
                        Texture2D::layout_entry(2),
                        Sampler::layout_entry(3),
                    ],
                    &[
                        transform.group_entry(0),
                        buffer.group_entry(1),
                        texture.group_entry(2),
                        sampler.group_entry(3),
                    ],
                ));
                desc.dirty = false;
            }
        }
    }

    pub fn render(&mut self, data: RenderData, assets: &mut Assets, worlds: &mut Worlds) {
        let context = data.ctx.graphics.as_ref();
        let fbo = &self.framebuffer;
        let fbo_view: TextureView = (&self.framebuffer).into();

        let _ = assets.update();

        if let Some(world) = worlds.get_mut() {
            let mut config_keys = Vec::new();

            for (_entity, (transform, sprite)) in
                world.query_mut::<(&mut Transform2DDesc, &mut SpriteDesc)>()
            {
                Self::update_transform(context, transform);
                self.update_material(context, transform.uniform.as_ref().unwrap(), sprite, assets);

                let material = sprite.material.as_ref().unwrap();
                let vertex = assets.try_get(VertexShader::ptr(material)).unwrap();
                let fragment = assets.try_get(FragmentShader::ptr(material)).unwrap();
                let shader = ShaderVariant::Double(vertex, fragment);

                let config = RenderPipelineConfig::new(
                    &shader,
                    Some(&self.sprite_mesh),
                    material,
                    &[CameraBuffer::layout(context)],
                );

                self.pipelines.prepare(context, &config);
                config_keys.push(config.key());
            }

            let mut encoder =
                context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Renderer2D Render Encoder"),
                });
            {
                let mut renderables = world.query::<(&Transform2DDesc, &SpriteDesc)>();

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
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.3,
                                g: 0.7,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    ..Default::default()
                });

                if let Some(camera) = &self.camera_buffer {
                    let mesh = &self.sprite_mesh;

                    for (i, renderable) in renderables.into_iter().enumerate() {
                        let (_transform, sprite) = renderable.1;

                        let material = sprite.material.as_ref().unwrap();

                        let pipeline = self
                            .pipelines
                            .get_key(unsafe { config_keys.get_unchecked(i) })
                            .unwrap();

                        render_pass.set_pipeline(pipeline);

                        //Set material
                        for (i, bind_group) in material.groups().iter().enumerate() {
                            render_pass.set_bind_group(i as u32, bind_group, &[]);
                        }

                        //Set camera buffer
                        render_pass.set_bind_group(1, camera.bind_group(), &[]);

                        //Set vertex buffer
                        render_pass
                            .set_vertex_buffer(0, VertexBuffer::buffer(mesh).unwrap().slice(..));

                        //Set index buffer
                        let (buffer, format) = IndexBuffer::buffer(mesh).unwrap();
                        render_pass.set_index_buffer(buffer.slice(..), format);

                        //Draw the quad.
                        render_pass.draw_indexed(0..mesh.num_indices(), 0, 0..1);
                    }
                }
            }
            context.queue.submit(std::iter::once(encoder.finish()));
        }
    }
}
