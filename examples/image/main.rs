#![allow(non_snake_case)]

use std::ops::Add;
use std::path::Path;

use glam::Vec4;
use noise::{NoiseFn, Perlin};
use rand::Rng;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use RustyBear_Engine::assets::texture::Texture2D;
use RustyBear_Engine::context::Context;
use RustyBear_Engine::core::{Application, ModuleStack};
use RustyBear_Engine::environment::config::Config;
use RustyBear_Engine::event::{Event, EventSubscriber};
use RustyBear_Engine::input::InputState;
use RustyBear_Engine::logging;
use RustyBear_Engine::render::material::BackgroundMaterial;
use RustyBear_Engine::render::render2d::{ImmType, Renderer2D};
use RustyBear_Engine::utils::Timestep;
use RustyBear_Engine::window::Window;

struct App {
    renderer: Renderer2D,
    input_state: InputState,
    texture: Texture2D,
    background: BackgroundMaterial,
    pixels: Vec<u8>,
    perlin: Perlin,
    size: usize,
    time: f32,
}

impl App {
    fn new(context: &Context) -> Self {
        let renderer = Renderer2D::new(context);
        let size = 512;

        let pixels = vec![255; size * size * 4];

        let texture =
            Texture2D::new(&context.graphics, None, (size as u32, size as u32), pixels.as_slice());
        let background = BackgroundMaterial::new(&context.graphics, &texture, Vec4::ONE);

        let input_state = InputState::new();

        let perlin = Perlin::new(Perlin::DEFAULT_SEED);

        Self { renderer, input_state, texture, background, pixels, perlin, size, time: 0.0 }
    }

    fn apply_lava(&mut self, scale: f64, speed: f64) {
        let mut rng = rand::thread_rng();

        for x in 0..self.size {
            for y in 0..self.size {
                let mut noise =
                    self.perlin.get([x as f64 * scale, y as f64 * scale, self.time as f64 * speed]);

                noise = (noise + 1.0) / 2.0;

                let index = (x * self.size + y) * 4;

                let r = noise * 50_f64;
                let g = noise * 240_f64;
                let b = noise * 70_f64;

                let decay = 0.95;

                self.pixels[index] = (self.pixels[index] as f64 * decay + r * (1.0 - decay)) as u8;
                self.pixels[index + 1] =
                    (self.pixels[index + 1] as f64 * decay + g * (1.0 - decay)) as u8;
                self.pixels[index + 2] =
                    (self.pixels[index + 2] as f64 * decay + b * (1.0 - decay)) as u8;

                let shifted = ((x + rng.gen_range(0..1)) * self.size + y + rng.gen_range(0..1)) * 4;

                if shifted < self.pixels.len() {
                    self.pixels[index] = self.pixels[shifted];
                    self.pixels[index + 1] = self.pixels[shifted + 1];
                    self.pixels[index + 2] = self.pixels[shifted + 2];
                }
            }
        }
    }
}

impl<'a> Application<'a> for App {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool {
        self.renderer.on_event(event, context);
        self.input_state.on_event(event, context);
        false
    }

    fn gui_render(&mut self, _: &wgpu::TextureView, _: &mut Context) {}

    fn quit(&mut self) {}

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        self.renderer.draw_imm(context, view, ImmType::Background(&self.background));
        self.renderer.draw_egui(context, view, window);
    }

    #[profiling::function]
    fn update(&mut self, delta: &Timestep, context: &mut Context) {
        self.apply_lava(0.01, 0.01);

        self.time = self.time.add(1.0 * delta.norm());

        self.texture.set_pixels(
            &context.graphics,
            (self.size as u32, self.size as u32),
            &self.pixels,
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    //Create the config and init the example project.
    let mut config = Config::new(None);
    config.find_project(Path::new("examples/hello_world")).unwrap();

    //Create the window from the config and create the context.
    let window = Window::new("{}".to_string());
    window.native.set_ime_allowed(true);
    window.native.set_cursor_visible(true);

    let context = pollster::block_on(Context::new(window.native.clone(), config));

    let stack = ModuleStack::new();

    //Create and init the application
    let myapp = App::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window, stack);
}
