//! Game entity system with simple integer IDs

use serde::{Deserialize, Serialize};
use std::fmt;
use rustc_hash::FxHashMap;
use crate::Result;
use crate::MtgError;

/// Simple integer ID for game entities
///
/// Keeps IDs simple and contiguous for human readability and dense storage.
/// These IDs are stable throughout a game - entities don't get deallocated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

impl EntityId {
    pub fn new(id: u32) -> Self {
        EntityId(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Base trait for all game entities
pub trait GameEntity {
    fn id(&self) -> EntityId;
    fn name(&self) -> &str;
}

/// Central storage for all game entities
///
/// Provides fast lookup by EntityId and manages entity lifecycle.
/// Uses FxHashMap for fast hashing of integer keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityStore<T> {
    entities: FxHashMap<EntityId, T>,
    next_id: u32,
}

impl<T> EntityStore<T> {
    pub fn new() -> Self {
        EntityStore {
            entities: FxHashMap::default(),
            next_id: 0,
        }
    }

    /// Generate a new unique EntityId
    pub fn next_id(&mut self) -> EntityId {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Insert an entity with a specific ID
    pub fn insert(&mut self, id: EntityId, entity: T) {
        self.entities.insert(id, entity);
    }

    /// Get an entity by ID
    pub fn get(&self, id: EntityId) -> Result<&T> {
        self.entities
            .get(&id)
            .ok_or(MtgError::EntityNotFound(id.as_u32()))
    }

    /// Get a mutable reference to an entity
    pub fn get_mut(&mut self, id: EntityId) -> Result<&mut T> {
        self.entities
            .get_mut(&id)
            .ok_or(MtgError::EntityNotFound(id.as_u32()))
    }

    /// Check if an entity exists
    pub fn contains(&self, id: EntityId) -> bool {
        self.entities.contains_key(&id)
    }

    /// Remove an entity (rarely used - entities typically persist)
    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        self.entities.remove(&id)
    }

    /// Iterate over all entities
    pub fn iter(&self) -> impl Iterator<Item = (&EntityId, &T)> {
        self.entities.iter()
    }

    /// Get count of entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl<T> Default for EntityStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: EntityId,
        name: String,
    }

    impl GameEntity for TestEntity {
        fn id(&self) -> EntityId {
            self.id
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_entity_store() {
        let mut store = EntityStore::new();
        let id1 = store.next_id();
        let id2 = store.next_id();

        assert_eq!(id1.as_u32(), 0);
        assert_eq!(id2.as_u32(), 1);

        let entity1 = TestEntity {
            id: id1,
            name: "Test1".to_string(),
        };
        let entity2 = TestEntity {
            id: id2,
            name: "Test2".to_string(),
        };

        store.insert(id1, entity1.clone());
        store.insert(id2, entity2.clone());

        assert_eq!(store.len(), 2);
        assert_eq!(store.get(id1).unwrap().name, "Test1");
        assert_eq!(store.get(id2).unwrap().name, "Test2");
        assert!(store.get(EntityId::new(999)).is_err());
    }
}
