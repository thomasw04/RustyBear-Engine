use glam::{Mat4, Vec2, Vec3, Vec4};
use std::mem::size_of;

use crate::assets::assets::{Ptr, SPRITE_SHADER};
use crate::assets::buffer::{Indices, UniformBuffer, Vertices};
use crate::assets::shader::Shader;
use crate::assets::texture::{Sampler, Texture2D};
use crate::context::VisContext;
use crate::hecs_hierarchy::Hierarchy;
use crate::render::material::GenericMaterial;
use crate::render::mesh::GenericMesh;
use crate::render::types::{BindGroupEntry, Vertex2D};
use crate::utils::Timestep;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

#[derive(Debug)]
pub struct Transform2D {
    position: Vec3,
    rotation: f32,
    scale: Vec2,
    parent: Mat4,
    global: Mat4,
    uniform: UniformBuffer,
    group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
    dirty: bool,
}

impl Transform2D {
    pub fn new(context: &VisContext, position: Vec3, rotation: f32, scale: Vec2) -> Self {
        let mut uniform = UniformBuffer::new(context, size_of::<[[f32; 4]; 4]>());

        let global = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(scale.x, scale.y, 1.0),
            glam::Quat::from_rotation_z(rotation),
            glam::Vec3::new(position.x, position.y, 0.0),
        );

        let parent = Mat4::IDENTITY;

        uniform.update_buffer(context, bytemuck::cast_slice(&global.to_cols_array_2d()));

        let layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[UniformBuffer::layout_entry(0)],
        });

        let group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[uniform.group_entry(0)],
        });

        Self { position, rotation, scale, parent, global, uniform, group, layout, dirty: true }
    }

    pub fn update(&mut self, context: &VisContext, entity: hecs::Entity, world: &hecs::World) {
        self.parent = if let Ok(parent) = world.parent::<Transform2D>(entity) {
            world.get::<&Transform2D>(parent).unwrap().global
        } else {
            Mat4::IDENTITY
        };

        self.update_desc(context, entity, world);
    }

    fn update_desc(&mut self, context: &VisContext, entity: hecs::Entity, world: &hecs::World) {
        if self.dirty {
            //Get local transform
            let local = glam::Mat4::from_scale_rotation_translation(
                glam::Vec3::new(self.scale.x, self.scale.y, 1.0),
                glam::Quat::from_rotation_z(self.rotation),
                glam::Vec3::new(self.position.x, self.position.y, 0.0),
            );

            //Calculate global transform
            self.global = self.parent * local;

            //Propagate to descendants
            for child in world.children::<Transform2D>(entity) {
                if let Ok(mut transform) = world.get::<&mut Transform2D>(child) {
                    transform.dirty = true;
                    transform.parent = self.global;
                    transform.update_desc(context, child, world);
                }
            }

            self.uniform
                .update_buffer(context, bytemuck::cast_slice(&self.global.to_cols_array_2d()));
            self.dirty = false;
        }
    }

    pub fn group(&self) -> &wgpu::BindGroup {
        &self.group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
        self.dirty = true;
    }

    pub fn add_pos(&mut self, inc: Vec3) {
        self.position += inc;
        self.dirty = true;
    }

    pub fn add_rot(&mut self, inc: f32) {
        self.rotation += inc;
        self.dirty = true;
    }

    pub fn add_scale(&mut self, inc: Vec2) {
        self.scale += inc;
        self.dirty = true;
    }
}

pub struct Sprite<'a> {
    texture: Ptr<Texture2D>,
    tint: Vec4,
    sampler: Sampler,
    buffer: UniformBuffer,
    material: GenericMaterial,
    mesh: GenericMesh<'a>,
    waiting: bool,
}

