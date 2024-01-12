#![allow(non_snake_case)]

use std::{cell::Ref, path::Path};

use glam::Vec2;
use rccell::RcCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::keyboard::KeyCode;
use RustyBear_Engine::{
    assets::assets::Assets,
    context::Context,
    core::{Application, ModuleStack},
    environment::config::Config,
    event::{Event, EventType},
    input::InputState,
    logging,
    render::{camera::OrthographicCamera, renderer::Renderer},
    utils::Timestep,
    window::Window,
};

pub struct TwoDimApp<'a> {
    stack: ModuleStack<'a>,
    renderer: RcCell<Renderer<'a>>,
    camera: RcCell<OrthographicCamera>,
}

impl<'a> Application<'a> for TwoDimApp<'a> {
    fn on_event(&mut self, _event: &Event, _context: &mut Context) -> bool {
        false
    }

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        {
            let mut renderer = self.renderer.borrow_mut();

            renderer.update_camera_buffer(
                &context.graphics,
                self.camera.borrow_mut().view_projection().to_cols_array_2d(),
            );

            let view_matrix = self.camera.borrow_mut().view().to_cols_array_2d();
            let projection = self.camera.borrow_mut().projection().inverse().to_cols_array_2d();

            renderer.update_skybox_buffer(&context.graphics, view_matrix, projection);

            renderer.render(context, view, window);
        }
    }

    fn gui_render(
        &mut self, _view: &wgpu::TextureView, _context: &mut Context, _gui_context: &egui::Context,
    ) {
    }

    fn update(&mut self, delta: &Timestep, input_state: Ref<InputState>, _context: &mut Context) {
        let mut cam = self.camera.borrow_mut();

        if input_state.is_key_down(&KeyCode::KeyA) {
            cam.inc_pos(Vec2::new(-(0.1 * delta.norm()), 0.0));
        }

        if input_state.is_key_down(&KeyCode::KeyD) {
            cam.inc_pos(Vec2::new(0.1 * delta.norm(), 0.0));
        }

        if input_state.is_key_down(&KeyCode::Space) {
            cam.inc_pos(Vec2::new(0.0, 0.1 * delta.norm()));
        }

        if input_state.is_key_down(&KeyCode::ShiftLeft) {
            cam.inc_pos(Vec2::new(0.0, -(0.1 * delta.norm())));
        }
    }

    fn quit(&mut self) {}

    fn get_stack(&mut self) -> &mut ModuleStack<'a> {
        &mut self.stack
    }
}

impl<'a> TwoDimApp<'a> {
    pub fn new(context: &Context) -> Self {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let loc = context.config.project_config().location.clone().map(what::Location::File);

        if let Some(what::Location::File(path)) = &loc {
            log::warn!("Project: {:?}", path);
        }

        let assets =
            Assets::new(context.graphics.clone(), loc, (context.free_memory() / 2) as usize);

        let renderer = RcCell::new(Renderer::new(context, assets));
        stack.subscribe(EventType::Layer, renderer.clone());

        let camera = RcCell::new(OrthographicCamera::default());
        stack.subscribe(EventType::Layer, camera.clone());

        camera.borrow_mut().set_aspect_ratio(
            context.surface_config.width as f32 / context.surface_config.height as f32,
        );

        TwoDimApp { stack, renderer, camera }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    //Create the config and init the example project.
    let mut config = Config::new(None);
    config.find_project(Path::new("examples/two_dim")).unwrap();

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    window.native.set_ime_allowed(true);
    window.native.set_cursor_visible(false);

    let context = pollster::block_on(Context::new(&mut window, config));

    //Create and init the application
    let myapp = TwoDimApp::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
