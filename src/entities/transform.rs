use std::ops::Mul;

use glam::{Mat4, Quat, Vec3};
use once_cell::sync::OnceCell;

use crate::{assets::buffer::UniformBuffer, context::VisContext};

use super::world::World;

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
}

#[derive(Debug)]
struct WorldTransform {
    translation: Mat4,
    rotation: Quat,
}

impl Mul<LocalTransform> for WorldTransform {
    type Output = WorldTransform;

    fn mul(self, rhs: LocalTransform) -> Self::Output {
        let translation = self.translation * Mat4::from_translation(rhs.position);
        let rotation = self.rotation * rhs.rotation;

        Self { translation, rotation }
    }
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self { translation: Mat4::IDENTITY, rotation: Quat::IDENTITY }
    }
}

impl Transform {
    pub fn new(context: &VisContext, position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        let translation = glam::Mat4::from_scale(scale) * glam::Mat4::from_translation(position);

        let rotation = Quat::from_scaled_axis(rotation);
        let local = LocalTransform { position, rotation, scale };

        Self { local, parent: WorldTransform::default() }
    }

    pub fn layout(context: &VisContext) -> &'static wgpu::BindGroupLayout {
        static LAYOUT: OnceCell<wgpu::BindGroupLayout> = OnceCell::new();

        LAYOUT.get_or_init(|| {
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Transform Layout"),
                entries: &[UniformBuffer::layout_entry(0)],
            })
        })
    }

    /// Transform System: Updates the world transform of all entities.
    /// Note: This should be optimized sometime.
    pub fn update(context: &VisContext, world: &World) {
        for (entity, _) in world.root::<Transform>() {
            let mut stack = vec![entity.into()];

            while let Some(entity) = stack.pop() {
                if let Some(parent) = world.get::<&mut Transform>(entity) {
                    let new_parent = parent.parent * parent.local;
                    for entity in world.children::<Transform>((entity).into()) {
                        if let Some(mut transform) = world.get::<&mut Transform>(entity.into()) {
                            transform.parent = new_parent;
                        }

                        stack.push(entity.into());
                    }
                }
            }
        }
    }
}

impl Transform {
    pub fn position(&self) -> Vec3 {
        self.local.position
    }

    pub fn move_by(&mut self, change: Vec3) {
        self.local.position += change;
    }

    pub fn scale_by(&mut self, change: Vec3) {
        self.local.scale += change;
    }

    pub fn rotate_by(&mut self, change: Quat) {
        self.local.rotation = self.local.rotation.mul_quat(change);
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.local.position = position;
    }

    pub fn rotation(&self) -> Vec3 {
        self.local.rotation.to_scaled_axis()
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.local.rotation = Quat::from_scaled_axis(rotation);
    }

    pub fn scale(&self) -> Vec3 {
        self.local.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.local.scale = scale;
    }
}
