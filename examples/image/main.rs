#![allow(non_snake_case)]

use std::path::Path;

use glam::Vec4;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use RustyBear_Engine::assets::texture::Texture2D;
use RustyBear_Engine::context::Context;
use RustyBear_Engine::core::{Application, ModuleStack};
use RustyBear_Engine::environment::config::Config;
use RustyBear_Engine::event::{Event, EventSubscriber};
use RustyBear_Engine::input::{self, InputState};
use RustyBear_Engine::logging;
use RustyBear_Engine::render::render2d::Renderer2D;
use RustyBear_Engine::utils::Timestep;
use RustyBear_Engine::window::Window;

struct App {
    renderer: Renderer2D,
    input_state: InputState,
}

impl App {
    fn new(context: &Context) -> Self {
        //let texture = Texture2D::error_texture(context);

        let mut renderer = Renderer2D::new(context);
        renderer.set_background(
            &context.graphics,
            Texture2D::error_texture(&context.graphics),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        let input_state = InputState::new();

        Self { renderer, input_state }
    }
}

impl<'a> Application<'a> for App {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool {
        self.renderer.on_event(event, context);
        self.input_state.on_event(event, context);
        false
    }

    fn gui_render(&mut self, view: &wgpu::TextureView, context: &mut Context) {}

    fn quit(&mut self) {}

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        //Copy the texture to the view.
        self.renderer.render_background(context, view);
    }

    fn update(&mut self, delta: &Timestep, context: &mut Context) {
        /*self.texture.set_pixels(
            &context.graphics,
            (2, 2),
            &[255, 0, 0, 255, 0, 155, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255],
        );*/
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
    window.native.set_cursor_visible(false);

    let context = pollster::block_on(Context::new(window.native.clone(), config));

    let stack = ModuleStack::new();

    //Create and init the application
    let myapp = App::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window, stack);
}
