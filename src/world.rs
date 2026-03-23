//! ECS world, generational entity allocator, game clock, event bus
//!
//! Provides the fundamental building blocks for the Kiran game engine:
//! - Entity allocation with generational indices
//! - Component storage (type-erased, per-entity)
//! - Singleton resources
//! - Typed event bus
//! - Game clock with fixed timestep support

use std::any::{Any, TypeId};
use std::collections::HashMap;

use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by kiran.
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

    /// Reconstruct an entity from a raw u64 id.
    #[inline]
    pub fn from_id(id: u64) -> Self {
        Self(id)
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
    /// Cached count of alive entities (avoids O(n) scan).
    alive_count: usize,
}

impl EntityAllocator {
    /// Allocate a new entity.
    pub fn spawn(&mut self) -> Entity {
        self.alive_count += 1;
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
        self.alive_count -= 1;
        self.generations[idx] += 1;
        self.free_list.push(entity.index());
        Ok(())
    }

    /// Check whether an entity handle is still alive.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index() as usize;
        idx < self.alive.len() && self.alive[idx] && self.generations[idx] == entity.generation()
    }

    /// Number of currently alive entities (O(1)).
    pub fn alive_count(&self) -> usize {
        self.alive_count
    }
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

/// Dense component storage indexed by entity index (O(1) access).
type ComponentVec = Vec<Option<Box<dyn Any + Send + Sync>>>;

/// A resource entry: value + change tracking ticks.
struct ResourceEntry {
    value: Box<dyn Any + Send + Sync>,
    /// Tick at which this resource was last mutated.
    changed_tick: u64,
    /// Tick at which this resource was last checked via `clear_resource_changed`.
    last_checked_tick: Option<u64>,
}

/// The central ECS container — entities, components, and resources.
pub struct World {
    allocator: EntityAllocator,
    /// component storage: TypeId -> vec indexed by entity index
    components: HashMap<TypeId, ComponentVec>,
    /// singleton resources with integrated change tracking (single HashMap lookup)
    resources: HashMap<TypeId, ResourceEntry>,
    /// Global change tick (incremented via `increment_tick()`).
    tick: u64,
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
            tick: 0,
        }
    }

    /// Spawn a new entity.
    pub fn spawn(&mut self) -> Entity {
        self.allocator.spawn()
    }

    /// Despawn an entity and remove all its components.
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        self.allocator.despawn(entity)?;
        let idx = entity.index() as usize;
        for storage in self.components.values_mut() {
            if idx < storage.len() {
                storage[idx] = None;
            }
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
        let idx = entity.index() as usize;
        let storage = self.components.entry(TypeId::of::<T>()).or_default();
        if idx >= storage.len() {
            storage.resize_with(idx + 1, || None);
        }
        storage[idx] = Some(Box::new(component));
        Ok(())
    }

    /// Check if an entity has a component of the given type.
    pub fn has_component<T: 'static + Send + Sync>(&self, entity: Entity) -> bool {
        let idx = entity.index() as usize;
        self.components
            .get(&TypeId::of::<T>())
            .is_some_and(|storage| storage.get(idx).is_some_and(|slot| slot.is_some()))
    }

    /// Get a reference to an entity's component.
    pub fn get_component<T: 'static + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        let idx = entity.index() as usize;
        self.components
            .get(&TypeId::of::<T>())?
            .get(idx)?
            .as_ref()?
            .downcast_ref::<T>()
    }

    /// Get a mutable reference to an entity's component.
    pub fn get_component_mut<T: 'static + Send + Sync>(
        &mut self,
        entity: Entity,
    ) -> Option<&mut T> {
        let idx = entity.index() as usize;
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(idx)?
            .as_mut()?
            .downcast_mut::<T>()
    }

    /// Remove a component from an entity, returning it if it existed.
    pub fn remove_component<T: 'static + Send + Sync>(&mut self, entity: Entity) -> Option<T> {
        let idx = entity.index() as usize;
        let storage = self.components.get_mut(&TypeId::of::<T>())?;
        let boxed = storage.get_mut(idx)?.take()?;
        boxed.downcast::<T>().ok().map(|b| *b)
    }

    /// Insert a singleton resource.
    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources.insert(
            TypeId::of::<T>(),
            ResourceEntry {
                value: Box::new(resource),
                changed_tick: self.tick,
                last_checked_tick: None,
            },
        );
    }

    /// Get a reference to a singleton resource.
    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())?
            .value
            .downcast_ref::<T>()
    }

    /// Get a mutable reference to a singleton resource.
    /// Marks the resource as changed at the current tick.
    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let entry = self.resources.get_mut(&TypeId::of::<T>())?;
        entry.changed_tick = self.tick;
        entry.value.downcast_mut::<T>()
    }

    /// Check if a resource has changed since the last call to `clear_resource_changed`.
    pub fn is_resource_changed<T: 'static + Send + Sync>(&self) -> bool {
        let Some(entry) = self.resources.get(&TypeId::of::<T>()) else {
            return false;
        };
        match entry.last_checked_tick {
            Some(checked) => entry.changed_tick > checked,
            None => true, // never checked → changed
        }
    }

    /// Mark a resource as "seen" — future `is_resource_changed` returns false until modified again.
    pub fn clear_resource_changed<T: 'static + Send + Sync>(&mut self) {
        if let Some(entry) = self.resources.get_mut(&TypeId::of::<T>()) {
            entry.last_checked_tick = Some(self.tick);
        }
    }

    /// Increment the global change tick. Call once per frame.
    pub fn increment_tick(&mut self) {
        self.tick += 1;
    }

    /// Current global tick.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Number of alive entities.
    pub fn entity_count(&self) -> usize {
        self.allocator.alive_count()
    }
}

