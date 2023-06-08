use serde::{Serialize, Deserialize};
use winit::{event_loop::{EventLoop, ControlFlow}, window::{WindowBuilder}, dpi::{PhysicalSize, LogicalPosition}, event::{Event, WindowEvent, ElementState, MouseScrollDelta, TouchPhase, MouseButton}};
use winit_fullscreen::WindowFullScreen;

use crate::{core::ModuleStack, event::{self, KeyState, TouchState}};

#[derive(Serialize, Deserialize)]
pub struct WindowConfig {
    title: String,
    size: (u32, u32),
    position: (f64, f64),
    resizeable: bool,
    fullscreen: bool,
    visible: bool,
    border: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig { title: "RustyBear-Sandbox".to_string(), size: (1280, 720), position: (0.0, 0.0), resizeable: true, fullscreen: false, visible: true, border: true }
    }
}


pub struct Window {
    pub native: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>
}

impl Window {

    pub fn new(config_json: String) -> Window
    {
        let json_unchecked = serde_json::from_str(&config_json);
        
        if json_unchecked.is_err() {
            log::error!("Failed to parse window config. Defaulting...");
        }
        
        let window_config: WindowConfig = json_unchecked.unwrap_or(Default::default());

        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title(window_config.title)
        .with_inner_size(PhysicalSize{width: window_config.size.0, height: window_config.size.1})
        .with_position(LogicalPosition{x: window_config.position.0, y: window_config.position.1})
        .with_resizable(window_config.resizeable)
        .with_visible(window_config.visible)
        .with_decorations(window_config.border).build(&event_loop).unwrap();

        if window.fullscreen().is_some() ^ window_config.fullscreen {
            window.toggle_fullscreen();
        }

        Window { native: window, event_loop: event_loop }
    }

    pub fn dispatch_event(apps: &mut ModuleStack, event: &WindowEvent, control_flow: &mut ControlFlow)
    {
        match event {
            WindowEvent::Resized(size) => {
                apps.dispatch_event(true, &event::Event::Resized { width: size.width, height: size.height })
            },
            WindowEvent::Moved(pos) => {
                apps.dispatch_event(true, &event::Event::Moved { x: pos.x, y: pos.y })
            },
            WindowEvent::CloseRequested => {
                apps.dispatch_event(true, &event::Event::CloseRequested);
                control_flow.set_exit();
            },
            WindowEvent::Destroyed => {
                apps.dispatch_event(true, &event::Event::Destroyed);
                control_flow.set_exit();
            },
            WindowEvent::DroppedFile(path) => {
                apps.dispatch_event(true, &event::Event::DroppedFile(path.clone()))
            },
            WindowEvent::HoveredFile(path) => {
                apps.dispatch_event(true, &event::Event::HoveredFile(path.clone()))
            },
            WindowEvent::HoveredFileCancelled => {
                apps.dispatch_event(true, &event::Event::HoveredFileCancelled)
            },
            WindowEvent::ReceivedCharacter(ch) => {
                apps.dispatch_event(true, &event::Event::ReceivedCharacter(*ch))
            },
            WindowEvent::Focused( focused) => {
                apps.dispatch_event(true, &event::Event::Focused(*focused))
            },
            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                
                let key_state = match input.state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                };

                apps.dispatch_event(true, &event::Event::KeyboardInput { scancode: input.scancode, state: key_state })
            },
            WindowEvent::ModifiersChanged( state ) => {
                apps.dispatch_event(true, &event::Event::ModifiersChanged(*state))
            },
            WindowEvent::CursorMoved { device_id, position, modifiers } => {
                apps.dispatch_event(true, &event::Event::CursorMoved { x: position.x, y: position.y })
            },
            WindowEvent::CursorEntered { device_id } => {
                apps.dispatch_event(true, &event::Event::CursorEntered)
            },
            WindowEvent::CursorLeft { device_id } => {
                apps.dispatch_event(true, &event::Event::CursorLeft)
            },
            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {

                let touch_state = match phase {
                    TouchPhase::Started => TouchState::Started,
                    TouchPhase::Moved => TouchState::Moved,
                    TouchPhase::Ended => TouchState::Ended,
                    TouchPhase::Cancelled => TouchState::Cancelled,
                };
                
                match delta {
                    MouseScrollDelta::PixelDelta( d) => {
                        apps.dispatch_event(true, &event::Event::MouseWheel { delta_x: d.x, delta_y: d.y, state: touch_state})
                    },
                    MouseScrollDelta::LineDelta(x, y) => {
                        apps.dispatch_event(true, &event::Event::MouseScroll { delta_x: *x, delta_y: *y, state: touch_state})
                    }
                }  
            },
            WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                
                let key_state = match state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                };

                let button_code: u32 = match button {
                    MouseButton::Left => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Right => 3,
                    _ => 0,
                };

                apps.dispatch_event(true, &&event::Event::MouseInput { scancode: button_code, state: key_state })
            }

            _ => {}
        }
    }
}