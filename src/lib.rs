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

//Re-exports
pub use egui;
pub use glam;
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

pub struct RustyRuntime {
    renderer: Renderer,
    camera: PerspectiveCamera,
    input_state: InputState,
    demo_window: egui_demo_lib::DemoWindows,
}

impl<'a> Application<'a> for RustyRuntime {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool {
        if let event::Event::KeyboardInput { keycode, state } = event {
            match keycode {
                KeyCode::KeyV => {
                    if *state == ElementState::Pressed {
                        context.set_vsync(!context.vsync());
                    }
                }
                KeyCode::Escape => {
                    if *state == ElementState::Pressed {
                        log::info!("Quitting Application");
                    }
                }
                _ => {}
            }
        }

        self.input_state.on_event(event, context);
        self.renderer.on_event(event, context);
        self.camera.on_event(event, context);
        false
    }

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        {
            self.renderer.update_camera_buffer(
                &context.graphics,
                self.camera.view_projection().to_cols_array_2d(),
            );

            let view_matrix = self.camera.view().to_cols_array_2d();
            let projection = self.camera.projection().inverse().to_cols_array_2d();

            self.renderer.update_skybox_buffer(&context.graphics, view_matrix, projection);

            self.renderer.render(context, view, window);
        }
    }

    fn gui_render(&mut self, _view: &wgpu::TextureView, context: &mut Context) {
        self.demo_window.ui(context.egui.egui_ctx());
    }

    fn update(&mut self, delta: &utils::Timestep, context: &mut Context) {
        let (x, y) = self.input_state.get_mouse_pos();
        let (last_x, last_y) = self.input_state.get_last_mouse_pos();

        let (width, height) = (context.surface_config.width, context.surface_config.height);

        //Convert x and y to degrees using the window with and height.
        let (x, y) = ((x / width as f64) * 180.0 - 90.0, (y / height as f64) * 180.0 - 90.0);

        let (last_x, last_y) =
            ((last_x / width as f64) * 180.0 - 90.0, (last_y / height as f64) * 180.0 - 90.0);

        let newX = lerp(last_x..=x, 0.6 * delta.norm() as f64);
        let newY = lerp(last_y..=y, 0.6 * delta.norm() as f64);

        let rot = self.camera.rotation();

        self.camera.set_rotation(Vec3::new(-newY.clamp(-90.0, 90.0) as f32, -newX as f32, rot.z));

        if self.input_state.is_key_down(&KeyCode::KeyW) {
            self.camera.inc_pos(glam::Vec3::new(0.0, 0.0, -(0.1 * delta.norm())));
        }

        if self.input_state.is_key_down(&KeyCode::KeyS) {
            self.camera.inc_pos(glam::Vec3::new(0.0, 0.0, 0.1 * delta.norm()));
        }

        if self.input_state.is_key_down(&KeyCode::KeyA) {
            self.camera.inc_pos(glam::Vec3::new(-(0.1 * delta.norm()), 0.0, 0.0));
        }

        if self.input_state.is_key_down(&KeyCode::KeyD) {
            self.camera.inc_pos(glam::Vec3::new(0.1 * delta.norm(), 0.0, 0.0));
        }

        if self.input_state.is_key_down(&KeyCode::Space) {
            self.camera.inc_pos(glam::Vec3::new(0.0, 0.1 * delta.norm(), 0.0));
        }

        if self.input_state.is_key_down(&KeyCode::ShiftLeft) {
            self.camera.inc_pos(glam::Vec3::new(0.0, -(0.1 * delta.norm()), 0.0));
        }
    }

    fn quit(&mut self) {}
}

impl<'a> RustyRuntime {
    pub fn new(context: &Context, stack: &mut ModuleStack<'a>) -> RustyRuntime {
        log::info!("Init Application");

        let loc = context.config.project_config().location.clone().map(what::Location::File);

        if let Some(what::Location::File(path)) = &loc {
            log::warn!("Project: {:?}", path);
        }

        let assets =
            Assets::new(context.graphics.clone(), loc, (context.free_memory() / 2) as usize);

        let handler = RcCell::new(MyHandler::new(context));
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = Renderer::new(context, assets);
        let mut camera = PerspectiveCamera::default();

        camera.set_aspect_ratio(
            context.surface_config.width as f32 / context.surface_config.height as f32,
        );
        camera.set_position(glam::Vec3::new(0.0, 1.0, 2.0));
        camera.set_centered(true);

        RustyRuntime {
            renderer,
            camera,
            input_state: InputState::new(),
            demo_window: egui_demo_lib::DemoWindows::default(),
        }
    }
}
