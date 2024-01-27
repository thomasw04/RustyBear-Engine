use std::sync::Arc;

use sysinfo::{System, SystemExt};
use wgpu::{rwh::{HasDisplayHandle, HasRawDisplayHandle}, PresentMode, TextureFormatFeatureFlags};
use winit::{event::{WindowEvent, Event}, event_loop::EventLoopWindowTarget, dpi::PhysicalSize, keyboard::{Key, NamedKey}};
use crate::{window::Window, core::{ModuleStack, Application}, utils::Timestep, event, input::InputState, environment::config::Config};

pub struct Features {
    pub texture_features: wgpu::TextureFormatFeatureFlags
} 

pub struct VisContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,
}

pub struct Context<'a> {
    pub graphics: Arc<VisContext>,
    pub surface: wgpu::Surface<'a>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub features: Features,
    pub egui: egui_winit::State,
    pub config: Config,
    pub sysinfo: System,
}

impl<'a> Context<'a> {
    pub async fn new(window: Arc<winit::window::Window>, config: Config) -> Context<'a> {
        let sysinfo = System::new_with_specifics(sysinfo::RefreshKind::new().with_memory());

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor 
        {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            ..Default::default()
        });

        let surface: wgpu::Surface<'static> = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities.formats.iter()
        .copied().find(|f| f.is_srgb()).unwrap_or(capabilities.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let texture_features = adapter.get_texture_format_features(format).flags;
        let mut features = Features { texture_features };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: Context::activated_features(adapter.features()),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            }, None,
        ).await.unwrap();

        if !device.features().contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
            features.texture_features.set(TextureFormatFeatureFlags::MULTISAMPLE_X2, false);
            features.texture_features.set(TextureFormatFeatureFlags::MULTISAMPLE_X8, false);
            features.texture_features.set(TextureFormatFeatureFlags::MULTISAMPLE_X16, false);
        }

        surface.configure(&device, &surface_config);

        //Create new egui context.
        let egui = egui::Context::default();
        let viewport_id = egui.viewport_id();
        let egui = egui_winit::State::new(egui, viewport_id, &window, Some(window.scale_factor() as f32), None);

        Context { graphics: Arc::new(VisContext { device, queue, format }), surface, surface_config, features, egui, config, sysinfo }
    }

    fn activated_features(supported_features: wgpu::Features) -> wgpu::Features
    {
        let mut activated_features: wgpu::Features = wgpu::Features::empty();

        if supported_features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
            activated_features |= wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        }
        
        activated_features
    }

    pub fn run(mut self, mut app: impl Application<'a> + 'static, window: Window)
    {
        let mut gilrs = gilrs::Gilrs::new().unwrap();

        //Register an EventSubscriber which maintains a list of current KeyStates.
        let input_state = rccell::RcCell::new(InputState::new());
        app.get_stack().subscribe(event::EventType::App, input_state.clone());

       //Time since last frame
        let mut ts = Timestep::default();

        let _ = window.event_loop.run(enclose! { (input_state) move |event, window_target|
        {
            let _handled = match event
            {
                Event::WindowEvent { window_id, ref event }

                if window_id == window.native.id() => 
                {
                    self.egui.on_window_event(&window.native, &event);

                    match event {
                        WindowEvent::Resized(new_size) => {
                            self.resize(*new_size);
                        },
                        /*WindowEvent::ScaleFactorChanged { new_inner_size, ..} => {
                            self.resize(**new_inner_size);
                        },*/
                        WindowEvent::RedrawRequested => {
                            app.update(ts.step_fwd(), input_state.borrow(), &mut self);
                            self.egui.take_egui_input(&window.native);

                            match self.render(&window.native, &mut app) {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => { self.resize(PhysicalSize { width: self.surface_config.width, height: self.surface_config.height }); },
                                Err(wgpu::SurfaceError::OutOfMemory) => { window_target.exit(); },
                                Err(e) => 
                                { 
                                    log::error!("{:?}", e);
                                },
                            }
                        }
                        _ => {}
                    }

                    Context::dispatch_event(app.get_stack(), &window.native, event, window_target, &mut self);
                    app.on_event(&event::to_event(event), &mut self)
                },

                Event::AboutToWait => {
                    window.native.request_redraw();
                    false
                },
                _ => {false}
            };

            let gilrs_event_option = gilrs.next_event();

            if let Some(gilrs_event) = gilrs_event_option {
                Context::dispatch_gamepad_event(app.get_stack(), &gilrs_event, window_target, &mut self);
            }
        }});
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;

            self.surface.configure(&self.graphics.device, &self.surface_config);
        }
    }

    fn render(&mut self, window: &winit::window::Window, app: &mut impl Application<'a>) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        //self.egui.begin_frame();
        app.gui_render(&view, self);

        app.render(&view, self, window);

        output.present();
        Ok(())
    }

    pub fn set_vsync(&mut self, vsync: bool)
    {
        match vsync {
            true => self.surface_config.present_mode = PresentMode::AutoVsync,
            false => self.surface_config.present_mode = PresentMode::AutoNoVsync,
        }
       
        self.surface.configure(&self.graphics.device, &self.surface_config);
    }

    pub fn vsync(&self) -> bool
    {
        self.surface_config.present_mode == PresentMode::AutoVsync
    }

    //These wrapper are just making the code structure more logical in my opinion.
    fn dispatch_event(apps: &mut ModuleStack, window: &winit::window::Window, event: &WindowEvent, window_target: &EventLoopWindowTarget<()>, context: &mut Context) -> bool
    {
        let return_value = apps.dispatch_event(event::EventType::Layer, &event::to_event(event), context);

        if *event == WindowEvent::CloseRequested || *event == WindowEvent::Destroyed {
            window_target.exit();
        }

        if let WindowEvent::KeyboardInput { event, .. } = event {
            if let Key::Named(NamedKey::Escape) = event.logical_key {
                window_target.exit();
            }
        }

        if let WindowEvent::MouseInput { device_id: _, state, button } = *event {
            if button == winit::event::MouseButton::Right && state == winit::event::ElementState::Pressed {
                window.set_cursor_visible(false);
                //window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
            } else if button == winit::event::MouseButton::Right && state == winit::event::ElementState::Released {
                //window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
            }
        }

        return_value
    }

    pub fn free_memory(&self) -> u64 
    {
        self.sysinfo.free_memory()
    }

    fn dispatch_gamepad_event(apps: &mut ModuleStack, event: &gilrs::Event, _window_target: &EventLoopWindowTarget<()>, context: &mut Context) -> bool
    {
        apps.dispatch_event(event::EventType::Layer, &event::to_gamepad_event(event), context)
    }
}