use crate::assets::assets::{Ptr, SPRITE_SHADER};
use crate::assets::buffer::{Indices, UniformBuffer, Vertices};
use crate::assets::shader::Shader;
use crate::assets::texture::{Sampler, Texture2D};
use crate::context::VisContext;

use crate::render::material::GenericMaterial;
use crate::render::mesh::GenericMesh;
use crate::render::types::{BindGroupEntry, Vertex2D};
use glam::{Vec2, Vec4};
use std::mem::size_of;

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
