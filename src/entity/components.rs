use glam::{Vec3, Vec4};

use crate::assets::{assets::Ptr, texture::Texture2D};

pub struct Transformation {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transformation {
    fn default() -> Self {
        Transformation { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }
    }
}

pub struct Sprite {
    pub texture: Ptr<Texture2D>,
    pub tint: Vec4,
}
