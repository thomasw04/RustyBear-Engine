use std::cell::Ref;
use std::ops::Deref;

use crate::entities::world::Entity;
use crate::input::InputState;
use crate::utils::Timestep;
use derive::Entity;
use legion::IntoQuery;

use super::world::{Script, Scriptable, World};

use crate::entities::world::Entable;

#[Entity]
pub struct Player {
    x: f32,
    y: f32,
}

impl Scriptable for Player<'_> {
    fn on_spawn(&mut self, entity: Entity) {
        println!("Player spawned: {:?}", entity);
    }

    fn tick(&mut self, _delta: &Timestep, _input_state: &Ref<InputState>) {
        self.iter::<()>().for_each(|(entity, player)| {
            println!("Player: {:?}", entity);
        });
    }

    fn on_destroy(&mut self, entity: Entity) {}
}

fn test() {
    let mut world = World::new();
    world.instantiate::<Player>(());
}

pub fn tick_scripts(world: &mut World, delta: &Timestep, input_state: &Ref<InputState>) {
    for (entity, mut script) in world.iter::<&mut Script>() {
        script.tick(delta, input_state);
    }
}
