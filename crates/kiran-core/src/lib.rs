//! kiran-core — ECS, game loop, time management, events
//!
//! Provides the fundamental building blocks for the Kiran game engine:
//! - Entity/Component/System (ECS) world
//! - Entity allocation with generational indices
//! - Typed event bus
//! - Game clock with fixed timestep support

use std::any::{Any, TypeId};
use std::collections::HashMap;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by kiran-core.
#[derive(Debug, Error)]
pub enum KiranError {
    #[error("entity {0:?} does not exist")]
    EntityNotFound(Entity),

    #[error("component not found for entity {0:?}")]
    ComponentNotFound(Entity),

    #[error("resource of type `{0}` not found")]
    ResourceNotFound(&'static str),

    #[error("entity {0:?} has already been despawned")]
    EntityDespawned(Entity),

    #[error("scene error: {0}")]
    Scene(String),

    #[error("render error: {0}")]
    Render(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, KiranError>;

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// A handle to an entity in the ECS world.
///
/// Upper 32 bits = generation, lower 32 bits = index.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u64);

impl Entity {
    /// Create an entity from an index and generation.
    #[inline]
    pub fn new(index: u32, generation: u32) -> Self {
        Self((generation as u64) << 32 | index as u64)
    }

    /// Index portion (lower 32 bits).
    #[inline]
    pub fn index(self) -> u32 {
        self.0 as u32
    }

    /// Generation portion (upper 32 bits).
    #[inline]
    pub fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Raw u64 id.
    #[inline]
    pub fn id(self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({}v{})", self.index(), self.generation())
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.index(), self.generation())
    }
}

// ---------------------------------------------------------------------------
// EntityAllocator
// ---------------------------------------------------------------------------

/// Allocates and recycles entity indices with generational safety.
#[derive(Debug, Default)]
pub struct EntityAllocator {
    /// Next fresh index (used when free list is empty).
    next_index: u32,
    /// Generation per index slot.
    generations: Vec<u32>,
    /// Recycled indices available for reuse.
    free_list: Vec<u32>,
    /// Tracks which entities are alive.
    alive: Vec<bool>,
}

impl EntityAllocator {
    /// Allocate a new entity.
    pub fn spawn(&mut self) -> Entity {
        if let Some(index) = self.free_list.pop() {
            self.alive[index as usize] = true;
            Entity::new(index, self.generations[index as usize])
        } else {
            let index = self.next_index;
            self.next_index += 1;
            self.generations.push(0);
            self.alive.push(true);
            Entity::new(index, 0)
        }
    }

    /// Despawn an entity, bumping its generation for reuse.
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        let idx = entity.index() as usize;
        if idx >= self.alive.len() || !self.alive[idx] {
            return Err(KiranError::EntityNotFound(entity));
        }
        if self.generations[idx] != entity.generation() {
            return Err(KiranError::EntityDespawned(entity));
        }
        self.alive[idx] = false;
        self.generations[idx] += 1;
        self.free_list.push(entity.index());
        Ok(())
    }

    /// Check whether an entity handle is still alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index() as usize;
        idx < self.alive.len()
            && self.alive[idx]
            && self.generations[idx] == entity.generation()
    }

    /// Number of currently alive entities.
    pub fn alive_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

type ComponentMap = HashMap<u64, Box<dyn Any + Send + Sync>>;

/// The central ECS container — entities, components, and resources.
pub struct World {
    allocator: EntityAllocator,
    /// component storage: TypeId -> (entity id -> boxed component)
    components: HashMap<TypeId, ComponentMap>,
    /// singleton resources
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::default(),
            components: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Spawn a new entity.
    pub fn spawn(&mut self) -> Entity {
        self.allocator.spawn()
    }

    /// Despawn an entity and remove all its components.
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        self.allocator.despawn(entity)?;
        let eid = entity.id();
        for storage in self.components.values_mut() {
            storage.remove(&eid);
        }
        Ok(())
    }

    /// Check if an entity is alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.allocator.is_alive(entity)
    }

    /// Insert a component on an entity.
    pub fn insert_component<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
        component: T,
    ) -> Result<()> {
        if !self.allocator.is_alive(entity) {
            return Err(KiranError::EntityNotFound(entity));
        }
        self.components
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(entity.id(), Box::new(component));
        Ok(())
    }

    /// Get a reference to an entity's component.
    pub fn get_component<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(&entity.id())?
            .downcast_ref::<T>()
    }

    /// Get a mutable reference to an entity's component.
    pub fn get_component_mut<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(&entity.id())?
            .downcast_mut::<T>()
    }

    /// Remove a component from an entity, returning it if it existed.
    pub fn remove_component<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Option<T> {
        let storage = self.components.get_mut(&TypeId::of::<T>())?;
        let boxed = storage.remove(&entity.id())?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }

    /// Insert a singleton resource.
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources
            .insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Get a reference to a singleton resource.
    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())?
            .downcast_ref::<T>()
    }

    /// Get a mutable reference to a singleton resource.
    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())?
            .downcast_mut::<T>()
    }

    /// Number of alive entities.
    pub fn entity_count(&self) -> usize {
        self.allocator.alive_count()
    }
}

