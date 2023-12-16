#![allow(non_snake_case)]

pub mod utils;
#[macro_use]
pub mod core;
pub mod assets;
pub mod context;
pub mod entry;
pub mod environment;
pub mod event;
pub mod input;
pub mod logging;
pub mod render;
pub mod sound;
pub mod window;

use std::{cell::Ref, fs::File, ops::Sub, panic::Location};

use assets::manager::AssetManager;
use glam::Vec3;
use input::InputState;

use rccell::RcCell;
use render::{camera::PerspectiveCamera, renderer::Renderer};

use crate::{context::Context, core::Application, environment::config::Config, sound::AudioEngine};

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
    asset_manager: AssetManager,
    demo_window: egui_demo_lib::DemoWindows,
    last_mouse_pos: (f64, f64),
}

impl<'a> Application<'a> for RustyRuntime<'a> {
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

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        context: &mut Context,
        window: &winit::window::Window,
    ) {
        {
            let mut renderer = self.renderer.borrow_mut();
            self.asset_manager.update();

            renderer.update_camera_buffer(
                &context.graphics,
                self.camera
                    .borrow_mut()
                    .view_projection()
                    .to_cols_array_2d(),
            );

            let view_matrix = self.camera.borrow_mut().view().to_cols_array_2d();
            let projection = self
                .camera
                .borrow_mut()
                .projection()
                .inverse()
                .to_cols_array_2d();

            renderer.update_skybox_buffer(&context.graphics, view_matrix, projection);

            renderer.render(context, view, window, &self.asset_manager);
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

    fn update(
        &mut self,
        delta: &utils::Timestep,
        input_state: Ref<InputState>,
        _context: &mut Context,
    ) {
        let mut cam = self.camera.borrow_mut();

        let (mut dx, mut dy) = input_state.get_mouse_pos();
        let (last_dx, last_dy) = self.last_mouse_pos;
        dx -= last_dx;
        dy -= last_dy;

        let rot = cam.rotation();

        cam.set_rotation(Vec3::new(
            (rot.x - (dy as f32 * delta.norm() * 1.0)).clamp(-90.0, 90.0),
            rot.y - (dx as f32 * delta.norm() * 1.0),
            rot.z,
        ));

        self.last_mouse_pos = input_state.get_mouse_pos();

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

    fn get_stack(&mut self) -> &mut ModuleStack<'a> {
        &mut self.stack
    }
}

impl<'a> RustyRuntime<'a> {
    pub fn new(context: &Context) -> RustyRuntime<'a> {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let loc = context
            .config
            .project_config()
            .location
            .clone()
            .map(what::Location::File);

        if let Some(loc) = &loc {
            if let what::Location::File(path) = loc {
                log::warn!("Project: {:?}", path);
            }
        }

        let mut asset_manager = AssetManager::new(
            context.graphics.clone(),
            loc,
            (context.free_memory() / 2) as usize,
        );

        let handler = RcCell::new(MyHandler::new(context));
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = RcCell::new(Renderer::new(context, &mut asset_manager));
        stack.subscribe(event::EventType::Layer, renderer.clone());

        let camera = RcCell::new(PerspectiveCamera::default());
        stack.subscribe(event::EventType::Layer, camera.clone());

        camera.borrow_mut().set_aspect_ratio(
            context.surface_config.width as f32 / context.surface_config.height as f32,
        );
        camera
            .borrow_mut()
            .set_position(glam::Vec3::new(0.0, 1.0, 2.0));

        RustyRuntime {
            stack,
            renderer,
            camera,
            asset_manager,
            demo_window: egui_demo_lib::DemoWindows::default(),
            last_mouse_pos: (0.0, 0.0),
        }
    }
}

pub fn example_app() {
    logging::init();
    println!();

    let config = Config::new(None);

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    let context = pollster::block_on(Context::new(&mut window, config));

    //Create and init the application
    let myapp = RustyRuntime::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
