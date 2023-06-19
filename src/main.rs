#![allow(non_snake_case)]

use rccell::RcCell;

use crate::{core::{Application}, context::Context};

use event::EventSubscriber;
use log::{info};
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


struct MyHandler{
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event) -> bool
    {
        match event {
            event::Event::KeyboardInput { keycode, state } =>
            {
                match keycode 
                {
                    VirtualKeyCode::D => match state
                    {
                        ElementState::Pressed => info!("Pressed the d"),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }

        return false;
    }
}

impl MyHandler {
    pub fn new() -> MyHandler
    {
        MyHandler {  }
    }
}

struct MyApp<'a> {
    stack: ModuleStack<'a>
}

impl<'a> Application<'a> for MyApp<'a> { 

    fn init(&mut self, config_json: String) -> Window
    {
        info!("Init Application");

        let handler = RcCell::new(MyHandler::new());
        self.stack.subscribe(event::EventType::Layer, handler);

        Window::new(config_json)
    }

    fn get_stack(&mut self) -> &mut ModuleStack<'a> {
        return &mut self.stack;
    }

    fn update(&mut self, _delta: &utils::Timestep) {}

    fn quit(&mut self) {}
}

impl<'a> MyApp<'a> {
    pub fn new() -> MyApp<'a>
    {
        MyApp { stack: ModuleStack::new() }
    }
}

fn main() {
    logging::init();

    println!();

    //Create the application
    let mut myapp = MyApp::new();

    //Init the application and create the window
    let mut window = myapp.init("{}".to_string());
    let context = pollster::block_on(Context::new(&mut window));

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