impl<'a> Sprite<'a> {
    pub fn new_custom(
        context: &VisContext, vertex: Ptr<Shader>, fragment: Ptr<Shader>, texture: Ptr<Texture2D>,
        tint: Vec4, coords: Option<&[f32]>, sampler: Option<Sampler>,
    ) -> Self {
        let mut buffer = UniformBuffer::new(context, size_of::<[f32; 4]>());
        buffer.update_buffer(context, bytemuck::cast_slice(&tint.to_array()));
        let sampler = sampler.unwrap_or(Sampler::two_dim(context));

        let vertices = if let Some(coords) = coords {
            vec![
                Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [coords[0], coords[1]] },
                Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [coords[2], coords[3]] },
                Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [coords[4], coords[5]] },
                Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [coords[6], coords[7]] },
            ]
        } else {
            vec![
                Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [0.0, 1.0] },
                Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [1.0, 0.0] },
                Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [0.0, 0.0] },
                Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [1.0, 1.0] },
            ]
        };

        const INDICES: &[u16] = &[0, 1, 2, 0, 3, 1];
        let vertices = Vertices::new(&context, bytemuck::cast_slice(&vertices), Vertex2D::LAYOUT);
        let indices =
            Indices::new(&context, bytemuck::cast_slice(&INDICES), wgpu::IndexFormat::Uint16);
        let mesh = GenericMesh::new(vertices, indices, 6);

        let material = GenericMaterial::new(
            context,
            vertex,
            fragment,
            &[UniformBuffer::layout_entry(0), Texture2D::layout_entry(1), Sampler::layout_entry(2)],
            &[
                buffer.group_entry(0),
                Texture2D::error_texture(context).group_entry(1),
                sampler.group_entry(2),
            ],
        );

        Self { texture, sampler, tint, buffer, material, mesh, waiting: true }
    }

    pub fn new(
        context: &VisContext, texture: Ptr<Texture2D>, tint: Vec4, coords: Option<&[f32]>,
        sampler: Option<Sampler>,
    ) -> Self {
        Self::new_custom(
            context,
            SPRITE_SHADER.clone(),
            SPRITE_SHADER.clone(),
            texture,
            tint,
            coords,
            sampler,
        )
    }

    pub fn set_coords(&mut self, context: &VisContext, coords: &[f32]) {
        let vertices = vec![
            Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [coords[0], coords[1]] },
            Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [coords[2], coords[3]] },
            Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [coords[4], coords[5]] },
            Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [coords[6], coords[7]] },
        ];

        self.mesh.update_vertices(context, bytemuck::cast_slice(&vertices));
    }

    pub fn set_coords_quad(&mut self, context: &VisContext, min: Vec2, max: Vec2) {
        let vertices = vec![
            Vertex2D { position: [-1.0, -1.0, -0.0], texture_coords: [min.x, max.y] },
            Vertex2D { position: [1.0, 1.0, -0.0], texture_coords: [max.x, min.y] },
            Vertex2D { position: [-1.0, 1.0, -0.0], texture_coords: [min.x, min.y] },
            Vertex2D { position: [1.0, -1.0, -0.0], texture_coords: [max.x, max.y] },
        ];

        self.mesh.update_vertices(context, bytemuck::cast_slice(&vertices));
    }

    pub fn set_texture(&mut self, texture: Ptr<Texture2D>) {
        if self.texture != texture {
            self.texture = texture;
            self.waiting = true;
        }
    }

    pub fn set_tint(&mut self, context: &VisContext, tint: Vec4) {
        if self.tint != tint {
            self.tint = tint;
            self.buffer.update_buffer(context, bytemuck::cast_slice(&tint.to_array()));
        }
    }

    pub fn texture(&self) -> &Ptr<Texture2D> {
        &self.texture
    }

    pub fn tint(&self) -> &Vec4 {
        &self.tint
    }

    pub fn update(&mut self, context: &VisContext, texture: &Texture2D) {
        if self.waiting {
            self.material.update_group(
                context,
                &[self.buffer.group_entry(0), texture.group_entry(1), self.sampler.group_entry(2)],
            );
            self.waiting = false;
        }
    }

    pub fn material(&self) -> &GenericMaterial {
        &self.material
    }

    pub fn mesh(&self) -> &GenericMesh<'a> {
        &self.mesh
    }
}

pub struct Animation2D {
    frames: Ptr<Texture2D>,
    frames_per_second: f64,
    current_frame: f32,
    total_frames: f32,
    mirrored: bool,
    looped: bool,
    delta: f64,
}

impl Animation2D {
    pub fn new(
        frames: Ptr<Texture2D>, frames_per_second: u32, total_frames: u32, mirrored: bool,
        looped: bool,
    ) -> Self {
        Self {
            frames,
            frames_per_second: frames_per_second as f64,
            total_frames: total_frames as f32,
            current_frame: 0.0,
            mirrored,
            looped,
            delta: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.current_frame = 0.0;
        self.delta = 0.0;
    }

    pub fn set_mirrored(&mut self, mirrored: bool) {
        self.mirrored = mirrored;
    }

    pub fn update(&mut self, context: &VisContext, delta: &Timestep, sprite: &mut Sprite) {
        if !self.looped && self.current_frame >= self.total_frames {
            return;
        }

        sprite.set_texture(self.frames);

        if self.delta > 1000.0 / self.frames_per_second {
            let mirror_value = if self.mirrored { 1.0 } else { 0.0 };

            sprite.set_coords_quad(
                context,
                Vec2::new((1.0 / self.total_frames) * (self.current_frame + mirror_value), 0.0),
                Vec2::new(
                    (1.0 / self.total_frames) * (self.current_frame + 1.0 - mirror_value),
                    1.0,
                ),
            );

            self.current_frame = (self.current_frame + 1.0) % self.total_frames;
            self.delta = 0.0;
        } else {
            self.delta += delta.millis();
        }
    }
}
