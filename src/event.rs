
use std::path::PathBuf;

use gilrs::GamepadId;
use winit::event::ModifiersState;

#[derive(Clone)]
pub enum KeyState {
    Pressed,
    Released
}

#[derive(Clone)]
pub enum GamepadButtonState {
    Pressed,
    Released,
    Repeated
}

#[derive(Clone)]
pub enum TouchState {
    Started,
    Moved,
    Ended,
    Cancelled,
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
    KeyboardInput {scancode: u32, state: KeyState },
    ModifiersChanged (ModifiersState),
    CursorMoved { x: f64, y: f64 },
    CursorEntered,
    CursorLeft,
    MouseWheel { delta_x: f64, delta_y: f64, state: TouchState },
    MouseScroll { delta_x: f32, delta_y: f32, state: TouchState },
    MouseInput { scancode: u32, state: KeyState },

    //gilrs Events todo
    GamepadInput {id: GamepadId, scancode: u32, state: GamepadButtonState},
    GamepadInputChanged {id: GamepadId, scancode: u32, value: f32},
    GamepadAxis {id: GamepadId, scancode: u32, value: f32},
    GamepadConnected {id: GamepadId},
    GamepadDisconnected {id: GamepadId},
    GamepadDropped {id: GamepadId}
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

    pub fn push(&mut self, input_stack: bool, callback: impl FnMut(&Event) -> bool + 'a) -> usize
    {
        if input_stack {
            self.input_stack.push(Box::new(callback));
            return self.input_stack.len()-1;
        }
        else {
            self.app_stack.push(Box::new(callback));
            return self.app_stack.len()-1;
        } 
    }

    pub fn swap(&mut self, lhs: usize, rhs: usize)
    {
        self.input_stack.swap(lhs, rhs);
    }

    pub fn propagate_event(&mut self, input_stack: bool, event: &Event) -> bool
    {
        if input_stack {
            for callback in self.input_stack.iter_mut().rev()
            {
                if (callback)(event)
                {
                    return true;
                }
            }
        }
        else {
            for callback in self.input_stack.iter_mut()
            {
                if !(callback)(event)
                {
                    log::error!("Error while processing event. Application layer returned false.");
                }
            }
        }

        return false;
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