use std::mem::size_of;

use glam::{Mat4, Quat, Vec3, Vec4};

use crate::{assets::buffer::UniformBuffer, context::VisContext, render::types::BindGroupEntry};

use super::world::World;

#[derive(Debug)]
struct WorldTransform {
    translation: Mat4,
    rotation: Quat,
}

#[derive(Debug)]
struct LocalTransform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

#[derive(Debug)]
pub struct Transform {
    local: LocalTransform,
    parent: WorldTransform,

    //GPU
    uniform: UniformBuffer,
    group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl Transform {
    pub fn new(context: &VisContext, position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        let mut uniform = UniformBuffer::new(context, size_of::<[[f32; 4]; 4]>());

        let translation = glam::Mat4::from_scale(scale) * glam::Mat4::from_translation(position);

        let rotation = Quat::from_scaled_axis(rotation);
        let world = WorldTransform { translation, rotation };
        let global = translation * Mat4::from_quat(rotation);

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

        Self { position, rotation, scale, world, uniform, group, layout, dirty: false }
    }

    fn update_buffer(&self, context: &VisContext) {
        let global = self.world.translation * Mat4::from_quat(self.world.rotation);
        self.uniform.update_buffer(context, bytemuck::cast_slice(&global.to_cols_array_2d()));
    }

    fn diff(&self) -> (Mat4, Quat) {
        let local = glam::Mat4::from_scale(self.local.scale)
            * glam::Mat4::from_translation(self.local.position);

        let trans_diff = local - self.world.translation;
        let rot_diff = self.local.rotation * self.world.rotation.conjugate();

        (trans_diff, rot_diff)
    }

    fn apply_local(&mut self) {
        self.world.translation =
            glam::Mat4::from_scale(self.scale) * glam::Mat4::from_translation(self.position);
        self.world.rotation = self.rotation;
    }

    pub fn update(context: &VisContext, world: &World) {
        for entity in world.dirties() {
            loop {
                if let Some(parent) = world.get::<&mut Transform>(*entity) {
                    let (mut transcale, mut rot) = parent.diff();

                    for entity in world.children::<Transform>((*entity).into()) {
                        if let Some(mut transform) = world.get::<&mut Transform>(entity.into()) {
                            if transform.dirty {}

                            transform.global = parent.global * transform.local;
                        }
                    }
                }
            }
        }
    }
}

impl Transform {
    pub fn group(&self) -> &wgpu::BindGroup {
        &self.group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn position(&self) -> Vec3 {
        self.local.position
    }

    pub fn move_by(&mut self, change: Vec3) {
        // Add to local
        self.local.position += change;

        //Add to global translation
        *self.world.translation.col_mut(3) += Vec4::new(change.x, change.y, change.z, 1.0);
    }

    pub fn scale_by(&mut self, change: Vec3) {
        //Add to local
        self.local.scale += change;

        //Add to global
        self.world.translation = self.world.translation.add_mat4(&Mat4::from_scale(change));
    }

    pub fn rotate_by(&mut self, change: Quat) {
        //Add to local
        self.local.rotation = self.local.rotation.mul_quat(change);

        //Add to global
        self.local.rotation = self.world.rotation.mul_quat(change);
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.move_by(position - self.position());
    }

    pub fn rotation(&self) -> Vec3 {
        self.local.rotation.to_scaled_axis()
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        let diff = Quat::from_scaled_axis(rotation) * self.local.rotation.conjugate();
        self.rotate_by(diff);
    }

    pub fn scale(&self) -> Vec3 {
        self.local.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale_by(scale - self.scale());
    }
}
