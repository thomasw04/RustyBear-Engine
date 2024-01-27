use std::mem::size_of;

use glam::{Mat4, Quat, Vec2, Vec3};

use crate::assets::buffer::UniformBuffer;
use crate::context::VisContext;
use crate::render::types::BindGroupEntry;

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

        let transform: [[f32; 4]; 4] = Mat4::from_scale_rotation_translation(
            Vec3::new(scale.x, scale.y, 1.0),
            Quat::from_rotation_z(rotation),
            Vec3::new(position.x, position.y, 0.0),
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
        let transform: [[f32; 4]; 4] = Mat4::from_scale_rotation_translation(
            Vec3::new(self.scale.x, self.scale.y, 1.0),
            Quat::from_rotation_z(self.rotation),
            Vec3::new(self.position.x, self.position.y, 0.0),
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
