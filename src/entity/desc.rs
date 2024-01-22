use glam::{Vec2, Vec3, Vec4};

use crate::{
    assets::{assets::Ptr, texture::Texture2D},
    context::VisContext,
    render::material::GenericMaterial,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transform2D {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform3D {
    fn default() -> Self {
        Transform3D { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Transform2D { position: Vec3::ZERO, rotation: 0.0, scale: Vec2::ONE }
    }
}
