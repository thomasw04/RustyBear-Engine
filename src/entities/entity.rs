use hecs::{ComponentError, NoSuchEntity};

use super::vecs::{EntityHandle, World};

pub trait Entity {
    fn handle(&self) -> EntityHandle;
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;

    fn add_component<T: hecs::Component>(&mut self, component: T) -> Result<(), NoSuchEntity> {
        let handle = self.handle();
        self.world_mut().insert_one(handle.into(), component)
    }

    fn remove_component<T: hecs::Component>(&mut self) -> Result<T, ComponentError> {
        let handle = self.handle();
        self.world_mut().remove_one::<T>(handle.into())
    }

    fn get_component<'a, T: hecs::ComponentRef<'a>>(&'a self) -> Result<T::Ref, ComponentError> {
        self.world().get::<T>(self.handle())
    }
}
