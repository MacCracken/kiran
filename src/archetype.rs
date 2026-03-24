//! Archetype-based SOA component storage.
//!
//! Groups entities by their component signature (archetype). Entities with the
//! same set of component types are stored contiguously for cache-friendly iteration.
//!
//! This is an **opt-in** storage layer that sits alongside the existing
//! `World` component storage. Use it for hot-path iteration over large entity
//! sets where cache locality matters (e.g. physics bodies, particles, transforms).

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::world::Entity;

/// A set of component types that defines an archetype.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchetypeId(Vec<TypeId>);

impl ArchetypeId {
    /// Create an archetype ID from a sorted list of type IDs.
    fn from_sorted(mut types: Vec<TypeId>) -> Self {
        types.sort();
        Self(types)
    }

    /// Number of component types in this archetype.
    #[must_use]
    #[inline]
    pub fn component_count(&self) -> usize {
        self.0.len()
    }

    /// Check if this archetype contains a specific component type.
    #[must_use]
    pub fn contains(&self, type_id: &TypeId) -> bool {
        self.0.contains(type_id)
    }
}

/// A column of components for a single type within an archetype.
struct Column {
    /// Type-erased component data.
    data: Vec<Box<dyn Any + Send + Sync>>,
}

impl Column {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, value: Box<dyn Any + Send + Sync>) {
        self.data.push(value);
    }

    fn get<T: 'static>(&self, row: usize) -> Option<&T> {
        self.data.get(row)?.downcast_ref::<T>()
    }

    fn get_mut<T: 'static>(&mut self, row: usize) -> Option<&mut T> {
        self.data.get_mut(row)?.downcast_mut::<T>()
    }

    fn swap_remove(&mut self, row: usize) -> Box<dyn Any + Send + Sync> {
        self.data.swap_remove(row)
    }
}

/// An archetype — a group of entities with the same component set.
struct Archetype {
    id: ArchetypeId,
    /// Entity IDs in this archetype (parallel with column rows).
    entities: Vec<Entity>,
    /// Component columns keyed by TypeId.
    columns: HashMap<TypeId, Column>,
}

impl Archetype {
    fn new(id: ArchetypeId) -> Self {
        let mut columns = HashMap::new();
        for &tid in &id.0 {
            columns.insert(tid, Column::new());
        }
        Self {
            id,
            entities: Vec::new(),
            columns,
        }
    }

    fn len(&self) -> usize {
        self.entities.len()
    }

    /// Remove an entity by swapping with the last row.
    fn remove(&mut self, row: usize) -> Entity {
        let entity = self.entities.swap_remove(row);
        for column in self.columns.values_mut() {
            column.swap_remove(row);
        }
        entity
    }
}

/// Archetype storage — manages all archetypes and entity→archetype mapping.
pub struct ArchetypeStore {
    /// All archetypes, indexed by archetype ID.
    archetypes: Vec<Archetype>,
    /// ArchetypeId → index into `archetypes`.
    id_to_index: HashMap<ArchetypeId, usize>,
    /// Entity → (archetype index, row within archetype).
    entity_locations: HashMap<Entity, (usize, usize)>,
}

