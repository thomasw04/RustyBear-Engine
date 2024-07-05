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
    cell::{Ref, RefMut, UnsafeCell},
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
    fn world_mut(&mut self) -> &mut World;
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
        self.world().for_each::<Q>(func)
    }

    fn get<'a, Q: IntoQuery + Send + Sync>(
        &'a self,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        self.world().query_get::<Q>(self.handle())
    }

    fn get_mut<'a, Q: IntoQuery + Send + Sync>(
        &'a mut self,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element> {
        let handle = self.handle();
        self.world_mut().query_mut::<Q>(handle)
    }

    fn get_by_entity<'a, Q: IntoQuery + Send + Sync>(
        &'a self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        self.world().query_get::<Q>(entity)
    }

    fn get_by_entity_mut<'a, Q: IntoQuery + Send + Sync>(
        &'a mut self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element> {
        self.world_mut().query_mut::<Q>(entity)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(Option<legion::Entity>);

impl Default for Entity {
    fn default() -> Self {
        Self(None)
    }
}

impl From<legion::Entity> for Entity {
    fn from(value: legion::Entity) -> Self {
        Self(Some(value))
    }
}

impl Into<legion::Entity> for Entity {
    fn into(self) -> legion::Entity {
        match self.0 {
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

    pub fn spawn<T>(&mut self, components: T) -> Entity
    where
        Option<T>: IntoComponentSource,
    {
        let entity = self.0.get_mut().push(components);

        if let Some(mut ent) = self.0.get_mut().entry(entity) {
            if let Ok(com) = ent.get_component_mut::<Script>() {
                com.on_spawn(entity.into());
            }
        }

        entity.into()
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let world =
            unsafe { self.0.get().as_ref().expect("This is a bug. World is not initialized.") };
        world.contains(entity.into())
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.0.get_mut().remove(entity.into())
    }

    pub fn get<'a, T: Component>(&'a self, _entity: Entity) -> Option<Ref<'a, T>> {
        /*  let inner = self.0.borrow();

        if inner.contains(entity.into()) && inner.entry_ref(entity.into()).is_ok() {
            Some(Ref::map(self.0.borrow(), |x| {
                x.entry_ref(entity.into()).unwrap().into_component().unwrap()
            }))
        } else {
            None
        }*/
        todo!()
    }

    pub fn add<T: Component>(&mut self, entity: Entity, com: T) {
        if let Some(mut entity) = self.0.get_mut().entry(entity.into()) {
            entity.add_component(com);
        }
    }

    pub fn remove<T: Component>(&mut self, entity: Entity) {
        if let Some(mut entity) = self.0.get_mut().entry(entity.into()) {
            entity.remove_component::<T>();
        }
    }

    pub fn for_each<Q: IntoQuery + Send + Sync>(
        &self, func: impl Fn(<<Q as IntoView>::View as legion::query::View<'_>>::Element),
    ) where
        Q::View: ReadOnly,
    {
        let world =
            unsafe { self.0.get().as_ref().expect("This is a bug. World is not initialized.") };
        unsafe {
            <Q>::query().for_each_unchecked(world, func);
        }
    }

    pub fn query_get<Q: IntoQuery + Send + Sync>(
        &self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element>
    where
        Q::View: ReadOnly,
    {
        let world =
            unsafe { self.0.get().as_ref().expect("This is a bug. World is not initialized.") };

        unsafe { <Q>::query().get_unchecked(world, entity.into()).ok() }
    }

    pub fn query_mut<Q: IntoQuery + Send + Sync>(
        &mut self, entity: Entity,
    ) -> Option<<<Q as IntoView>::View as legion::query::View<'_>>::Element> {
        unsafe {
            let world = self.0.get_mut();
            <Q>::query().get_unchecked(world, entity.into()).ok()
        }
    }

    pub fn for_each_mut<Q: IntoQuery + Send + Sync>(
        &mut self, func: impl FnMut(<<Q as IntoView>::View as legion::query::View<'_>>::Element),
    ) {
        let mut world = self.0.get_mut();
        unsafe {
            <Q>::query().for_each_unchecked(world.deref_mut(), func);
        }
    }

    pub fn inner_mut(&mut self) -> RefMut<legion::World> {
        // self.0.get_mut()
        todo!()
    }

    pub fn inner(&self) -> Ref<legion::World> {
        //  self.0.borrow()
        todo!()
    }

    pub fn tick(&mut self, delta: &Timestep, input_state: &Ref<InputState>) {
        let mut world = self.0.get_mut();
        for script in <&mut Script>::query().iter_mut(world.deref_mut()) {
            script.tick(delta, input_state);
        }
    }
}
