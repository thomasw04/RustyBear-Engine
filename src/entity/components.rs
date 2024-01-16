use glam::{Vec3, Vec4};

use crate::assets::{assets::Ptr, texture::Texture2D};

pub struct Transformation {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

pub struct Sprite {
    texture: Ptr<Texture2D>,
    tint: Vec4,
}
