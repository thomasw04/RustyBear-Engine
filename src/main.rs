use std::{ops::ControlFlow, time::Instant};

use crate::core::{Application, Module};

use event::EventSubscriber;
use gilrs::{Gilrs};
use log::{info, debug};
use utils::Timestep;
use window::Window;
use winit::{event_loop::{EventLoop}, window::WindowBuilder, event::{Event, WindowEvent}};

use crate::core::ModuleStack;

pub mod utils;
pub mod core;
pub mod event;
pub mod window;
pub mod logging;
pub mod entry;


struct MyHandler{
}

impl EventSubscriber for MyHandler {
    fn on_event(&mut self, event: &event::Event) -> bool
    {
        info!("Event received.");
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
    fn init(&mut self, config_json: String, stack: &mut ModuleStack) -> Window
    {
        info!("Init Application");
        stack.subscribe(true, MyHandler::new());

        Window::new(config_json)
    }

    fn update(&mut self, delta: &utils::Timestep) {}
    fn quit(&mut self) {}
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

    let mut apps = ModuleStack::new();
    let mut gilrs = Gilrs::new().unwrap();

    let mut myapp = MyApp::new();
    let window = myapp.init("{}".to_string(), &mut apps);

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

            if window_id == window.native.id() => Window::dispatch_event(&mut apps, event, control_flow),
            _ => {}
        }

        let gilrs_event_option = gilrs.next_event();

        if gilrs_event_option.is_some() {
            let gilrs_event = gilrs_event_option.unwrap();
            Window::dispatch_gamepad_event(&mut apps, &gilrs_event, control_flow);
        }
    });
}
