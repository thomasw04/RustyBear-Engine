use gilrs::Gilrs;
use log::info;
use winit::{event::{WindowEvent, Event, VirtualKeyCode}, event_loop::ControlFlow, dpi::PhysicalSize};

use crate::{window::Window, core::{ModuleStack, Application}, utils::Timestep, event, input::InputState};

pub struct Context {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
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

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            }, None,
        ).await.unwrap();

        let capabilities = surface.get_capabilities(&adapter);

        let format = capabilities.formats.iter()
        .copied().find(|f| f.is_srgb()).unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: format,
            width: window.native.inner_size().width,
            height: window.native.inner_size().height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        Context { surface: surface, device: device, queue: queue, config: config }
    }

    pub fn run(mut self, mut app: impl Application<'a> + 'static, window: Window)
    {
        let mut gilrs = Gilrs::new().unwrap();

        //Register an EventSubscriber which maintains a list of current KeyStates.
        let input_state = rccell::RcCell::new(InputState::new());
        app.get_stack().subscribe(event::EventType::App, input_state.clone());

       //Time since last frame
        let mut ts = Timestep::new();

        window.event_loop.run(enclose! { (input_state) move |event, _, control_flow|
        {
            if input_state.borrow().is_key_down(&VirtualKeyCode::A) {
                info!("The A is down.");
            }

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

                    Context::dispatch_event(app.get_stack(), event, control_flow)
                },

                Event::RedrawRequested(window_id)

                if window_id == window.native.id() =>
                {
                    app.update(ts.step_fwd());

                    match self.render() {
                        Ok(_) => {true}
                        Err(wgpu::SurfaceError::Lost) => { self.resize(PhysicalSize { width: self.config.width, height: self.config.height }); false},
                        Err(wgpu::SurfaceError::OutOfMemory) => { *control_flow = ControlFlow::Exit; true},
                        Err(e) => { log::error!("{:?}", e); true},
                    }
                },
                _ => {false}
            };

            let gilrs_event_option = gilrs.next_event();

            if gilrs_event_option.is_some() {
                let gilrs_event = gilrs_event_option.unwrap();
                Context::dispatch_gamepad_event(app.get_stack(), &gilrs_event, control_flow);
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        let output = self.surface.get_current_texture()?;

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor 
        {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.7,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    //These wrapper are just making the code structure more logical in my opinion.
    fn dispatch_event(apps: &mut ModuleStack, event: &WindowEvent, control_flow: &mut ControlFlow) -> bool
    {
        Window::dispatch_event(apps, event, control_flow)
    }

    fn dispatch_gamepad_event(apps: &mut ModuleStack, event: &gilrs::Event, control_flow: &mut ControlFlow) -> bool
    {
        Window::dispatch_gamepad_event(apps, event, control_flow)
    }
}