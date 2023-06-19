use std::collections::HashMap;

use winit::event::ElementState;

use crate::event::{EventSubscriber, Event};

#[allow(dead_code)] //TODO
pub struct InputState {
    keyboard: HashMap<winit::event::VirtualKeyCode, bool>,
    mouse_button: HashMap<winit::event::MouseButton, bool>,
    gamepad_button: HashMap<gilrs::Button, bool>,
    gamepad_axis: HashMap<gilrs::Axis, f32>,
    mouse_position: (f64, f64),
}

impl InputState {

    pub fn new() -> InputState
    {
        InputState { keyboard: HashMap::new(), mouse_button: HashMap::new(), gamepad_button: HashMap::new(), gamepad_axis: HashMap::new(), mouse_position: (0.0, 0.0) }
    }

    pub fn is_key_down(&self, keycode: &winit::event::VirtualKeyCode) -> bool {
        *self.keyboard.get(keycode).unwrap_or(&false)
    }
}

impl EventSubscriber for InputState {
    fn on_event(&mut self, event: &Event) -> bool
    {
        match event {
            Event::KeyboardInput { keycode, state } =>
            {
                if *state == ElementState::Pressed
                {
                    self.keyboard.insert(*keycode, true);
                }
                else 
                {
                    self.keyboard.insert(*keycode, false);
                }
            },
            _ => {}
        }

        return true;
    }
}