// ---------------------------------------------------------------------------
// GameClock
// ---------------------------------------------------------------------------

/// Tracks frame timing and provides a fixed timestep accumulator.
#[derive(Debug, Clone)]
pub struct GameClock {
    /// Delta time for this frame (seconds).
    pub delta: f64,
    /// Total elapsed time (seconds).
    pub elapsed: f64,
    /// Frame counter.
    pub frame: u64,
    /// Fixed timestep interval (seconds).
    pub fixed_timestep: f64,
    /// Internal accumulator for fixed updates.
    accumulator: f64,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            frame: 0,
            fixed_timestep: 1.0 / 60.0,
            accumulator: 0.0,
        }
    }
}

impl GameClock {
    /// Create a clock with a given fixed timestep.
    pub fn with_timestep(fixed_timestep: f64) -> Self {
        Self {
            fixed_timestep,
            ..Default::default()
        }
    }

    /// Advance the clock by `dt` seconds.
    pub fn tick(&mut self, dt: f64) {
        self.delta = dt;
        self.elapsed += dt;
        self.frame += 1;
        self.accumulator += dt;
    }

    /// Consume one fixed-timestep chunk if available. Returns true if consumed.
    pub fn consume_fixed(&mut self) -> bool {
        if self.accumulator >= self.fixed_timestep {
            self.accumulator -= self.fixed_timestep;
            true
        } else {
            false
        }
    }

    /// How many fixed steps are pending.
    pub fn pending_fixed_steps(&self) -> u32 {
        (self.accumulator / self.fixed_timestep) as u32
    }
}

// ---------------------------------------------------------------------------
// EventBus
// ---------------------------------------------------------------------------

