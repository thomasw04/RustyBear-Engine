use rccell::RcCell;

use crate::{core::{Application}, input::InputState};

use event::EventSubscriber;
use gilrs::{Gilrs};
use log::{info, debug};
use utils::Timestep;
use window::Window;
use winit::{event::{Event, VirtualKeyCode, ElementState}};

use crate::core::ModuleStack;

pub mod utils;
#[macro_use] pub mod core;
pub mod event;
pub mod window;
pub mod logging;
pub mod entry;
pub mod input;


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

struct MyApp {

}

impl Application for MyApp { 

    fn init(&mut self, config_json: String, stack: &mut ModuleStack) -> Window
    {
        info!("Init Application");

        let handler = RcCell::new(MyHandler::new());
        stack.subscribe(event::EventType::Layer, handler);

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
    let mut gilrs = Gilrs::new().unwrap();

    println!();

    //Init the module stack.
    let mut apps = ModuleStack::new();

    //Register an EventSubscriber which maintains a list of current KeyStates.
    let input_state = rccell::RcCell::new(InputState::new());
    apps.subscribe(event::EventType::App, input_state.clone());

    //Create the application
    let mut myapp = MyApp::new();

    //Init the application and create the window
    let window = myapp.init("{}".to_string(), &mut apps);

    //Time since last frame
    let mut ts = Timestep::new();

    window.event_loop.run(enclose! { (input_state) move |event, _, control_flow|
    {
        myapp.update(ts.step_fwd());

        if input_state.borrow().is_key_down(&VirtualKeyCode::A) {
            info!("The A is down.");
        }

        let _handled = match event
        {
            Event::WindowEvent { window_id, ref event }

            if window_id == window.native.id() => Window::dispatch_event(&mut apps, event, control_flow),
            _ => {false}
        };

        let gilrs_event_option = gilrs.next_event();

        if gilrs_event_option.is_some() {
            let gilrs_event = gilrs_event_option.unwrap();
            Window::dispatch_gamepad_event(&mut apps, &gilrs_event, control_flow);
        }
    }});
}
