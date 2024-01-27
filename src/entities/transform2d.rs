use std::mem::size_of;

use glam::{Mat4, Vec2, Vec3};

use crate::assets::buffer::UniformBuffer;
use crate::context::VisContext;
use crate::render::types::BindGroupEntry;
use hecs_hierarchy::Hierarchy;

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
