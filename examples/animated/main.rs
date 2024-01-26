#![allow(non_snake_case)]

use std::cell::Ref;
use std::path::Path;

use glam::{Vec2, Vec3, Vec4};
use hecs::World;
use rccell::RcCell;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::keyboard::KeyCode;
use RustyBear_Engine::assets::assets::Assets;
use RustyBear_Engine::context::{Context, VisContext};
use RustyBear_Engine::core::{Application, ModuleStack};
use RustyBear_Engine::entity::desc::{Sprite, Transform2D};
use RustyBear_Engine::entity::entities::Worlds;
use RustyBear_Engine::entity::script::{Scriptable, Scripts};
use RustyBear_Engine::environment::config::Config;
use RustyBear_Engine::event::{Event, EventType};
use RustyBear_Engine::input::InputState;
use RustyBear_Engine::logging;
use RustyBear_Engine::render::camera::OrthographicCamera;
use RustyBear_Engine::render::render2d::{RenderData, Renderer2D};
use RustyBear_Engine::utils::Timestep;
use RustyBear_Engine::window::Window;

pub struct AnimatedApp<'a> {
    stack: ModuleStack<'a>,
    assets: Assets,
    worlds: Worlds,
    scripts: Scripts,
    renderer: RcCell<Renderer2D>,
    camera: RcCell<OrthographicCamera>,
}

impl<'a> Application<'a> for AnimatedApp<'a> {
    fn on_event(&mut self, _event: &Event, _context: &mut Context) -> bool { false }

    fn render(
        &mut self, view: &wgpu::TextureView, context: &mut Context, window: &winit::window::Window,
    ) {
        {
            let mut renderer = self.renderer.borrow_mut();

            renderer.update_camera_buffer(
                &context.graphics,
                self.camera.borrow_mut().view_projection().to_cols_array_2d(),
            );

            let render_data = RenderData { ctx: context, view, window };

            renderer.render(render_data, &mut self.assets, &mut self.worlds);
        }
    }

    fn gui_render(
        &mut self, _view: &wgpu::TextureView, _context: &mut Context, _gui_context: &egui::Context,
    ) {
    }

    fn update(&mut self, delta: &Timestep, input_state: Ref<InputState>, context: &mut Context) {
        if let Some(world) = self.worlds.get_mut() {
            self.scripts.tick(&context.graphics, delta, world);
        }

        let mut cam = self.camera.borrow_mut();

        if input_state.is_key_down(&KeyCode::KeyA) {
            cam.inc_pos(Vec2::new(-(0.1 * delta.norm()), 0.0));
        }

        if input_state.is_key_down(&KeyCode::KeyD) {
            cam.inc_pos(Vec2::new(0.1 * delta.norm(), 0.0));
        }

        if input_state.is_key_down(&KeyCode::Space) {
            cam.inc_pos(Vec2::new(0.0, 0.1 * delta.norm()));
        }

        if input_state.is_key_down(&KeyCode::ShiftLeft) {
            cam.inc_pos(Vec2::new(0.0, -(0.1 * delta.norm())));
        }
    }

    fn quit(&mut self) {}

    fn get_stack(&mut self) -> &mut ModuleStack<'a> { &mut self.stack }
}

struct Animator {
    delta: f64,

    total_frames: usize,
    current_frame: usize,
    fps: usize,
}

impl Scriptable for Animator {
    fn on_spawn(&mut self, _context: &VisContext, _entity: hecs::Entity, _world: &mut World) {}

    fn tick(
        &mut self, context: &VisContext, entity: hecs::Entity, delta: &Timestep,
        world: &mut hecs::World,
    ) {
        if let Ok(mut sprite) = world.get::<&mut Sprite>(entity) {
            if self.delta > 1000.0 / self.fps as f64 {
                sprite.set_coords_quad(
                    context,
                    Vec2::new((1.0 / self.total_frames as f32) * (self.current_frame as f32), 0.0),
                    Vec2::new(
                        (1.0 / self.total_frames as f32) * (self.current_frame as f32 + 1.0),
                        1.0,
                    ),
                );

                self.current_frame = (self.current_frame + 1) % self.total_frames;

                self.delta = 0.0;
            } else {
                self.delta += delta.millis();
            }
        }
    }

    fn on_destroy(&mut self, _context: &VisContext, _entity: hecs::Entity, _world: &mut World) {}
}

impl<'a> AnimatedApp<'a> {
    pub fn new(context: &Context) -> Self {
        log::info!("Init Application");

        let mut stack = ModuleStack::new();

        let loc = context.config.project_config().location.clone().map(what::Location::File);

        if let Some(what::Location::File(path)) = &loc {
            log::warn!("Project: {:?}", path);
        }

        let mut scripts = Scripts::new();
        let mut assets =
            Assets::new(context.graphics.clone(), loc, (context.free_memory() / 2) as usize);

        let mut worlds = Worlds::new();

        let mut default = World::new();

        let default_texture = assets.request_asset("data/amogus.fur", 0);
        let player_script = Animator { delta: 0.0, total_frames: 60, current_frame: 0, fps: 30 };
        let player_script = scripts.add_script(Box::new(player_script));

        let trans = Transform2D::new(&context.graphics, Vec3::new(0.0, 0.0, 1.0), 0.0, Vec2::ONE);

        let player = default
            .spawn((trans, Sprite::new(&context.graphics, default_texture, Vec4::ONE, None)));

        scripts.attach(player_script, player);

        let default = worlds.add_world(default);
        worlds.start_world(default);

        let renderer = RcCell::new(Renderer2D::new(context, &mut assets));
        stack.subscribe(EventType::Layer, renderer.clone());

        let camera = RcCell::new(OrthographicCamera::default());
        stack.subscribe(EventType::Layer, camera.clone());

        camera.borrow_mut().set_aspect_ratio(
            context.surface_config.width as f32 / context.surface_config.height as f32,
        );

        AnimatedApp { stack, assets, scripts, worlds, renderer, camera }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    logging::init();
    println!();

    //Create the config and init the example project.
    let mut config = Config::new(None);
    config.find_project(Path::new("examples/animated")).unwrap();

    //Create the window from the config and create the context.
    let mut window = Window::new("{}".to_string());
    window.native.set_ime_allowed(true);
    window.native.set_cursor_visible(false);

    let context = pollster::block_on(Context::new(&mut window, config));

    //Create and init the application
    let myapp = AnimatedApp::new(&context);

    //Move my app and window into the context. And run the app.
    context.run(myapp, window);
}
