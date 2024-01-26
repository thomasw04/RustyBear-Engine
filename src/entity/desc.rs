use std::mem::size_of;

use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::{
        assets::{Ptr, SPRITE_SHADER},
        buffer::{Indices, UniformBuffer, Vertices},
        shader::Shader,
        texture::{Sampler, Texture2D},
    },
    context::VisContext,
    render::{
        material::GenericMaterial,
        mesh::GenericMesh,
        types::{BindGroupEntry, Vertex2D},
    },
};

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
    uniform: UniformBuffer,
    group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl Transform2D {
    pub fn new(context: &VisContext, position: Vec3, rotation: f32, scale: Vec2) -> Self {
        let mut uniform = UniformBuffer::new(context, size_of::<[[f32; 4]; 4]>());

        let transform: [[f32; 4]; 4] = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(scale.x, scale.y, 1.0),
            glam::Quat::from_rotation_z(rotation),
            glam::Vec3::new(position.x, position.y, 0.0),
        )
        .to_cols_array_2d();

        uniform.update_buffer(context, bytemuck::cast_slice(&transform));

        let layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[UniformBuffer::layout_entry(0)],
        });

        let group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[uniform.group_entry(0)],
        });

        Self { position, rotation, scale, uniform, group, layout }
    }

    fn update(&mut self, context: &VisContext) {
        let transform: [[f32; 4]; 4] = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(self.scale.x, self.scale.y, 1.0),
            glam::Quat::from_rotation_z(self.rotation),
            glam::Vec3::new(self.position.x, self.position.y, 0.0),
        )
        .to_cols_array_2d();

        self.uniform.update_buffer(context, bytemuck::cast_slice(&transform));
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

    pub fn set_position(&mut self, context: &VisContext, position: Vec3) {
        self.position = position;
        self.update(context);
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, context: &VisContext, rotation: f32) {
        self.rotation = rotation;
        self.update(context);
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, context: &VisContext, scale: Vec2) {
        self.scale = scale;
        self.update(context);
    }

    pub fn add_pos(&mut self, context: &VisContext, inc: Vec3) {
        self.position += inc;
        self.update(context);
    }

    pub fn add_rot(&mut self, context: &VisContext, inc: f32) {
        self.rotation += inc;
        self.update(context);
    }

    pub fn add_scale(&mut self, context: &VisContext, inc: Vec2) {
        self.scale += inc;
        self.update(context);
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
        tint: Vec4, coords: Option<&[f32]>,
    ) -> Self {
        let mut buffer = UniformBuffer::new(context, size_of::<[f32; 4]>());
        buffer.update_buffer(context, bytemuck::cast_slice(&tint.to_array()));
        let sampler = Sampler::two_dim(context);

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
    ) -> Self {
        Self::new_custom(
            context,
            SPRITE_SHADER.clone(),
            SPRITE_SHADER.clone(),
            texture,
            tint,
            coords,
        )
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