/// A simple typed event bus: publish events, drain per type.
#[derive(Default)]
pub struct EventBus {
    channels: HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish an event.
    pub fn publish<E: 'static + Send + Sync>(&mut self, event: E) {
        self.channels
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(event));
    }

    /// Drain all events of a given type, returning them.
    pub fn drain<E: 'static + Send + Sync>(&mut self) -> Vec<E> {
        self.channels
            .remove(&TypeId::of::<E>())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|b| b.downcast::<E>().ok().map(|b| *b))
            .collect()
    }

    /// Peek at the count of pending events of a given type.
    pub fn count<E: 'static + Send + Sync>(&self) -> usize {
        self.channels
            .get(&TypeId::of::<E>())
            .map_or(0, |v| v.len())
    }

    /// Clear all events across all types.
    pub fn clear(&mut self) {
        self.channels.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Entity tests --

    #[test]
    fn entity_index_generation() {
        let e = Entity::new(42, 7);
        assert_eq!(e.index(), 42);
        assert_eq!(e.generation(), 7);
    }

    #[test]
    fn entity_id_roundtrip() {
        let e = Entity::new(100, 3);
        let id = e.id();
        let e2 = Entity(id);
        assert_eq!(e, e2);
    }

    #[test]
    fn entity_display() {
        let e = Entity::new(5, 2);
        assert_eq!(format!("{e}"), "5v2");
        assert_eq!(format!("{e:?}"), "Entity(5v2)");
    }

    // -- EntityAllocator tests --

    #[test]
    fn allocator_spawn_sequential() {
        let mut alloc = EntityAllocator::default();
        let e0 = alloc.spawn();
        let e1 = alloc.spawn();
        assert_eq!(e0.index(), 0);
        assert_eq!(e1.index(), 1);
        assert_eq!(e0.generation(), 0);
        assert_eq!(alloc.alive_count(), 2);
    }

    #[test]
    fn allocator_despawn_and_recycle() {
        let mut alloc = EntityAllocator::default();
        let e0 = alloc.spawn();
        alloc.despawn(e0).unwrap();
        assert_eq!(alloc.alive_count(), 0);

        let e0_reused = alloc.spawn();
        assert_eq!(e0_reused.index(), 0);
        assert_eq!(e0_reused.generation(), 1);
        assert!(alloc.is_alive(e0_reused));
        assert!(!alloc.is_alive(e0)); // stale handle
    }

    #[test]
    fn allocator_despawn_invalid() {
        let mut alloc = EntityAllocator::default();
        let fake = Entity::new(999, 0);
        assert!(alloc.despawn(fake).is_err());
    }

    #[test]
    fn allocator_double_despawn() {
        let mut alloc = EntityAllocator::default();
        let e = alloc.spawn();
        alloc.despawn(e).unwrap();
        assert!(alloc.despawn(e).is_err());
    }

    // -- World tests --

    #[derive(Debug, Clone, PartialEq)]
    struct Health(i32);

    #[derive(Debug, Clone, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[test]
    fn world_spawn_and_count() {
        let mut world = World::new();
        assert_eq!(world.entity_count(), 0);
        let _e = world.spawn();
        assert_eq!(world.entity_count(), 1);
        let _e2 = world.spawn();
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn world_insert_get_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100)).unwrap();

        let h = world.get_component::<Health>(e).unwrap();
        assert_eq!(h.0, 100);
    }

    #[test]
    fn world_get_missing_component() {
        let world = World::new();
        let e = Entity::new(0, 0);
        assert!(world.get_component::<Health>(e).is_none());
    }

    #[test]
    fn world_remove_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(50)).unwrap();

        let removed = world.remove_component::<Health>(e);
        assert_eq!(removed, Some(Health(50)));
        assert!(world.get_component::<Health>(e).is_none());
    }

    #[test]
    fn world_despawn_removes_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(75)).unwrap();
        world
            .insert_component(e, Velocity { x: 1.0, y: 2.0 })
            .unwrap();

        world.despawn(e).unwrap();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn world_component_mut() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(10)).unwrap();

        let h = world.get_component_mut::<Health>(e).unwrap();
        h.0 += 5;

        assert_eq!(world.get_component::<Health>(e).unwrap().0, 15);
    }

    #[test]
    fn world_insert_component_dead_entity() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e).unwrap();
        assert!(world.insert_component(e, Health(1)).is_err());
    }

    #[test]
    fn world_multiple_component_types() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100)).unwrap();
        world
            .insert_component(e, Velocity { x: 3.0, y: 4.0 })
            .unwrap();

        assert_eq!(world.get_component::<Health>(e).unwrap().0, 100);
        assert_eq!(world.get_component::<Velocity>(e).unwrap().x, 3.0);
    }

    // -- Resource tests --

    #[derive(Debug, PartialEq)]
    struct Gravity(f64);

    #[test]
    fn world_resources() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));
        assert_eq!(world.get_resource::<Gravity>().unwrap().0, 9.81);

        let g = world.get_resource_mut::<Gravity>().unwrap();
        g.0 = 1.625;
        assert_eq!(world.get_resource::<Gravity>().unwrap().0, 1.625);
    }

    #[test]
    fn world_missing_resource() {
        let world = World::new();
        assert!(world.get_resource::<Gravity>().is_none());
    }

    // -- GameClock tests --

    #[test]
    fn clock_tick() {
        let mut clock = GameClock::default();
        clock.tick(0.016);
        assert_eq!(clock.frame, 1);
        assert!((clock.delta - 0.016).abs() < 1e-10);
        assert!((clock.elapsed - 0.016).abs() < 1e-10);
    }

    #[test]
    fn clock_fixed_step() {
        let mut clock = GameClock::with_timestep(1.0 / 60.0);
        clock.tick(0.033); // ~2 frames at 60fps
        assert_eq!(clock.pending_fixed_steps(), 1);
        assert!(clock.consume_fixed());
        assert_eq!(clock.pending_fixed_steps(), 0);
    }

    #[test]
    fn clock_no_fixed_step_when_below() {
        let mut clock = GameClock::with_timestep(1.0 / 60.0);
        clock.tick(0.001);
        assert!(!clock.consume_fixed());
    }

    // -- EventBus tests --

    #[derive(Debug, PartialEq)]
    struct Collision {
        a: u64,
        b: u64,
    }

    #[derive(Debug, PartialEq)]
    struct ScoreChanged(i32);

    #[test]
    fn event_bus_publish_drain() {
        let mut bus = EventBus::new();
        bus.publish(Collision { a: 1, b: 2 });
        bus.publish(Collision { a: 3, b: 4 });

        assert_eq!(bus.count::<Collision>(), 2);
        let events = bus.drain::<Collision>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Collision { a: 1, b: 2 });
        assert_eq!(bus.count::<Collision>(), 0);
    }

    #[test]
    fn event_bus_different_types() {
        let mut bus = EventBus::new();
        bus.publish(Collision { a: 1, b: 2 });
        bus.publish(ScoreChanged(10));

        assert_eq!(bus.count::<Collision>(), 1);
        assert_eq!(bus.count::<ScoreChanged>(), 1);

        let scores = bus.drain::<ScoreChanged>();
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].0, 10);
    }

    #[test]
    fn event_bus_drain_empty() {
        let mut bus = EventBus::new();
        let events = bus.drain::<Collision>();
        assert!(events.is_empty());
    }

    #[test]
    fn event_bus_clear() {
        let mut bus = EventBus::new();
        bus.publish(Collision { a: 0, b: 0 });
        bus.publish(ScoreChanged(5));
        bus.clear();
        assert_eq!(bus.count::<Collision>(), 0);
        assert_eq!(bus.count::<ScoreChanged>(), 0);
    }
}