// ---------------------------------------------------------------------------
// System trait + scheduler
// ---------------------------------------------------------------------------

/// Pipeline stage for ordering systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SystemStage {
    /// Read input events, update InputState.
    Input = 0,
    /// Fixed-timestep physics simulation.
    Physics = 1,
    /// Gameplay logic (AI, scripting, game rules).
    GameLogic = 2,
    /// Submit draw commands, update cameras.
    Render = 3,
}

/// A system that operates on the world each frame.
pub trait System: Send {
    /// Run this system against the world.
    fn run(&mut self, world: &mut World);

    /// Which stage this system belongs to.
    fn stage(&self) -> SystemStage;

    /// Human-readable name for debugging.
    fn name(&self) -> &str;
}

/// Runs systems in stage order: Input → Physics → GameLogic → Render.
pub struct Scheduler {
    systems: Vec<Box<dyn System>>,
    sorted: bool,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            sorted: false,
        }
    }

    /// Add a system to the scheduler.
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
        self.sorted = false;
    }

    /// Run all systems in stage order against the world.
    pub fn run(&mut self, world: &mut World) {
        if !self.sorted {
            self.systems.sort_by_key(|s| s.stage());
            self.sorted = true;
        }
        for system in &mut self.systems {
            system.run(world);
        }
    }

    /// Number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// List system names in execution order.
    pub fn system_names(&mut self) -> Vec<&str> {
        if !self.sorted {
            self.systems.sort_by_key(|s| s.stage());
            self.sorted = true;
        }
        self.systems.iter().map(|s| s.name()).collect()
    }
}

/// Convenience: wrap a closure as a system.
pub struct FnSystem<F: FnMut(&mut World) + Send> {
    func: F,
    stage: SystemStage,
    name: String,
}

impl<F: FnMut(&mut World) + Send> FnSystem<F> {
    pub fn new(name: impl Into<String>, stage: SystemStage, func: F) -> Self {
        Self {
            func,
            stage,
            name: name.into(),
        }
    }
}

impl<F: FnMut(&mut World) + Send> System for FnSystem<F> {
    fn run(&mut self, world: &mut World) {
        (self.func)(world);
    }

    fn stage(&self) -> SystemStage {
        self.stage
    }

    fn name(&self) -> &str {
        &self.name
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
        self.channels.get(&TypeId::of::<E>()).map_or(0, |v| v.len())
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

    // -- Stress / edge case tests --

    #[test]
    fn stress_spawn_despawn_1000() {
        let mut world = World::new();
        let mut entities = Vec::new();
        for _ in 0..1000 {
            entities.push(world.spawn());
        }
        assert_eq!(world.entity_count(), 1000);

        // Despawn odd-indexed
        for i in (1..1000).step_by(2) {
            world.despawn(entities[i]).unwrap();
        }
        assert_eq!(world.entity_count(), 500);

        // Respawn into recycled slots
        for _ in 0..500 {
            let e = world.spawn();
            assert_eq!(e.generation(), 1); // recycled
        }
        assert_eq!(world.entity_count(), 1000);
    }

    #[test]
    fn has_component() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(!world.has_component::<Health>(e));
        world.insert_component(e, Health(42)).unwrap();
        assert!(world.has_component::<Health>(e));
        world.remove_component::<Health>(e);
        assert!(!world.has_component::<Health>(e));
    }

