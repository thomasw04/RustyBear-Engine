
pub struct Window {
    pub native: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>
}

impl Window {

    pub fn new(native: winit::window::Window, event_loop: winit::event_loop::EventLoop<()>) -> Window
    {
        Window { native: native, event_loop: event_loop }
    }
}