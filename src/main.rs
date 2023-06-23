#![allow(non_snake_case)]

use rccell::RcCell;
use renderer::Renderer2D;

use crate::{core::{Application}, context::Context};

use event::EventSubscriber;
use window::Window;
use winit::{event::{VirtualKeyCode, ElementState}};

use crate::core::ModuleStack;

pub mod utils;
#[macro_use] pub mod core;
pub mod event;
pub mod window;
pub mod logging;
pub mod entry;
pub mod input;
pub mod context;
pub mod buffer;
pub mod renderer;


struct MyHandler{
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event, _context: &Context) -> bool
    {
        if let event::Event::KeyboardInput { keycode, state } = event {
            match keycode {
                VirtualKeyCode::D => if state == &ElementState::Pressed { log::info!("Pressed the d") },
                VirtualKeyCode::E => if state == &ElementState::Pressed { log::info!("Pressed the e") },
                _ => {}
            }
        }
        false
    }
}

impl MyHandler {
    pub fn new() -> MyHandler
    {
        MyHandler {  }
    }
}

struct MyApp<'a> {
    stack: ModuleStack<'a>,
    renderer: RcCell<Renderer2D>,
}

impl<'a> Application<'a> for MyApp<'a> { 

    fn render(&mut self, view: wgpu::TextureView, context: &mut Context)
    {
        {
            let mut renderer = self.renderer.borrow_mut();
            renderer.render(context, view);
        }
    } 

    fn get_stack(&mut self) -> &mut ModuleStack<'a>
    {
        &mut self.stack
    }

    fn update(&mut self, _delta: &utils::Timestep) {}

    fn quit(&mut self) {}
}

impl<'a> MyApp<'a> {
    pub fn new(context: &Context) -> MyApp<'a>
    {
        log::info!("Init Application");

        let mut  stack = ModuleStack::new();

        let handler = RcCell::new(MyHandler::new());
        stack.subscribe(event::EventType::Layer, handler);

        let renderer = RcCell::new(Renderer2D::new(context));
        stack.subscribe(event::EventType::Layer, renderer.clone());

        MyApp { stack, renderer }
    }
}

fn main() {
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
