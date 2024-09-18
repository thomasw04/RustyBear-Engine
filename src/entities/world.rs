use hecs::{Component, ComponentRef, DynamicBundle, Without};
use hecs_hierarchy::{
    BreadthFirstIterator, Child, ChildrenIter, DepthFirstIterator, Hierarchy, HierarchyMut, Parent,
};
use smallvec::SmallVec;
// -------------------------------------
// World System.
// Abstraction layer for managing state.
// -------------------------------------
use std::cell::{Ref, UnsafeCell};
use std::slice::Iter;

use crate::{input::InputState, utils::Timestep};

pub trait Scriptable {
    fn on_spawn(&mut self, entity: Entity);
    fn tick(&mut self, delta: &Timestep, input_state: &Ref<InputState>);
    fn on_destroy(&mut self, entity: Entity);
}

pub type Script = Box<dyn Scriptable + Send + Sync>;

/*pub trait Entable {
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
}*/

pub trait Entable: Scriptable {
    fn handle(&self) -> Entity;
    fn world(&self) -> &World;
    fn cmds(&self) -> &CommandBuffer;
    fn cmds_mut(&mut self) -> &mut CommandBuffer;

    fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        let entity = self.world().reserve_entity();
        self.cmds_mut().spawn(entity.into(), components).into()
    }

    fn add<T: Component>(&mut self, component: T) {
        let handle = self.handle();
        self.cmds_mut().buffer.insert_one(handle.into(), component);
    }

    fn remove<T: Component>(&mut self) {
        let handle = self.handle();
        self.cmds_mut().buffer.remove_one::<T>(handle.into());
    }

    fn add_by_entity<T: Component>(&mut self, entity: Entity, component: T) {
        self.cmds_mut().buffer.insert_one(entity.into(), component);
    }

    fn remove_by_entity<T: Component>(&mut self, entity: Entity) {
        self.cmds_mut().buffer.remove_one::<T>(entity.into());
    }

    fn get<'a, T: ComponentRef<'a>>(&self) -> Option<T::Ref> {
        self.world().get::<T>(self.handle().into())
    }

    fn get_by_entity<'a, T: ComponentRef<'a>>(&self, entity: Entity) -> Option<T::Ref> {
        self.world().get::<T>(entity.into())
    }

    fn iter<'a, Q: hecs::Query>(&self) -> hecs::QueryIter<'a, Q> {
        self.world().iter()
    }

    fn view<'a, Q: hecs::Query>(&self) -> hecs::View<'a, Q> {
        self.world().view()
    }

    fn despawn(&mut self, entity: Entity) {
        self.cmds_mut().despawn(entity);
    }
}

pub trait Instantiable: Entable {
    fn instantiate<T: DynamicBundle>(world: &World, components: T);
}

