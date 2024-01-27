use hashbrown::HashMap;

use crate::{context::VisContext, utils::Timestep};

use super::entities;

pub trait Scriptable {
    fn on_spawn(&mut self, context: &VisContext, entity: hecs::Entity, world: &mut hecs::World);
    fn tick(
        &mut self, context: &VisContext, entity: hecs::Entity, delta: &Timestep,
        world: &mut hecs::World,
    );
    fn on_destroy(&mut self, context: &VisContext, entity: hecs::Entity, world: &mut hecs::World);
}

pub struct ScriptHandle {
    id: u64,
}

pub struct Scripts {
    ids: HashMap<u64, u64>,
    scripts: Vec<(Box<dyn Scriptable>, Vec<hecs::Entity>)>,
    id_generator: u64,
}

impl Scripts {
    pub fn new() -> Self {
        Self { ids: HashMap::new(), scripts: Vec::new(), id_generator: 0 }
    }

    pub fn add_script(&mut self, script: Box<dyn Scriptable>) -> ScriptHandle {
        self.id_generator += 1;
        let id = self.id_generator;
        self.scripts.push((script, Vec::new()));
        self.ids.insert(id, self.scripts.len() as u64 - 1);
        ScriptHandle { id }
    }

    pub fn attach(&mut self, script: ScriptHandle, entity: hecs::Entity) {
        if let Some(index) = self.ids.get(&script.id) {
            self.scripts[*index as usize].1.push(entity);
        }
    }

    pub fn detach(&mut self, script: ScriptHandle, entity: hecs::Entity) {
        if let Some(index) = self.ids.get(&script.id) {
            let script = &mut self.scripts[*index as usize];
            script.1.retain(|e| *e != entity);
        }
    }

    pub fn on_spawn(
        &mut self, context: &VisContext, target: hecs::Entity, world: &mut hecs::World,
    ) {
        for (script, entities) in self.scripts.iter_mut() {
            for entity in entities.iter() {
                if *entity == target {
                    script.on_spawn(context, *entity, world);
                }
            }
        }
    }

    pub fn tick(&mut self, context: &VisContext, delta: &Timestep, world: &mut hecs::World) {
        for (script, entities) in self.scripts.iter_mut() {
            for entity in entities.iter() {
                script.tick(context, *entity, delta, world);
            }
        }
    }

    pub fn on_destroy(
        &mut self, context: &VisContext, target: hecs::Entity, world: &mut hecs::World,
    ) {
        for (script, entities) in self.scripts.iter_mut() {
            for entity in entities.iter() {
                if *entity == target {
                    script.on_destroy(context, *entity, world);
                }
            }
        }
    }
}
