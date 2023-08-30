use wgpu::{TextureFormatFeatureFlags, PresentMode};
use winit::{event::{WindowEvent, Event}, event_loop::ControlFlow, dpi::PhysicalSize};
use crate::{window::Window, core::{ModuleStack, Application}, utils::Timestep, event, input::InputState};

pub struct Features {
    pub texture_features: wgpu::TextureFormatFeatureFlags
} 

pub struct Context {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub features: Features,
}

impl<'a> Context {
    pub async fn new(window: &mut Window) -> Context {

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor 
        {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(), 
        });

        let surface = unsafe {
            instance.create_surface(&window.native)
        }.unwrap();

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.native.inner_size().width,
            height: window.native.inner_size().height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        let texture_features = adapter.get_texture_format_features(format).flags;
        let mut features = Features { texture_features };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: Context::activated_features(adapter.features()),
                limits: if cfg!(target_arch = "wasm32") {
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

        surface.configure(&device, &config);

        Context { surface, device, queue, config, features }
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
        let mut ts = Timestep::new();

        window.event_loop.run(enclose! { (input_state) move |event, _, control_flow|
        {
            let _handled = match event
            {
                Event::WindowEvent { window_id, ref event }

                if window_id == window.native.id() => 
                {
                    match event {
                        WindowEvent::Resized(new_size) => {
                            self.resize(*new_size);
                        },
                        WindowEvent::ScaleFactorChanged { new_inner_size, ..} => {
                            self.resize(**new_inner_size);
                        },
                        _ => {}
                    }

                    Context::dispatch_event(app.get_stack(), event, control_flow, &mut self);
                    app.on_event(&event::to_event(event), &mut self)
                },

                Event::RedrawRequested(window_id)

                if window_id == window.native.id() =>
                {
                    app.update(ts.step_fwd(), input_state.borrow(), &mut self);

                    match self.render(&mut app) {
                        Ok(_) => {true}
                        Err(wgpu::SurfaceError::Lost) => { self.resize(PhysicalSize { width: self.config.width, height: self.config.height }); false},
                        Err(wgpu::SurfaceError::OutOfMemory) => { *control_flow = ControlFlow::Exit; true},
                        Err(e) => 
                        { 
                            log::error!("{:?}", e);
                            true
                        },
                    }
                },

                Event::MainEventsCleared => {
                    window.native.request_redraw();
                    false
                },
                _ => {false}
            };

            let gilrs_event_option = gilrs.next_event();

            if let Some(gilrs_event) = gilrs_event_option {
                Context::dispatch_gamepad_event(app.get_stack(), &gilrs_event, control_flow, &mut self);
            }
        }});
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>)
    {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self, app: &mut impl Application<'a>) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        app.render(view, self);
        output.present();
        Ok(())
    }

    pub fn set_vsync(&mut self, vsync: bool)
    {
        match vsync {
            true => self.config.present_mode = PresentMode::AutoVsync,
            false => self.config.present_mode = PresentMode::AutoNoVsync,
        }
       
       self.surface.configure(&self.device, &self.config);
    }

    pub fn vsync(&self) -> bool
    {
        self.config.present_mode == PresentMode::AutoVsync
    }

    //These wrapper are just making the code structure more logical in my opinion.
    fn dispatch_event(apps: &mut ModuleStack, event: &WindowEvent, control_flow: &mut ControlFlow, context: &mut Context) -> bool
    {
        let return_value = apps.dispatch_event(event::EventType::Layer, &event::to_event(event), context);

        if *event == WindowEvent::CloseRequested || *event == WindowEvent::Destroyed {
            control_flow.set_exit();
        }

        return_value
    }

    fn dispatch_gamepad_event(apps: &mut ModuleStack, event: &gilrs::Event, control_flow: &mut ControlFlow, context: &mut Context) -> bool
    {
        apps.dispatch_event(event::EventType::Layer, &event::to_gamepad_event(event), context)
    }
}