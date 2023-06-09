
use crate::utils::Timestep;
use crate::window::Window;
use crate::event::{EventSubscriber, EventStack, Event, EventType};

use rccell::RcCell;
use std::string::String;

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

pub trait Application {
    fn init(&mut self, config_json: String, stack: &mut ModuleStack) -> Window;
    fn update(&mut self, delta: &Timestep);
    fn quit(&mut self);
}

pub struct ModuleStack<'a> {
    events: EventStack<'a>,
    modules: Vec<Box<dyn Module + 'a>>,
}

impl<'a> ModuleStack<'a> {

    pub fn new() -> ModuleStack<'a>
    {
        ModuleStack { events: EventStack::new(), modules: Vec::new() }
    }

    pub fn push(&mut self, module: impl Module + 'a)
    {
        self.modules.push(Box::new(module));
    }

    fn update(&mut self, ts: &Timestep)
    {
        for mods in &mut self.modules
        {
            mods.update(ts);
        }
    } 

    pub fn dispatch_event(&mut self, event_type: EventType, event: &Event) -> bool
    {
        match event_type {
            EventType::App => {
                self.events.propagate_app_event(event)
            },
            EventType::Layer => {
                self.events.propagate_event(event)
            }
        }
    }

    pub fn subscribe(&mut self, event_type: EventType, subscriber: RcCell<impl EventSubscriber + 'a>)
    {
        self.events.push(event_type, enclose! { (subscriber) move |event: &Event| { subscriber.borrow_mut().on_event(event) }});
    }
}

