use glam::{Vec3, Vec4};

use crate::assets::{assets::Ptr, texture::Texture2D};

pub struct Transformation {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

pub struct Sprite {
    pub texture: Ptr<Texture2D>,
    pub tint: Vec4,
}
