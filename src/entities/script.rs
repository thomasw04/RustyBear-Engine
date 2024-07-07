use std::cell::Ref;
use std::ops::Deref;

use crate::input::InputState;
use crate::utils::Timestep;
use derive::Entity;
use legion::IntoQuery;

use super::world::{Script, World};

use crate::entities::world::Entable;

#[Entity]
pub struct Player {
    x: f32,
    y: f32,
}

fn test() {
    let world = World::new();
    let handle = unsafe { world.inner_mut().push(()) };

    let player = Player::instantiate(handle.into(), &world);

    player.for_each::<&Script>(|_| println!("AMOGUS!"));
}

pub fn tick_scripts(world: &mut World, delta: &Timestep, input_state: &Ref<InputState>) {
    <&mut Script>::query().for_each_mut(unsafe { world.inner_mut() }, |script| {
        script.tick(delta, input_state);
    });
}
