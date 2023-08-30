#![allow(non_snake_case)]

pub mod utils;
#[macro_use] pub mod core;
pub mod event;
pub mod window;
pub mod logging;
pub mod entry;
pub mod input;
pub mod context;
pub mod render;
pub mod sound;
pub mod config;

use std::cell::Ref;

use input::InputState;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use rccell::RcCell;
use render::{renderer::Renderer, camera::PerspectiveCamera};

use crate::{core::Application, context::Context, config::load_themes, sound::AudioEngine};

use event::{EventSubscriber, Event};
use window::Window;
use winit::event::{VirtualKeyCode, ElementState, MouseButton};

use crate::core::ModuleStack;

struct MyHandler{
    audio: AudioEngine
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event, _context: &mut Context) -> bool
    {
        if let event::Event::MouseInput { mousecode, state } = event {
            match mousecode {
                MouseButton::Left => if state == &ElementState::Pressed { self.audio.play_click() },
                MouseButton::Right => if state == &ElementState::Pressed { self.audio.play_click() },
                _ => {}
            }
        }
        false
    }
}

impl MyHandler {
    pub fn new() -> MyHandler
    {
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
}

impl<'a> Application<'a> for MyApp<'a> { 

    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool
    {
        match event {
            event::Event::KeyboardInput { keycode, state } => {
                match keycode {
                    VirtualKeyCode::V => {
                        if *state == ElementState::Pressed {
                            context.set_vsync(!context.vsync());
                        }
                        false
                    },
                    _ => {false}
                }
                
            },
            _ => {false}
        }
    }

    fn render(&mut self, view: wgpu::TextureView, context: &mut Context)
    {
        {
            let mut renderer = self.renderer.borrow_mut();

            renderer.update_camera_buffer(context, self.camera.borrow_mut().view_projection().to_cols_array_2d());

            renderer.render(context, view);
        }
    } 

    fn get_stack(&mut self) -> &mut ModuleStack<'a>
    {
        &mut self.stack
    }

    fn update(&mut self, delta: &utils::Timestep, input_state: Ref<InputState>, context: &mut Context) 
    {
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
    pub fn new(context: &Context) -> MyApp<'a>
    {
        log::info!("Init Application");

        let mut  stack = ModuleStack::new();

        let handler = RcCell::new(MyHandler::new());
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = RcCell::new(Renderer::new(context));
        stack.subscribe(event::EventType::Layer, renderer.clone());

        let camera = RcCell::new(PerspectiveCamera::new());
        stack.subscribe(event::EventType::Layer, camera.clone());

        camera.borrow_mut().set_aspect_ratio(context.config.width as f32 / context.config.height as f32);
        camera.borrow_mut().set_position(glam::Vec3::new(0.0, 1.0, 2.0));

        MyApp { stack, renderer, camera }
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