pub struct CommandBuffer {
    buffer: hecs::CommandBuffer,
    spawn: SmallVec<[hecs::Entity; 16]>,
    despawn: SmallVec<[hecs::Entity; 16]>,
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            buffer: hecs::CommandBuffer::new(),
            spawn: SmallVec::new(),
            despawn: SmallVec::new(),
        }
    }

    pub fn spawn(&mut self, entity: hecs::Entity, components: impl DynamicBundle) -> Entity {
        self.buffer.insert(entity, components);
        self.spawn.push(entity);
        entity.into()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.buffer.despawn(entity.into());
        self.despawn.push(entity.into());
    }

    pub fn apply(&mut self, world: &mut World) {
        world.apply(&mut self.buffer);

        let world = unsafe { &mut *world.inner.get() };

        for entity in self.despawn.iter() {
            if let Ok(mut component) = world.get::<&mut Script>(*entity) {
                component.on_destroy((*entity).into());
            }
        }

        for entity in self.spawn.iter() {
            if let Ok(mut component) = world.get::<&mut Script>(*entity) {
                component.on_spawn((*entity).into());
            }
        }

        self.spawn.clear();
        self.despawn.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Entity(Option<hecs::Entity>);

impl From<hecs::Entity> for Entity {
    fn from(value: hecs::Entity) -> Self {
        Self(Some(value))
    }
}

impl From<Entity> for hecs::Entity {
    fn from(value: Entity) -> Self {
        match value.0 {
            Some(x) => x,
            None => panic!("Invalid entity identifier."),
        }
    }
}

// --------------------------------------------------------------

pub struct World {
    inner: UnsafeCell<hecs::World>,
    global: hecs::Entity,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let mut world = hecs::World::new();

        //We spawn a global entity where systems can store state.
        let global = world.spawn(());

        Self { inner: UnsafeCell::new(world), global }
    }

    pub fn root<T: Component>(&self) -> hecs::QueryIter<'_, Without<&Parent<T>, &Child<T>>> {
        let world = unsafe { &*self.inner.get() };
        world.query::<&Parent<T>>().without::<&Child<T>>().iter()
    }

    pub fn global(&self) -> Entity {
        self.global.into()
    }

    pub fn children<T: Component>(&self, entity: Entity) -> ChildrenIter<T> {
        let world = unsafe { &*self.inner.get() };
        world.children(entity.into())
    }

    pub fn set_parent<T: Component>(&self, child: Entity, parent: Entity) {
        let world = unsafe { &*self.inner.get() };
        if let Ok(old) = world.parent::<T>(child.into()) {
            if old == parent.into() {
                return;
            }
        }

        let _ = world.detach::<T>(child.into());
        world.attach::<T>(child.into(), parent.into());
    }

    pub fn bfs<T: Component>(&self, entity: Entity) -> BreadthFirstIterator<'_, hecs::World, T> {
        let world = unsafe { &*self.inner.get() };
        world.descendants_breadth_first(entity.into())
    }

    pub fn dfs<T: Component>(&self, entity: Entity) -> DepthFirstIterator<'_, T> {
        let world = unsafe { &*self.inner.get() };
        world.descendants_depth_first(entity.into())
    }

    pub fn instantiate<T: Instantiable>(&self, components: impl DynamicBundle) {
        T::instantiate(self, components);
    }

    pub fn spawn(&self, components: impl DynamicBundle) -> Entity {
        let world = unsafe { &mut *self.inner.get() };
        let entity = world.spawn(components);

        if let Ok(mut component) = world.get::<&mut Script>(entity) {
            component.on_spawn(entity.into());
        }

        entity.into()
    }

    pub fn insert(&self, entity: Entity, components: impl DynamicBundle) {
        let world = unsafe { &mut *self.inner.get() };
        world.insert(entity.into(), components);
    }

    pub fn insert_one<T: Component>(&self, entity: Entity, component: T) {
        let world = unsafe { &mut *self.inner.get() };
        world.insert_one(entity.into(), component);
    }

    pub fn despawn(&self, entity: Entity) {
        let world = unsafe { &mut *self.inner.get() };

        if let Ok(mut component) = world.get::<&mut Script>(entity.into()) {
            component.on_destroy(entity);
        }

        world.despawn(entity.into());
    }

    pub fn iter<Q: hecs::Query>(&self) -> hecs::QueryIter<Q> {
        let world = unsafe { &*self.inner.get() };
        world.query::<Q>().iter()
    }

    pub fn reserve_entity(&self) -> Entity {
        let world = unsafe { &*self.inner.get() };
        world.reserve_entity().into()
    }

    pub fn get<'a, T: ComponentRef<'a>>(&self, entity: Entity) -> Option<T::Ref> {
        let world = unsafe { &*self.inner.get() };
        world.get::<T>(entity.into()).ok()
    }

    pub fn apply(&self, cmds: &mut hecs::CommandBuffer) {
        let world = unsafe { &mut *self.inner.get() };
        cmds.run_on(world);
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryIter<Q> {
        let world = unsafe { &*self.inner.get() };
        world.query::<Q>().iter()
    }

    pub fn view<Q: hecs::Query>(&self) -> hecs::View<Q> {
        let world = unsafe { &*self.inner.get() };
        world.query::<Q>().view()
    }
}

unsafe impl Send for World {}
unsafe impl Sync for World {}

/*pub struct World(UnsafeCell<legion::World>);

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

struct Children {
    children: SmallVec<[Entity; 8]>,
}

struct Parent {
    parent: Entity,
}

impl World {
    pub fn new() -> Self {
        Self(UnsafeCell::new(legion::World::default()))
    }

    pub unsafe fn root<T: EntityFilter>(&self, filter: T) -> Vec<Entity> {
        <legion::Entity>::query()
            .filter(!legion::component::<Parent>())
            .filter(filter.into())
            .iter(self.inner())
            .copied()
            .map(|x| x.into())
            .collect()
    }

    /// # Safety
    /// Creates a reference to the world.
    /// This reference is only allowed to exist while no other mutable reference is alive.
    pub unsafe fn children<'a>(&'a self, entity: Entity) -> Iter<'a, Entity> {
        self.query_get::<&Children>(entity).map(|x| x.children.iter()).unwrap_or_else(|| [].iter())
    }

    /// # Safety
    /// Creates a mutable reference to the world.
    /// This reference is only allowed to exist while no other reference (mutable or not) is alive.
    pub unsafe fn set_parent(&self, entity: Entity, parent: Entity) {
        let world = self.inner_mut();
        if let Some(mut child) = world.entry(entity.into()) {
            if let Some(mut par) = world.entry(parent.into()) {
                if let Ok(mut com) = par.get_component_mut::<Children>() {
                    com.children.push(entity);
                } else {
                    par.add_component(Children { children: SmallVec::from_elem(entity, 1) });
                }

                if let Ok(mut com) = child.get_component_mut::<Parent>() {
                    if let Some(index) = com.parent.0 {
                        if let Some(mut parent) = world.entry(index) {
                            if let Ok(mut com) = parent.get_component_mut::<Children>() {
                                com.children.retain(|x| *x != entity);
                            }
                        }
                    }
                    com.parent = parent;
                } else {
                    child.add_component(Parent { parent });
                }
            } else {
                if let Ok(mut com) = child.get_component_mut::<Parent>() {
                    if let Some(index) = com.parent.0 {
                        if let Some(mut parent) = world.entry(index) {
                            if let Ok(mut com) = parent.get_component_mut::<Children>() {
                                com.children.retain(|x| *x != entity);
                            }
                        }
                    }
                }
                child.remove_component::<Parent>();
            }
        } else {
            log::warn!("Warining: Entity not found.");
        }
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
}*/
