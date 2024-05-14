// -------------------------------------
// Virtual Entity Component System.
// Abstraction layer for the actual Entity Component System
// -------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityHandle(hecs::Entity);

impl Default for EntityHandle {
    fn default() -> Self {
        Self(hecs::Entity::DANGLING)
    }
}

impl From<hecs::Entity> for EntityHandle {
    fn from(value: hecs::Entity) -> Self {
        Self(value)
    }
}

impl Into<hecs::Entity> for EntityHandle {
    fn into(self) -> hecs::Entity {
        self.0
    }
}

// -------------------------------------

pub struct World {
    inner: hecs::World,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl From<hecs::World> for World {
    fn from(value: hecs::World) -> Self {
        Self { inner: value }
    }
}

impl World {
    pub fn new() -> Self {
        Self { inner: hecs::World::new() }
    }

    pub fn spawn(&mut self, components: impl hecs::DynamicBundle) -> EntityHandle {
        self.inner.spawn(components).into()
    }

    pub fn is_alive(&self, entity: EntityHandle) -> bool {
        self.inner.contains(entity.into())
    }

    pub fn despawn(&mut self, entity: EntityHandle) -> bool {
        self.inner.despawn(entity.into()).is_ok()
    }

    pub fn get<'a, T: hecs::ComponentRef<'a>>(
        &'a self, entity: EntityHandle,
    ) -> Result<T::Ref, hecs::ComponentError> {
        self.inner.get::<T>(entity.into())
    }

    pub fn query<Q: hecs::Query>(&self) -> hecs::QueryBorrow<Q> {
        self.inner.query()
    }

    pub fn query_mut<Q: hecs::Query>(&mut self) -> hecs::QueryMut<Q> {
        self.inner.query_mut()
    }

    pub fn insert_one<T: hecs::Component>(
        &mut self, entity: EntityHandle, component: T,
    ) -> Result<(), hecs::NoSuchEntity> {
        self.inner.insert_one(entity.into(), component)
    }

    pub fn remove_one<T: hecs::Component>(
        &mut self, entity: EntityHandle,
    ) -> Result<T, hecs::ComponentError> {
        self.inner.remove_one::<T>(entity.into())
    }
}
