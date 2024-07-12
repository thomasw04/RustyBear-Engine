use legion::{
    query::{IntoQuery, IntoView, ReadOnly},
    storage::{Component, IntoComponentSource, KnownLength},
    Resources,
};
// -------------------------------------
// World System.
// Abstraction layer for managing state.
// -------------------------------------
use std::{
    cell::{Ref, UnsafeCell},
    ops::DerefMut,
};

use crate::{input::InputState, utils::Timestep};

pub trait Scriptable {
    fn on_spawn(&mut self, entity: Entity);
    fn tick(&mut self, delta: &Timestep, input_state: &Ref<InputState>);
    fn on_destroy(&mut self, entity: Entity);
}

pub type Script = Box<dyn Scriptable + Send + Sync>;

pub trait Entable {
    fn handle(&self) -> Entity;
    fn world(&self) -> &World;
    fn cmds(&self) -> &CommandBuffer;
    fn cmds_mut(&mut self) -> &mut CommandBuffer;

    fn spawn<T>(&mut self, components: T) -> Entity
    where
        Option<T>: 'static + IntoComponentSource,
        <Option<T> as IntoComponentSource>::Source: KnownLength + Send + Sync,
    {
        let entity = self.cmds_mut().0.push(components);
        let handle = self.handle().into();

        self.exec_mut(move |world, _| {
            if let Some(mut ent) = world.entry(handle) {
                if let Ok(com) = ent.get_component_mut::<Script>() {
                    com.on_spawn(handle.into());
                }
            }
        });

        entity.into()
    }

    fn despawn(&mut self, entity: Entity) {
        self.exec_mut(move |world, _| {
            if let Some(mut ent) = world.entry(entity.into()) {
                if let Ok(com) = ent.get_component_mut::<Script>() {
                    com.on_destroy(entity);
                }
            }
        });

        self.cmds_mut().0.remove(entity.into());
    }

    fn add<T: Component>(&mut self, com: T) {
        let handle = self.handle().into();
        self.cmds_mut().0.add_component(handle, com);
    }

    fn add_by_entity<T: Component>(&mut self, entity: Entity, com: T) {
        self.cmds_mut().0.add_component(entity.into(), com);
    }

    fn remove<T: Component>(&mut self) {
        let handle = self.handle().into();
        self.cmds_mut().0.remove_component::<T>(handle)
    }

    fn remove_by_entity<T: Component>(&mut self, entity: Entity) {
        self.cmds_mut().0.remove_component::<T>(entity.into())
    }

    fn exec_mut<F>(&mut self, func: F)
    where
        F: 'static + Fn(&mut legion::World, &mut Resources) + Send + Sync,
    {
        self.cmds_mut().0.exec_mut(func);
    }

    fn for_each<Q: IntoQuery + Send + Sync>(
        &self, func: impl Fn(<<Q as IntoView>::View as legion::query::View<'_>>::Element),
    ) where
        Q::View: ReadOnly,
    {
        unsafe { self.world().for_each::<Q>(func) }
    }

    fn get<Q: IntoQuery + Send + Sync>(
        &self,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        unsafe { self.world().query_get::<Q>(self.handle()) }
    }

    fn get_by_entity<Q: IntoQuery + Send + Sync>(
        &self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        unsafe { self.world().query_get::<Q>(entity) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Entity(Option<legion::Entity>);

impl From<legion::Entity> for Entity {
    fn from(value: legion::Entity) -> Self {
        Self(Some(value))
    }
}

impl From<Entity> for legion::Entity {
    fn from(value: Entity) -> Self {
        match value.0 {
            Some(x) => x,
            None => panic!("Invalid entity identifier."),
        }
    }
}

// --------------------------------------------------------------

pub struct CommandBuffer(legion::systems::CommandBuffer);

impl Default for CommandBuffer {
    fn default() -> Self {
        panic!("This is a bug. CommandBuffer cannot be created without a world. This implementation is necessary, although should be never called!");
    }
}

impl CommandBuffer {
    pub fn new(world: &legion::World) -> Self {
        Self(legion::systems::CommandBuffer::new(world))
    }
}

// --------------------------------------------------------------

pub struct World(UnsafeCell<legion::World>);

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl From<UnsafeCell<legion::World>> for World {
    fn from(value: UnsafeCell<legion::World>) -> Self {
        Self(value)
    }
}

impl World {
    pub fn new() -> Self {
        Self(UnsafeCell::new(legion::World::default()))
    }

    /// # Safety
    /// Creates a mutable reference to the world.
    /// This reference is only allowed to exist while no other reference (mutable or not) is alive.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn inner_mut(&self) -> &mut legion::World {
        self.0.get().as_mut().expect("This is a bug. World is not initialized.")
    }

    /// # Safety
    /// Creates a reference to the world.
    /// This reference is only allowed to exist while no other mutable reference is alive.
    pub unsafe fn inner(&self) -> &legion::World {
        self.0.get().as_ref().expect("This is a bug. World is not initialized.")
    }

    /// # Safety
    /// Creates a mutable reference to the world.
    /// This reference is only allowed to exist while no other reference (mutable or not) is alive.
    pub unsafe fn apply(&self, cmds: &mut CommandBuffer) {
        let world = self.inner_mut();

        cmds.0.flush(world, &mut legion::Resources::default());
    }

    /// # Safety
    /// Creates a reference to the world.
    /// This reference is only allowed to exist while no other mutable reference is alive.
    pub unsafe fn is_alive(&self, entity: Entity) -> bool {
        let world = self.inner();
        world.contains(entity.into())
    }

    /// # Safety
    /// Creates a reference to the world.
    /// This reference is only allowed to exist while no other mutable reference is alive.
    pub unsafe fn for_each<Q: IntoQuery + Send + Sync>(
        &self, func: impl Fn(<<Q as IntoView>::View as legion::query::View<'_>>::Element),
    ) where
        Q::View: ReadOnly,
    {
        let world = self.inner();
        <Q>::query().for_each_unchecked(world, func);
    }

    /// # Safety
    /// Creates a reference to the world.
    /// This reference is only allowed to exist while no other mutable reference is alive.
    pub unsafe fn query_get<Q: IntoQuery + Send + Sync>(
        &self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        let world = self.inner();
        <Q>::query().get_unchecked(world, entity.into()).ok()
    }

    pub fn tick(&mut self, delta: &Timestep, input_state: &Ref<InputState>) {
        let mut world = self.0.get_mut();
        for script in <&mut Script>::query().iter_mut(world.deref_mut()) {
            script.tick(delta, input_state);
        }
    }
}
