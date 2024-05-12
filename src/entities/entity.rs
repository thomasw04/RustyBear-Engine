use hecs::{ComponentError, NoSuchEntity};

pub trait Entity {
    fn handle(&self) -> hecs::Entity;
    fn world(&self) -> &hecs::World;
    fn world_mut(&mut self) -> &mut hecs::World;

    fn add_component<T: hecs::Component>(&mut self, component: T) -> Result<(), NoSuchEntity> {
        let handle = self.handle();
        self.world_mut().insert_one(handle, component)
    }

    fn remove_component<T: hecs::Component>(&mut self) -> Result<T, ComponentError> {
        let handle = self.handle();
        self.world_mut().remove_one::<T>(handle)
    }

    fn get_component<'a, T: hecs::ComponentRef<'a>>(&'a self) -> Result<T::Ref, ComponentError> {
        self.world().get::<T>(self.handle())
    }
}
