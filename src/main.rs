use std::{ops::ControlFlow, time::Instant};

use crate::core::{Application, Module, Windowable};

use event::EventSubscriber;
use log::{info, debug};
use utils::Timestep;
use window::Window;
use winit::{event_loop::{EventLoop}, window::WindowBuilder, event::{Event, WindowEvent}};

use crate::core::ApplicationStack;

pub mod utils;
pub mod core;
pub mod event;
pub mod window;
pub mod logging;


struct MyHandler{
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event) -> bool
    {
        info!("Window closing.");
        return false;
    }
}

impl MyHandler {
    pub fn new() -> MyHandler
    {
        MyHandler {  }
    }
}

struct MyApp {

}

impl Application for MyApp { 
    fn init(&mut self, config_json: String, stack: &mut ApplicationStack)
    {
        info!("Init Application");
        stack.subscribe(true, MyHandler::new());
    }

    fn update(&mut self, delta: &utils::Timestep) {}
    fn quit(&mut self) {}
}

impl Windowable for MyApp {
    fn create_window(&mut self, name: String) -> window::Window
    {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title(name).build(&event_loop).unwrap();
        Window::new(window, event_loop)
    }
}

impl MyApp {
    pub fn new() -> MyApp
    {
        MyApp {  }
    }
}

fn main() {
    logging::init();

    println!();

    let mut apps = ApplicationStack::new();

    let mut myapp = MyApp::new();
    myapp.init("{}".to_string(), &mut apps);

    let window = myapp.create_window("RustyBear".to_string());

    let mut last = Instant::now().elapsed().as_nanos();

    window.event_loop.run(move |event, _, control_flow|
    {
        let now = Instant::now().elapsed().as_nanos();
        let ts: Timestep = (now.saturating_sub(last) as f64 / 1000.0).into();
        last = now;

        myapp.update(&ts);

        match event
        {
            Event::WindowEvent { window_id, ref event }

            if window_id == window.native.id() => match event {
                WindowEvent::CloseRequested => {
                    apps.dispatch_event(true, &event::Event::CloseRequested);
                    control_flow.set_exit();
                },
                _ => {}
            },
            _ => {}
        }
    });
}
