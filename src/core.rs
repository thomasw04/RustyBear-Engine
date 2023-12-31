use std::cell::Ref;

use crate::context::Context;
use crate::event::{Event, EventStack, EventSubscriber, EventType};
use crate::input::InputState;
use crate::utils::Timestep;

use rccell::RcCell;

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub trait Module {
    fn init(&mut self);
    fn update(&mut self, delta: &Timestep);
    fn quit(&mut self);
}

pub trait Application<'a> {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> bool;
    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    );
    fn gui_render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, gui_context: &egui::Context,
    );
    fn update(&mut self, delta: &Timestep, input_state: Ref<InputState>, context: &mut Context);
    fn quit(&mut self);

    fn get_stack(&mut self) -> &mut ModuleStack<'a>;
}

#[derive(Default)]
pub struct ModuleStack<'a> {
    events: EventStack<'a>,
    modules: Vec<Box<dyn Module + 'a>>,
}

impl<'a> ModuleStack<'a> {
    pub fn new() -> ModuleStack<'a> {
        ModuleStack::default()
    }

    pub fn push(&mut self, module: impl Module + 'a) {
        self.modules.push(Box::new(module));
    }

    #[allow(dead_code)]
    fn update(&mut self, ts: &Timestep) {
        for mods in &mut self.modules {
            mods.update(ts);
        }
    }

    pub fn dispatch_event(
        &mut self, event_type: EventType, event: &Event, context: &mut Context,
    ) -> bool {
        match event_type {
            EventType::App => self.events.propagate_app_event(event, context),
            EventType::Layer => self.events.propagate_event(event, context),
        }
    }

    pub fn subscribe(
        &mut self, event_type: EventType, subscriber: RcCell<impl EventSubscriber + 'a>,
    ) {
        self.events.push(event_type, enclose! { (subscriber) move |event: &Event, context: &mut Context| { subscriber.borrow_mut().on_event(event, context) }});
    }
}