impl Default for ArchetypeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchetypeStore {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            id_to_index: HashMap::new(),
            entity_locations: HashMap::new(),
        }
    }

    /// Insert an entity with its components into the appropriate archetype.
    ///
    /// Components are provided as `(TypeId, Box<dyn Any>)` pairs.
    pub fn insert(
        &mut self,
        entity: Entity,
        components: Vec<(TypeId, Box<dyn Any + Send + Sync>)>,
    ) {
        let type_ids: Vec<TypeId> = components.iter().map(|(tid, _)| *tid).collect();
        let arch_id = ArchetypeId::from_sorted(type_ids);

        let arch_idx = if let Some(&idx) = self.id_to_index.get(&arch_id) {
            idx
        } else {
            let idx = self.archetypes.len();
            self.archetypes.push(Archetype::new(arch_id.clone()));
            self.id_to_index.insert(arch_id, idx);
            idx
        };

        let archetype = &mut self.archetypes[arch_idx];
        let row = archetype.len();
        archetype.entities.push(entity);

        for (tid, value) in components {
            if let Some(column) = archetype.columns.get_mut(&tid) {
                column.push(value);
            }
        }

        self.entity_locations.insert(entity, (arch_idx, row));
    }

    /// Remove an entity from its archetype.
    pub fn remove(&mut self, entity: Entity) -> bool {
        let Some((arch_idx, row)) = self.entity_locations.remove(&entity) else {
            return false;
        };

        let archetype = &mut self.archetypes[arch_idx];
        archetype.remove(row);

        // If we swapped, update the moved entity's location
        if row < archetype.len() {
            let moved_entity = archetype.entities[row];
            self.entity_locations.insert(moved_entity, (arch_idx, row));
        }

        true
    }

    /// Get a component for an entity.
    pub fn get<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        let &(arch_idx, row) = self.entity_locations.get(&entity)?;
        let archetype = &self.archetypes[arch_idx];
        let column = archetype.columns.get(&TypeId::of::<T>())?;
        column.get::<T>(row)
    }

    /// Get a mutable component for an entity.
    pub fn get_mut<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        let &(arch_idx, row) = self.entity_locations.get(&entity)?;
        let archetype = &mut self.archetypes[arch_idx];
        let column = archetype.columns.get_mut(&TypeId::of::<T>())?;
        column.get_mut::<T>(row)
    }

    /// Iterate over all entities with component A (cache-friendly within archetypes).
    pub fn query<A: 'static + Send + Sync>(&self) -> Vec<(Entity, &A)> {
        let tid = TypeId::of::<A>();
        let mut results = Vec::new();

        for archetype in &self.archetypes {
            if !archetype.id.contains(&tid) {
                continue;
            }
            let column = match archetype.columns.get(&tid) {
                Some(c) => c,
                None => continue,
            };
            for (row, entity) in archetype.entities.iter().enumerate() {
                if let Some(component) = column.get::<A>(row) {
                    results.push((*entity, component));
                }
            }
        }

        results
    }

    /// Iterate over entities with components A and B.
    pub fn query2<A: 'static + Send + Sync, B: 'static + Send + Sync>(
        &self,
    ) -> Vec<(Entity, &A, &B)> {
        let tid_a = TypeId::of::<A>();
        let tid_b = TypeId::of::<B>();
        let mut results = Vec::new();

        for archetype in &self.archetypes {
            if !archetype.id.contains(&tid_a) || !archetype.id.contains(&tid_b) {
                continue;
            }
            let col_a = match archetype.columns.get(&tid_a) {
                Some(c) => c,
                None => continue,
            };
            let col_b = match archetype.columns.get(&tid_b) {
                Some(c) => c,
                None => continue,
            };
            for (row, entity) in archetype.entities.iter().enumerate() {
                if let (Some(a), Some(b)) = (col_a.get::<A>(row), col_b.get::<B>(row)) {
                    results.push((*entity, a, b));
                }
            }
        }

        results
    }

    /// Check if an entity is in the archetype store.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        self.entity_locations.contains_key(&entity)
    }

    /// Total entities across all archetypes.
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entity_locations.len()
    }

    /// Number of archetypes.
    #[must_use]
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    /// Get the sizes of each archetype (for diagnostics).
    #[must_use]
    pub fn archetype_sizes(&self) -> Vec<usize> {
        self.archetypes.iter().map(|a| a.len()).collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position(f32, f32);
    #[derive(Debug, PartialEq)]
    struct Velocity(f32, f32);
    #[derive(Debug, PartialEq)]
    struct Health(u32);

    fn pos_component(x: f32, y: f32) -> (TypeId, Box<dyn Any + Send + Sync>) {
        (TypeId::of::<Position>(), Box::new(Position(x, y)))
    }

    fn vel_component(x: f32, y: f32) -> (TypeId, Box<dyn Any + Send + Sync>) {
        (TypeId::of::<Velocity>(), Box::new(Velocity(x, y)))
    }

    fn health_component(hp: u32) -> (TypeId, Box<dyn Any + Send + Sync>) {
        (TypeId::of::<Health>(), Box::new(Health(hp)))
    }

    #[test]
    fn store_new() {
        let store = ArchetypeStore::new();
        assert_eq!(store.entity_count(), 0);
        assert_eq!(store.archetype_count(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut store = ArchetypeStore::new();
        let e = Entity::new(0, 0);
        store.insert(e, vec![pos_component(1.0, 2.0), vel_component(3.0, 4.0)]);

        let pos = store.get::<Position>(e).unwrap();
        assert_eq!(*pos, Position(1.0, 2.0));
        let vel = store.get::<Velocity>(e).unwrap();
        assert_eq!(*vel, Velocity(3.0, 4.0));
    }

    #[test]
    fn same_archetype_grouped() {
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0), vel_component(0.0, 1.0)]);
        store.insert(e2, vec![pos_component(2.0, 0.0), vel_component(0.0, 2.0)]);

        assert_eq!(store.archetype_count(), 1); // same archetype
        assert_eq!(store.entity_count(), 2);
    }

    #[test]
    fn different_archetypes() {
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0)]);
        store.insert(e2, vec![pos_component(2.0, 0.0), vel_component(0.0, 1.0)]);

        assert_eq!(store.archetype_count(), 2);
    }

    #[test]
    fn get_mut() {
        let mut store = ArchetypeStore::new();
        let e = Entity::new(0, 0);
        store.insert(e, vec![pos_component(1.0, 2.0)]);

        store.get_mut::<Position>(e).unwrap().0 = 99.0;
        assert_eq!(store.get::<Position>(e).unwrap().0, 99.0);
    }

    #[test]
    fn remove_entity() {
        let mut store = ArchetypeStore::new();
        let e = Entity::new(0, 0);
        store.insert(e, vec![pos_component(1.0, 2.0)]);
        assert!(store.contains(e));

        store.remove(e);
        assert!(!store.contains(e));
        assert_eq!(store.entity_count(), 0);
    }

    #[test]
    fn remove_with_swap() {
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        let e3 = Entity::new(2, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0)]);
        store.insert(e2, vec![pos_component(2.0, 0.0)]);
        store.insert(e3, vec![pos_component(3.0, 0.0)]);

        store.remove(e1); // e3 gets swapped into row 0
        assert_eq!(store.entity_count(), 2);
        assert!(store.contains(e2));
        assert!(store.contains(e3));
        assert_eq!(store.get::<Position>(e3).unwrap().0, 3.0);
    }

    #[test]
    fn query_single() {
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        let e3 = Entity::new(2, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0)]);
        store.insert(e2, vec![pos_component(2.0, 0.0), vel_component(0.0, 1.0)]);
        store.insert(e3, vec![health_component(100)]);

        let positions = store.query::<Position>();
        assert_eq!(positions.len(), 2); // e1 and e2 have Position
    }

    #[test]
    fn query2_components() {
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0)]);
        store.insert(e2, vec![pos_component(2.0, 0.0), vel_component(0.0, 1.0)]);

        let results = store.query2::<Position, Velocity>();
        assert_eq!(results.len(), 1); // only e2 has both
        assert_eq!(results[0].1, &Position(2.0, 0.0));
    }

    #[test]
    fn get_missing_component() {
        let mut store = ArchetypeStore::new();
        let e = Entity::new(0, 0);
        store.insert(e, vec![pos_component(1.0, 0.0)]);
        assert!(store.get::<Velocity>(e).is_none());
    }

    #[test]
    fn get_missing_entity() {
        let store = ArchetypeStore::new();
        let e = Entity::new(99, 0);
        assert!(store.get::<Position>(e).is_none());
    }

    #[test]
    fn remove_missing_entity() {
        let mut store = ArchetypeStore::new();
        assert!(!store.remove(Entity::new(99, 0)));
    }

    #[test]
    fn archetype_sizes() {
        let mut store = ArchetypeStore::new();
        for i in 0..5 {
            store.insert(
                Entity::new(i, 0),
                vec![pos_component(i as f32, 0.0), vel_component(0.0, 0.0)],
            );
        }
        for i in 5..8 {
            store.insert(Entity::new(i, 0), vec![pos_component(i as f32, 0.0)]);
        }

        let sizes = store.archetype_sizes();
        assert_eq!(sizes.len(), 2);
        assert!(sizes.contains(&5));
        assert!(sizes.contains(&3));
    }

    #[test]
    fn archetype_id_order_independent() {
        // Components added in different order should produce the same archetype
        let mut store = ArchetypeStore::new();
        let e1 = Entity::new(0, 0);
        let e2 = Entity::new(1, 0);
        store.insert(e1, vec![pos_component(1.0, 0.0), vel_component(0.0, 1.0)]);
        store.insert(e2, vec![vel_component(0.0, 2.0), pos_component(2.0, 0.0)]);

        assert_eq!(store.archetype_count(), 1); // same archetype regardless of insert order
    }

    #[test]
    fn store_as_world_resource() {
        let mut world = crate::World::new();
        world.insert_resource(ArchetypeStore::new());
        let store = world.get_resource::<ArchetypeStore>().unwrap();
        assert_eq!(store.entity_count(), 0);
    }

    #[test]
    fn large_archetype_iteration() {
        let mut store = ArchetypeStore::new();
        for i in 0..1000u32 {
            store.insert(
                Entity::new(i, 0),
                vec![pos_component(i as f32, 0.0), vel_component(1.0, 1.0)],
            );
        }

        let results = store.query::<Position>();
        assert_eq!(results.len(), 1000);
    }
}
