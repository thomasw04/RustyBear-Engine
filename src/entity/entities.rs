use hashbrown::HashMap;

use crate::utils::{Guid, GuidGenerator};

//A collection of entities that represents a set of worlds.
pub struct Worlds {
    worlds: HashMap<Guid, hecs::World>,
    generator: GuidGenerator,
    current_world: Option<Guid>,
}

impl Default for Worlds {
    fn default() -> Self {
        Self { worlds: HashMap::new(), generator: GuidGenerator::new(), current_world: None }
    }
}

impl Worlds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_world(&mut self, world: hecs::World) -> Guid {
        let guid = self.generator.generate();
        self.worlds.insert(guid, world);
        guid
    }

    pub fn get_world(&mut self, guid: Guid) -> Option<&hecs::World> {
        self.worlds.get(&guid)
    }

    pub fn get_mut(&mut self) -> Option<&mut hecs::World> {
        if let Some(guid) = self.current_world {
            self.worlds.get_mut(&guid)
        } else {
            None
        }
    }

    pub fn get(&mut self) -> Option<&hecs::World> {
        if let Some(guid) = self.current_world {
            self.worlds.get(&guid)
        } else {
            None
        }
    }

    pub fn start_world(&mut self, guid: Guid) {
        self.current_world = Some(guid);
    }
}
