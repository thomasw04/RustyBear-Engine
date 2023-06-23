use std::collections::HashMap;

use winit::event::ElementState;

use crate::{event::{EventSubscriber, Event}, context::{Context}};

#[allow(dead_code)] //TODO
#[derive(Default)]
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
        InputState::default()
    }

    pub fn is_key_down(&self, keycode: &winit::event::VirtualKeyCode) -> bool {
        *self.keyboard.get(keycode).unwrap_or(&false)
    }
}

impl EventSubscriber for InputState {
    fn on_event(&mut self, event: &Event, _context: &Context) -> bool
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
            Event::MouseInput { mousecode, state } =>
            {
                if *state == ElementState::Pressed
                {
                    self.mouse_button.insert(*mousecode, true);
                }
                else 
                {
                    self.mouse_button.insert(*mousecode, false);
                }
            },
            _ => {}
        }

        true
    }
}
