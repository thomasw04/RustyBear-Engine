#![allow(non_snake_case)]

use std::cell::Ref;
use std::path::Path;

use glam::{Vec2, Vec4};
use hecs::World;
use rccell::RcCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::keyboard::KeyCode;
use RustyBear_Engine::assets::assets::Assets;
use RustyBear_Engine::context::Context;
use RustyBear_Engine::core::{Application, ModuleStack};
use RustyBear_Engine::entity::desc::Transform2D;
use RustyBear_Engine::entity::entities::Worlds;
use RustyBear_Engine::environment::config::Config;
use RustyBear_Engine::event::{Event, EventType};
use RustyBear_Engine::input::InputState;
use RustyBear_Engine::logging;
use RustyBear_Engine::render::camera::OrthographicCamera;
use RustyBear_Engine::render::render2d::{RenderData, Renderer2D};
use RustyBear_Engine::utils::Timestep;
use RustyBear_Engine::window::Window;

pub struct LDTKApp<'a> {
    stack: ModuleStack<'a>,
    assets: Assets,
    worlds: Worlds,
    renderer: RcCell<Renderer2D>,
    camera: RcCell<OrthographicCamera>,
}

impl<'a> Application<'a> for LDTKApp<'a> {
    fn on_event(&mut self, _event: &Event, _context: &mut Context) -> bool { false }

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        {
            let mut renderer = self.renderer.borrow_mut();

            renderer.update_camera_buffer(
                &context.graphics,
                self.camera.borrow_mut().view_projection().to_cols_array_2d(),
            );

            let render_data = RenderData { ctx: context, view, window };

            renderer.render(render_data, &mut self.assets, &mut self.worlds);
        }
    }

    fn gui_render(
        &mut self, _view: &wgpu::TextureView, _context: &mut Context, _gui_context: &egui::Context,
    ) {
    }

    fn update(&mut self, delta: &Timestep, input_state: Ref<InputState>, context: &mut Context) {
        let mut cam = self.camera.borrow_mut();

        if input_state.is_key_down(&KeyCode::KeyD) {
            cam.inc_pos(Vec2::new(-(0.01 * delta.norm()), 0.0));
        }

        if input_state.is_key_down(&KeyCode::KeyA) {
            cam.inc_pos(Vec2::new(0.01 * delta.norm(), 0.0));
        }

        if input_state.is_key_down(&KeyCode::KeyS) {
            cam.inc_pos(Vec2::new(0.0, 0.01 * delta.norm()));
        }

        if input_state.is_key_down(&KeyCode::KeyW) {
            cam.inc_pos(Vec2::new(0.0, -(0.01 * delta.norm())));
        }
    }

    fn quit(&mut self) {}

    fn get_stack(&mut self) -> &mut ModuleStack<'a> { &mut self.stack }
}

impl<'a> LDTKApp<'a> {
    pub fn new(context: &Context) -> Self {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let loc = context.config.project_config().location.clone().map(what::Location::File);

        if let Some(what::Location::File(path)) = &loc {
            log::warn!("Project: {:?}", path);
        }

        let mut assets =
            Assets::new(context.graphics.clone(), loc, (context.free_memory() / 2) as usize);

        let worlds = Worlds::from_ldtk_file(&context.graphics,
            &context.config.project_config().location.clone(),
            &mut assets, "examples/ldtk/data/test.ldtk").expect(
			"Failed to load ldtk file. Make sure you have the test.ldtk file in the examples/ldtk folder",
		);

        let renderer = RcCell::new(Renderer2D::new(context, &mut assets));
        stack.subscribe(EventType::Layer, renderer.clone());

        let mut cam = OrthographicCamera::default();
        let pos = cam.position();
        cam.set_position(Vec2::new(pos.x, 0.5));
        let camera = RcCell::new(cam);
        stack.subscribe(EventType::Layer, camera.clone());

        // Set it to bottom of the screen

        LDTKApp { stack, assets, worlds, renderer, camera }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    //Create the config and init the example project.
    let mut config = Config::new(None);
    config.find_project(Path::new("examples/ldtk")).unwrap();

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    window.native.set_ime_allowed(true);
    window.native.set_cursor_visible(false);

    let context = pollster::block_on(Context::new(&mut window, config));

    //Create and init the application
    let myapp = LDTKApp::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
