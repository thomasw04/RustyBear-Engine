use serde::{Serialize, Deserialize};
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::{PhysicalSize, LogicalPosition}};
use winit_fullscreen::WindowFullScreen;

#[derive(Serialize, Deserialize)]
pub struct WindowConfig {
    pub size: (u32, u32),
    pub title: String,
    pub position: (f64, f64),
    pub resizeable: bool,
    pub fullscreen: bool,
    pub visible: bool,
    pub border: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig { title: "RustyBear-Sandbox".to_string(), size: (1280, 720), position: (0.0, 0.0), resizeable: true, fullscreen: false, visible: true, border: true }
    }
}


pub struct Window {
    pub native: winit::window::Window,
    pub event_loop: winit::event_loop::EventLoop<()>,
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

        #[cfg(target_arch = "wasm32")]
        {
    
            window.set_inner_size(PhysicalSize{width: window_config.size.0, height: window_config.size.1});
    
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("rusty-bear-engine")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to the document body.");
        }

        Window { native: window, event_loop }
    }
}