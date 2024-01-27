#![allow(non_snake_case)]

pub mod utils;
#[macro_use]
pub mod core;
pub mod assets;
pub mod context;
pub mod entities;
pub mod entry;
pub mod environment;
pub mod event;
pub mod input;
pub mod logging;
pub mod render;
pub mod sound;
pub mod window;

use std::cell::Ref;

//Re-exports
pub use egui;
pub use glam;
pub use hecs;
pub use log;
pub use pollster;
pub use rccell;
pub use wgpu;
pub use what;
pub use winit;

use assets::assets::Assets;
use egui::lerp;
use glam::Vec3;
use input::InputState;

use rccell::RcCell;
use render::{camera::PerspectiveCamera, renderer::Renderer};

use crate::{context::Context, core::Application, sound::AudioEngine};

use event::{Event, EventSubscriber};
use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};

use crate::core::ModuleStack;

struct MyHandler {
    audio: AudioEngine,
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event, _context: &mut Context) -> bool {
        if let event::Event::MouseInput { mousecode, state } = event {
            match mousecode {
                MouseButton::Left => {
                    if state == &ElementState::Pressed {
                        self.audio.play_click()
                    }
                }
                MouseButton::Right => {
                    if state == &ElementState::Pressed {
                        self.audio.play_click()
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl MyHandler {
    pub fn new(context: &Context) -> MyHandler {
        let mut audio = AudioEngine::new(context.config.theme_config());
        audio.play_background();

        MyHandler { audio }
    }
}

pub struct RustyRuntime<'a> {
    stack: ModuleStack<'a>,
    renderer: RcCell<Renderer>,
    camera: RcCell<PerspectiveCamera>,
    demo_window: egui_demo_lib::DemoWindows,
}

impl<'a> Application<'a> for RustyRuntime<'a> {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool {
        match event {
            event::Event::KeyboardInput { keycode, state } => match keycode {
                KeyCode::KeyV => {
                    if *state == ElementState::Pressed {
                        context.set_vsync(!context.vsync());
                    }
                    false
                }
                _ => false,
            },
            _ => false,
        }
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

    fn gui_render(&mut self, _view: &wgpu::TextureView, context: &mut Context) {
        self.demo_window.ui(context.egui.egui_ctx());
    }

    fn update(
        &mut self, delta: &utils::Timestep, input_state: Ref<InputState>, context: &mut Context,
    ) {
        let mut cam = self.camera.borrow_mut();

        let (x, y) = input_state.get_mouse_pos();
        let (last_x, last_y) = input_state.get_last_mouse_pos();

        let (width, height) = (context.surface_config.width, context.surface_config.height);

        //Convert x and y to degrees using the window with and height.
        let (x, y) = ((x / width as f64) * 180.0 - 90.0, (y / height as f64) * 180.0 - 90.0);

        let (last_x, last_y) =
            ((last_x / width as f64) * 180.0 - 90.0, (last_y / height as f64) * 180.0 - 90.0);

        let newX = lerp(last_x..=x, 0.6 * delta.norm() as f64);
        let newY = lerp(last_y..=y, 0.6 * delta.norm() as f64);

        let rot = cam.rotation();

        cam.set_rotation(Vec3::new(-newY.clamp(-90.0, 90.0) as f32, -newX as f32, rot.z));

        if input_state.is_key_down(&KeyCode::KeyW) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.0, -(0.1 * delta.norm())));
        }

        if input_state.is_key_down(&KeyCode::KeyS) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.0, 0.1 * delta.norm()));
        }

        if input_state.is_key_down(&KeyCode::KeyA) {
            cam.inc_pos(glam::Vec3::new(-(0.1 * delta.norm()), 0.0, 0.0));
        }

        if input_state.is_key_down(&KeyCode::KeyD) {
            cam.inc_pos(glam::Vec3::new(0.1 * delta.norm(), 0.0, 0.0));
        }

        if input_state.is_key_down(&KeyCode::Space) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.1 * delta.norm(), 0.0));
        }

        if input_state.is_key_down(&KeyCode::ShiftLeft) {
            cam.inc_pos(glam::Vec3::new(0.0, -(0.1 * delta.norm()), 0.0));
        }
    }

    fn quit(&mut self) {}

    fn get_stack(&mut self) -> &mut ModuleStack<'a> {
        &mut self.stack
    }
}

impl<'a> RustyRuntime<'a> {
    pub fn new(context: &Context) -> RustyRuntime<'a> {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let loc = context.config.project_config().location.clone().map(what::Location::File);

        if let Some(what::Location::File(path)) = &loc {
            log::warn!("Project: {:?}", path);
        }

        let assets =
            Assets::new(context.graphics.clone(), loc, (context.free_memory() / 2) as usize);

        let handler = RcCell::new(MyHandler::new(context));
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = RcCell::new(Renderer::new(context, assets));
        stack.subscribe(event::EventType::Layer, renderer.clone());

        let camera = RcCell::new(PerspectiveCamera::default());
        stack.subscribe(event::EventType::Layer, camera.clone());

        camera.borrow_mut().set_aspect_ratio(
            context.surface_config.width as f32 / context.surface_config.height as f32,
        );
        camera.borrow_mut().set_position(glam::Vec3::new(0.0, 1.0, 2.0));

        camera.borrow_mut().set_centered(true);

        RustyRuntime { stack, renderer, camera, demo_window: egui_demo_lib::DemoWindows::default() }
    }
}
