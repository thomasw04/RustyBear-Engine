#![allow(non_snake_case)]

pub mod utils;
#[macro_use]
pub mod core;
pub mod config;
pub mod context;
pub mod entry;
pub mod event;
pub mod input;
pub mod logging;
pub mod render;
pub mod sound;
pub mod window;

use std::cell::Ref;

use input::InputState;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use rccell::RcCell;
use render::{camera::PerspectiveCamera, renderer::Renderer};

use crate::{config::load_themes, context::Context, core::Application, sound::AudioEngine};

use event::{Event, EventSubscriber};
use window::Window;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

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
    pub fn new() -> MyHandler {
        let theme_conf = load_themes();

        let mut audio = AudioEngine::new(&theme_conf);
        audio.play_background();

        MyHandler { audio }
    }
}

struct MyApp<'a> {
    stack: ModuleStack<'a>,
    renderer: RcCell<Renderer>,
    camera: RcCell<PerspectiveCamera>,
    demo_window: egui_demo_lib::DemoWindows,
}

impl<'a> Application<'a> for MyApp<'a> {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool {
        match event {
            event::Event::KeyboardInput { keycode, state } => match keycode {
                VirtualKeyCode::V => {
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

    fn gui_render(
        &mut self,
        view: &wgpu::TextureView,
        context: &mut Context,
        gui_context: &egui::Context,
    ) {
        self.demo_window.ui(gui_context);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        context: &mut Context,
        window: &winit::window::Window,
    ) {
        {
            let mut renderer = self.renderer.borrow_mut();

            renderer.update_camera_buffer(
                context,
                self.camera
                    .borrow_mut()
                    .view_projection()
                    .to_cols_array_2d(),
            );

            renderer.render(context, view, window);
        }
    }

    fn get_stack(&mut self) -> &mut ModuleStack<'a> {
        &mut self.stack
    }

    fn update(
        &mut self,
        delta: &utils::Timestep,
        input_state: Ref<InputState>,
        _context: &mut Context,
    ) {
        let mut cam = self.camera.borrow_mut();

        if input_state.is_key_down(&VirtualKeyCode::W) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.0, -(0.1 * delta.norm())));
        }

        if input_state.is_key_down(&VirtualKeyCode::S) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.0, 0.1 * delta.norm()));
        }

        if input_state.is_key_down(&VirtualKeyCode::A) {
            cam.inc_pos(glam::Vec3::new(-(0.1 * delta.norm()), 0.0, 0.0));
        }

        if input_state.is_key_down(&VirtualKeyCode::D) {
            cam.inc_pos(glam::Vec3::new(0.1 * delta.norm(), 0.0, 0.0));
        }

        if input_state.is_key_down(&VirtualKeyCode::Space) {
            cam.inc_pos(glam::Vec3::new(0.0, 0.1 * delta.norm(), 0.0));
        }

        if input_state.is_key_down(&VirtualKeyCode::LShift) {
            cam.inc_pos(glam::Vec3::new(0.0, -(0.1 * delta.norm()), 0.0));
        }
    }

    fn quit(&mut self) {}
}

impl<'a> MyApp<'a> {
    pub fn new(context: &Context) -> MyApp<'a> {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let handler = RcCell::new(MyHandler::new());
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = RcCell::new(Renderer::new(context));
        stack.subscribe(event::EventType::Layer, renderer.clone());

        let camera = RcCell::new(PerspectiveCamera::default());
        stack.subscribe(event::EventType::Layer, camera.clone());

        camera
            .borrow_mut()
            .set_aspect_ratio(context.config.width as f32 / context.config.height as f32);
        camera
            .borrow_mut()
            .set_position(glam::Vec3::new(0.0, 1.0, 2.0));

        MyApp {
            stack,
            renderer,
            camera,
            demo_window: egui_demo_lib::DemoWindows::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn entry_point() {
    logging::init();
    println!();

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    let context = pollster::block_on(Context::new(&mut window));

    //Create and init the application
    let myapp = MyApp::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