    #[test]
    fn resource_replacement() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));
        assert_eq!(world.get_resource::<Gravity>().unwrap().0, 9.81);

        world.insert_resource(Gravity(1.625));
        assert_eq!(world.get_resource::<Gravity>().unwrap().0, 1.625);
    }

    #[test]
    fn clock_spike_frame() {
        let mut clock = GameClock::with_timestep(1.0 / 60.0);
        clock.tick(0.5); // 500ms spike — 30 fixed steps pending
        assert_eq!(clock.pending_fixed_steps(), 30);
        let mut count = 0;
        while clock.consume_fixed() {
            count += 1;
        }
        assert_eq!(count, 30);
    }

    #[test]
    fn clock_zero_dt() {
        let mut clock = GameClock::default();
        clock.tick(0.0);
        assert_eq!(clock.frame, 1);
        assert_eq!(clock.delta, 0.0);
        assert!(!clock.consume_fixed());
    }

    #[test]
    fn event_bus_publish_after_drain() {
        let mut bus = EventBus::new();
        bus.publish(ScoreChanged(1));
        let _ = bus.drain::<ScoreChanged>();
        assert_eq!(bus.count::<ScoreChanged>(), 0);

        bus.publish(ScoreChanged(2));
        assert_eq!(bus.count::<ScoreChanged>(), 1);
        let events = bus.drain::<ScoreChanged>();
        assert_eq!(events[0].0, 2);
    }

    #[test]
    fn entity_boundary_values() {
        let e = Entity::new(u32::MAX, u32::MAX);
        assert_eq!(e.index(), u32::MAX);
        assert_eq!(e.generation(), u32::MAX);

        let e_zero = Entity::new(0, 0);
        assert_eq!(e_zero.id(), 0);
    }

    #[test]
    fn world_component_overwrite() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100)).unwrap();
        world.insert_component(e, Health(200)).unwrap();
        assert_eq!(world.get_component::<Health>(e).unwrap().0, 200);
    }

    #[test]
    fn alive_count_consistency() {
        let mut alloc = EntityAllocator::default();
        assert_eq!(alloc.alive_count(), 0);

        let e0 = alloc.spawn();
        let e1 = alloc.spawn();
        let e2 = alloc.spawn();
        assert_eq!(alloc.alive_count(), 3);

        alloc.despawn(e1).unwrap();
        assert_eq!(alloc.alive_count(), 2);

        alloc.despawn(e0).unwrap();
        alloc.despawn(e2).unwrap();
        assert_eq!(alloc.alive_count(), 0);

        // Respawn and verify
        let _ = alloc.spawn();
        assert_eq!(alloc.alive_count(), 1);
    }

    // -- System / Scheduler tests --

    #[test]
    fn scheduler_runs_in_stage_order() {
        use std::sync::{Arc, Mutex};

        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

        let log1 = log.clone();
        let log2 = log.clone();
        let log3 = log.clone();

        let mut scheduler = Scheduler::new();

        // Add in reverse order to verify sorting
        scheduler.add_system(Box::new(FnSystem::new(
            "render",
            SystemStage::Render,
            move |_| {
                log3.lock().unwrap().push("render");
            },
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "input",
            SystemStage::Input,
            move |_| {
                log1.lock().unwrap().push("input");
            },
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "logic",
            SystemStage::GameLogic,
            move |_| {
                log2.lock().unwrap().push("logic");
            },
        )));

        let mut world = World::new();
        scheduler.run(&mut world);

        let order = log.lock().unwrap();
        assert_eq!(*order, vec!["input", "logic", "render"]);
    }

    #[test]
    fn scheduler_system_names() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(Box::new(FnSystem::new(
            "physics",
            SystemStage::Physics,
            |_| {},
        )));
        scheduler.add_system(Box::new(FnSystem::new("input", SystemStage::Input, |_| {})));

        let names = scheduler.system_names();
        assert_eq!(names, vec!["input", "physics"]);
    }

    #[test]
    fn scheduler_system_modifies_world() {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(Box::new(FnSystem::new(
            "spawner",
            SystemStage::GameLogic,
            |world: &mut World| {
                world.spawn();
            },
        )));

        let mut world = World::new();
        assert_eq!(world.entity_count(), 0);
        scheduler.run(&mut world);
        assert_eq!(world.entity_count(), 1);
        scheduler.run(&mut world);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn system_stage_ordering() {
        assert!(SystemStage::Input < SystemStage::Physics);
        assert!(SystemStage::Physics < SystemStage::GameLogic);
        assert!(SystemStage::GameLogic < SystemStage::Render);
    }

    #[test]
    fn fn_system_basics() {
        let sys = FnSystem::new("test_sys", SystemStage::Input, |_: &mut World| {});
        assert_eq!(sys.name(), "test_sys");
        assert_eq!(sys.stage(), SystemStage::Input);
    }

    #[test]
    fn scheduler_all_four_stages() {
        use std::sync::{Arc, Mutex};

        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
        let (l1, l2, l3, l4) = (log.clone(), log.clone(), log.clone(), log.clone());

        let mut scheduler = Scheduler::new();
        scheduler.add_system(Box::new(FnSystem::new(
            "render",
            SystemStage::Render,
            move |_| {
                l4.lock().unwrap().push("render");
            },
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "physics",
            SystemStage::Physics,
            move |_| {
                l2.lock().unwrap().push("physics");
            },
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "input",
            SystemStage::Input,
            move |_| {
                l1.lock().unwrap().push("input");
            },
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "logic",
            SystemStage::GameLogic,
            move |_| {
                l3.lock().unwrap().push("logic");
            },
        )));

        let mut world = World::new();
        scheduler.run(&mut world);

        let order = log.lock().unwrap();
        assert_eq!(*order, vec!["input", "physics", "logic", "render"]);
    }

    #[test]
    fn scheduler_empty() {
        let mut scheduler = Scheduler::new();
        let mut world = World::new();
        scheduler.run(&mut world); // should not panic
        assert_eq!(scheduler.system_count(), 0);
    }

    #[test]
    fn scheduler_multiple_runs() {
        use std::sync::{Arc, Mutex};

        let count: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
        let c = count.clone();

        let mut scheduler = Scheduler::new();
        scheduler.add_system(Box::new(FnSystem::new(
            "counter",
            SystemStage::GameLogic,
            move |_| {
                *c.lock().unwrap() += 1;
            },
        )));

        let mut world = World::new();
        for _ in 0..5 {
            scheduler.run(&mut world);
        }
        assert_eq!(*count.lock().unwrap(), 5);
    }

    #[test]
    fn entity_from_id_roundtrip() {
        let e = Entity::new(42, 7);
        let id = e.id();
        let reconstructed = Entity::from_id(id);
        assert_eq!(e, reconstructed);
        assert_eq!(reconstructed.index(), 42);
        assert_eq!(reconstructed.generation(), 7);
    }

    #[test]
    fn get_component_mut_dead_entity() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100)).unwrap();
        world.despawn(e).unwrap();
        assert!(world.get_component_mut::<Health>(e).is_none());
    }

    #[test]
    fn resource_get_wrong_type() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));
        assert!(world.get_resource::<Health>().is_none());
    }

    #[test]
    fn world_is_alive() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.is_alive(e));
        world.despawn(e).unwrap();
        assert!(!world.is_alive(e));
    }

    #[test]
    fn remove_component_returns_none_if_missing() {
        let mut world = World::new();
        let e = world.spawn();
        assert!(world.remove_component::<Health>(e).is_none());
    }

    #[test]
    fn event_bus_large_batch() {
        let mut bus = EventBus::new();
        for i in 0..10000 {
            bus.publish(ScoreChanged(i));
        }
        assert_eq!(bus.count::<ScoreChanged>(), 10000);
        let events = bus.drain::<ScoreChanged>();
        assert_eq!(events.len(), 10000);
    }

    // -- Change detection tests --

    #[test]
    fn resource_change_detection_basic() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));

        // Just inserted — changed
        assert!(world.is_resource_changed::<Gravity>());

        // Clear — no longer changed
        world.clear_resource_changed::<Gravity>();
        assert!(!world.is_resource_changed::<Gravity>());
    }

    #[test]
    fn resource_change_on_mut_access() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));
        world.clear_resource_changed::<Gravity>();

        // Mutable access marks changed
        world.increment_tick();
        let g = world.get_resource_mut::<Gravity>().unwrap();
        g.0 = 1.625;

        assert!(world.is_resource_changed::<Gravity>());
    }

    #[test]
    fn resource_change_read_only_no_change() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));
        world.clear_resource_changed::<Gravity>();

        // Read-only access does NOT mark changed
        world.increment_tick();
        let _ = world.get_resource::<Gravity>();

        assert!(!world.is_resource_changed::<Gravity>());
    }

    #[test]
    fn resource_change_multi_tick() {
        let mut world = World::new();
        world.insert_resource(Gravity(9.81));

        // Tick 0: inserted
        world.clear_resource_changed::<Gravity>();

        // Tick 1: no mutation → not changed
        world.increment_tick();
        assert!(!world.is_resource_changed::<Gravity>());

        // Tick 2: mutate → changed
        world.increment_tick();
        world.get_resource_mut::<Gravity>().unwrap().0 = 0.0;
        assert!(world.is_resource_changed::<Gravity>());

        // Clear and tick 3: not changed
        world.clear_resource_changed::<Gravity>();
        world.increment_tick();
        assert!(!world.is_resource_changed::<Gravity>());
    }

    #[test]
    fn resource_change_untracked_type() {
        let world = World::new();
        // Never inserted → not changed
        assert!(!world.is_resource_changed::<Gravity>());
    }

    #[test]
    fn world_tick() {
        let mut world = World::new();
        assert_eq!(world.tick(), 0);
        world.increment_tick();
        assert_eq!(world.tick(), 1);
        world.increment_tick();
        assert_eq!(world.tick(), 2);
    }
}
