
use std::path::PathBuf;

use gilrs::GamepadId;
use winit::event::{WindowEvent, MouseScrollDelta};

use crate::context::Context;

#[derive(Clone)]
#[derive(PartialEq)]
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
    Unknown,

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
    fn on_event(&mut self, event: &Event, context: &Context) -> bool;
}

type EventCallback<'a> = Box<dyn FnMut(&Event, &Context) -> bool + 'a>;

#[derive(Default)]
pub struct EventStack<'a> {
    
    input_stack: Vec<EventCallback<'a>>,
    app_stack: Vec<EventCallback<'a>>,
}

impl<'a> EventStack<'a> {

    pub fn new() -> EventStack<'a>
    {
        EventStack { input_stack: Vec::new(), app_stack: Vec::new() }
    }

    pub fn push(&mut self, event_type: EventType, callback: impl FnMut(&Event, &Context) -> bool + 'a) -> usize
    {
        match event_type {
            EventType::App => {
                self.app_stack.push(Box::new(callback));

                self.app_stack.len()-1
            },

            EventType::Layer => {
                self.input_stack.push(Box::new(callback));

                self.input_stack.len()-1
            }
        }
    }

    pub fn swap(&mut self, lhs: usize, rhs: usize)
    {
        self.input_stack.swap(lhs, rhs);
    }

    pub fn propagate_event(&mut self, event: &Event, context: &Context) -> bool
    {
        self.propagate_app_event(event, context);

        for callback in self.input_stack.iter_mut().rev()
        {
            if (callback)(event, context)
            {
                return true;
            }
        }

        false
    }

    pub fn propagate_app_event(&mut self, event: &Event, context: &Context) -> bool
    {
        for callback in self.app_stack.iter_mut()
        {
            if !(callback)(event, context)
            {
                log::error!("Error while processing event. Application layer returned false.");
                return false;
            }
        }

        true
    }

    pub fn pop(&mut self) -> usize
    {
        self.input_stack.pop();

        self.input_stack.len()-1
    }

    pub fn remove(&mut self, index: usize)
    {
        let _dying_closure = self.app_stack.remove(index);
    }
}

pub fn to_gamepad_event(event: &gilrs::Event) -> Event
{
    match event.event {
        gilrs::EventType::Connected => {
            Event::GamepadConnected { id: event.id }
        },
        gilrs::EventType::Disconnected => {
            Event::GamepadDisconnected { id: event.id }
        },
        gilrs::EventType::ButtonPressed(button, ..) => {
            Event::GamepadInput { id: event.id, buttoncode: button, state: GamepadButtonState::Pressed }
        },
        gilrs::EventType::ButtonReleased(button, ..) => {
            Event::GamepadInput { id: event.id, buttoncode: button, state: GamepadButtonState::Released }
        },
        gilrs::EventType::ButtonRepeated(button, ..) => {
            Event::GamepadInput { id: event.id, buttoncode: button, state: GamepadButtonState::Repeated }
        },
        gilrs::EventType::ButtonChanged(button, value, ..) => {
            Event::GamepadInputChanged { id: event.id, scancode: button as u32, value }
        },
        gilrs::EventType::AxisChanged(axis, value, ..) => {
            Event::GamepadAxis { id: event.id, axiscode: axis, value }
        },
        _ => { Event::Unknown }
    }
}

pub fn to_event(event: &WindowEvent) -> Event
{
    match event {
        WindowEvent::Resized(size) => {
            Event::Resized { width: size.width, height: size.height }
        },
        WindowEvent::ScaleFactorChanged {new_inner_size, .. } => {
            Event::Resized { width: (**new_inner_size).width, height: (**new_inner_size).height }
        },
        WindowEvent::Moved(pos) => {
            Event::Moved { x: pos.x, y: pos.y }
        },
        WindowEvent::CloseRequested => {
            Event::CloseRequested
        },
        WindowEvent::Destroyed => {
            Event::Destroyed
        },
        WindowEvent::DroppedFile(path) => {
            Event::DroppedFile(path.clone())
        },
        WindowEvent::HoveredFile(path) => {
            Event::HoveredFile(path.clone())
        },
        WindowEvent::HoveredFileCancelled => {
            Event::HoveredFileCancelled
        },
        WindowEvent::ReceivedCharacter(ch) => {
            Event::ReceivedCharacter(*ch)
        },
        WindowEvent::Focused( focused) => {
            Event::Focused(*focused)
        },
        WindowEvent::KeyboardInput { input, .. } => {
            if input.virtual_keycode.is_some() {
                Event::KeyboardInput { keycode: input.virtual_keycode.unwrap(), state: input.state }
            } else {
                Event::Unknown
            }
        },
        WindowEvent::ModifiersChanged( state ) => {
            Event::ModifiersChanged(*state)
        },
        WindowEvent::CursorMoved { position, .. } => {
            Event::CursorMoved { x: position.x, y: position.y }
        },
        WindowEvent::CursorEntered { .. } => {
            Event::CursorEntered
        },
        WindowEvent::CursorLeft { .. } => {
            Event::CursorLeft
        },
        WindowEvent::MouseWheel { delta, phase, .. } => {
            match delta {
                MouseScrollDelta::PixelDelta( d) => {
                    Event::MouseWheel { delta_x: d.x, delta_y: d.y, state: *phase}
                },
                MouseScrollDelta::LineDelta(x, y) => {
                    Event::MouseScroll { delta_x: *x, delta_y: *y, state: *phase}
                }
            }  
        },
        WindowEvent::MouseInput { state, button, .. } => {
            Event::MouseInput { mousecode: *button, state: *state }
        }

        _ => {Event::Unknown}
    }
 }