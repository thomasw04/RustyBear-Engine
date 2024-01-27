use crate::assets::assets::Ptr;
use crate::assets::texture::Texture2D;
use crate::context::VisContext;
use crate::entities::sprite::Sprite;
use crate::utils::Timestep;
use glam::Vec2;

pub struct Animation2D {
    frames: Ptr<Texture2D>,
    frames_per_second: f64,
    current_frame: f32,
    total_frames: f32,
    mirrored: bool,
    looped: bool,
    delta: f64,
}

impl Animation2D {
    pub fn new(
        frames: Ptr<Texture2D>, frames_per_second: u32, total_frames: u32, mirrored: bool,
        looped: bool,
    ) -> Self {
        Self {
            frames,
            frames_per_second: frames_per_second as f64,
            total_frames: total_frames as f32,
            current_frame: 0.0,
            mirrored,
            looped,
            delta: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.current_frame = 0.0;
        self.delta = 0.0;
    }

    pub fn set_mirrored(&mut self, mirrored: bool) {
        self.mirrored = mirrored;
    }

    pub fn update(&mut self, context: &VisContext, delta: &Timestep, sprite: &mut Sprite) {
        if !self.looped && self.current_frame >= self.total_frames {
            return;
        }

        sprite.set_texture(self.frames);

        if self.delta > 1000.0 / self.frames_per_second {
            let mirror_value = if self.mirrored { 1.0 } else { 0.0 };

            sprite.set_coords_quad(
                context,
                Vec2::new((1.0 / self.total_frames) * (self.current_frame + mirror_value), 0.0),
                Vec2::new(
                    (1.0 / self.total_frames) * (self.current_frame + 1.0 - mirror_value),
                    1.0,
                ),
            );

            self.current_frame = (self.current_frame + 1.0) % self.total_frames;
            self.delta = 0.0;
        } else {
            self.delta += delta.millis();
        }
    }
}
