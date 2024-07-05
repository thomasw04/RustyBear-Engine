use std::borrow::Borrow;
use std::cell::Ref;
use std::ops::Deref;

use crate::input::InputState;
use crate::utils::Timestep;
use derive::Entable;

use super::world::{Entable, Entity, Script, World};

#[Entable]
pub struct Player {
    x: f32,
    y: f32,
}

fn test() {
    let mut world = World::new();
    let handle = world.spawn(());

    let player = Player::instantiate(handle.into(), World::new());

    player.for_each::<&Script>(|_| println!("AMOGUS!"));
}

pub fn tick_scripts(world: &mut World, delta: &Timestep, input_state: &Ref<InputState>) {
    world.for_each_mut::<&mut Script>(|script| {
        script.tick(delta, input_state);
    });
}
