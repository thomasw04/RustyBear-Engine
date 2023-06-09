
use std::path::PathBuf;

use gilrs::{GamepadId, Button};
use winit::event::{ModifiersState, VirtualKeyCode, MouseButton};

#[derive(Clone)]
pub enum GamepadButtonState {
    Pressed,
    Released,
    Repeated
}


#[derive(Clone)]
pub enum Event {
    //Winit Events
    Resized { width: u32, height: u32 },
    Moved { x: i32, y: i32 },
    CloseRequested,
    Destroyed,
    DroppedFile (PathBuf),
    HoveredFile (PathBuf),
    HoveredFileCancelled,
    ReceivedCharacter (char),
    Focused (bool),
    KeyboardInput {keycode: winit::event::VirtualKeyCode, state: winit::event::ElementState },
    ModifiersChanged (winit::event::ModifiersState),
    CursorMoved { x: f64, y: f64 },
    CursorEntered,
    CursorLeft,
    MouseWheel { delta_x: f64, delta_y: f64, state: winit::event::TouchPhase },
    MouseScroll { delta_x: f32, delta_y: f32, state: winit::event::TouchPhase },
    MouseInput { mousecode: winit::event::MouseButton, state: winit::event::ElementState },

    //gilrs Events todo
    GamepadInput {id: GamepadId, buttoncode: gilrs::Button, state: GamepadButtonState},
    GamepadInputChanged {id: GamepadId, scancode: u32, value: f32},
    GamepadAxis {id: GamepadId, axiscode: gilrs::Axis, value: f32},
    GamepadConnected {id: GamepadId},
    GamepadDisconnected {id: GamepadId},
    GamepadDropped {id: GamepadId}
}

#[derive(Clone)]
pub enum EventType {
    App,
    Layer,
}


pub trait EventSubscriber {
    fn on_event(&mut self, event: &Event) -> bool;
}

pub struct EventStack<'a> {
    input_stack: Vec<Box<dyn FnMut(&Event) -> bool + 'a>>,
    app_stack: Vec<Box<dyn FnMut(&Event) -> bool + 'a>>,
}

impl<'a> EventStack<'a> {

    pub fn new() -> EventStack<'a>
    {
        EventStack { input_stack: Vec::new(), app_stack: Vec::new() }
    }

    pub fn push(&mut self, event_type: EventType, callback: impl FnMut(&Event) -> bool + 'a) -> usize
    {
        match event_type {
            EventType::App => {
                self.app_stack.push(Box::new(callback));
                return self.app_stack.len()-1;
            },

            EventType::Layer => {
                self.input_stack.push(Box::new(callback));
                return self.input_stack.len()-1;
            }
        }
    }

    pub fn swap(&mut self, lhs: usize, rhs: usize)
    {
        self.input_stack.swap(lhs, rhs);
    }

    pub fn propagate_event(&mut self, event: &Event) -> bool
    {
        self.propagate_app_event(event);

        for callback in self.input_stack.iter_mut().rev()
        {
            if (callback)(event)
            {
                return true;
            }
        }

        return false;
    }

    pub fn propagate_app_event(&mut self, event: &Event) -> bool
    {
        for callback in self.app_stack.iter_mut()
        {
            if !(callback)(event)
            {
                log::error!("Error while processing event. Application layer returned false.");
                return false;
            }
        }

        return true;
    }

    pub fn pop(&mut self) -> usize
    {
        self.input_stack.pop();
        return self.input_stack.len()-1;
    }

    pub fn remove(&mut self, index: usize)
    {
        let _dying_closure = self.app_stack.remove(index);
    }
}