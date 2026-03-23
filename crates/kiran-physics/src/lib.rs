//! # kiran-physics — Impetus bridge for Kiran ECS
//!
//! Connects the impetus physics engine to kiran-core's ECS. Provides:
//! - Physics components (`RigidBody`, `Collider`, `PhysicsBody`)
//! - A `PhysicsEngine` resource wrapping `impetus::PhysicsWorld`
//! - A `physics_step` system function for the game loop
//! - Automatic sync between kiran `Position` and impetus body state
//!
//! ## Usage
//!
//! ```ignore
//! use kiran_physics::{PhysicsEngine, RigidBody, Collider, physics_step};
//!
//! // Store impetus world as a kiran resource
//! world.insert_resource(PhysicsEngine::new());
//!
//! // Spawn entity with physics
//! let entity = world.spawn();
//! world.insert_component(entity, Position(Vec3::new(0.0, 10.0, 0.0))).unwrap();
//! world.insert_component(entity, RigidBody::dynamic()).unwrap();
//! world.insert_component(entity, Collider::ball(0.5)).unwrap();
//!
//! // In game loop:
//! physics_step(&mut world, dt);
//! ```

use std::collections::HashMap;

use kiran_core::{Entity, World};

// ---------------------------------------------------------------------------
// Physics components (stored on kiran entities)
// ---------------------------------------------------------------------------

/// Rigid body component — marks an entity as physics-simulated.
#[derive(Debug, Clone)]
pub struct RigidBody {
    pub body_type: impetus::BodyType,
    pub linear_damping: f64,
    pub angular_damping: f64,
    pub fixed_rotation: bool,
    pub gravity_scale: Option<f64>,
}

impl RigidBody {
    /// Dynamic body — affected by gravity and forces.
    pub fn dynamic() -> Self {
        Self {
            body_type: impetus::BodyType::Dynamic,
            linear_damping: 0.0,
            angular_damping: 0.0,
            fixed_rotation: false,
            gravity_scale: None,
        }
    }

    /// Static body — immovable.
    pub fn fixed() -> Self {
        Self {
            body_type: impetus::BodyType::Static,
            linear_damping: 0.0,
            angular_damping: 0.0,
            fixed_rotation: false,
            gravity_scale: None,
        }
    }

    /// Kinematic body — user-controlled position.
    pub fn kinematic() -> Self {
        Self {
            body_type: impetus::BodyType::Kinematic,
            linear_damping: 0.0,
            angular_damping: 0.0,
            fixed_rotation: false,
            gravity_scale: None,
        }
    }

    pub fn with_damping(mut self, linear: f64, angular: f64) -> Self {
        self.linear_damping = linear;
        self.angular_damping = angular;
        self
    }

    pub fn with_fixed_rotation(mut self) -> Self {
        self.fixed_rotation = true;
        self
    }

    pub fn with_gravity_scale(mut self, scale: f64) -> Self {
        self.gravity_scale = Some(scale);
        self
    }
}

/// Collider component — defines the collision shape for a physics entity.
#[derive(Debug, Clone)]
pub struct Collider {
    pub shape: impetus::ColliderShape,
    pub offset: [f64; 3],
    pub material: impetus::PhysicsMaterial,
    pub is_sensor: bool,
    pub mass: Option<f64>,
}

impl Collider {
    pub fn ball(radius: f64) -> Self {
        Self {
            shape: impetus::ColliderShape::Ball { radius },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
        }
    }

    pub fn cuboid(hx: f64, hy: f64, hz: f64) -> Self {
        Self {
            shape: impetus::ColliderShape::Box {
                half_extents: [hx, hy, hz],
            },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
        }
    }

    pub fn capsule(half_height: f64, radius: f64) -> Self {
        Self {
            shape: impetus::ColliderShape::Capsule {
                half_height,
                radius,
            },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
        }
    }

    pub fn with_material(mut self, material: impetus::PhysicsMaterial) -> Self {
        self.material = material;
        self
    }

    pub fn with_offset(mut self, offset: [f64; 3]) -> Self {
        self.offset = offset;
        self
    }

    pub fn sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    pub fn with_mass(mut self, mass: f64) -> Self {
        self.mass = Some(mass);
        self
    }
}

/// Velocity component — readable/writable linear and angular velocity.
#[derive(Debug, Clone, Default)]
pub struct Velocity {
    pub linear: [f64; 3],
    pub angular: f64,
}

// ---------------------------------------------------------------------------
// Position component (kiran-scene already has one, but it uses glam::Vec3 f32)
// We use f64 arrays matching impetus's API.
// ---------------------------------------------------------------------------

/// Physics position — f64 precision. Updated by the physics engine each step.
#[derive(Debug, Clone)]
pub struct PhysicsPosition {
    pub position: [f64; 3],
    pub rotation: f64,
}

impl Default for PhysicsPosition {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Physics engine resource
// ---------------------------------------------------------------------------

/// The physics engine resource — wraps an impetus PhysicsWorld.
/// Stored as a kiran resource via `world.insert_resource(PhysicsEngine::new())`.
pub struct PhysicsEngine {
    pub physics: impetus::PhysicsWorld,
    /// Maps kiran entity → impetus BodyHandle
    entity_to_body: HashMap<Entity, impetus::BodyHandle>,
    /// Maps impetus BodyHandle → kiran entity
    body_to_entity: HashMap<impetus::BodyHandle, Entity>,
    /// Maps kiran entity → impetus ColliderHandle
    entity_to_collider: HashMap<Entity, impetus::ColliderHandle>,
}

impl PhysicsEngine {
    /// Create a new physics engine with default configuration.
    pub fn new() -> Self {
        Self::with_config(impetus::WorldConfig::default())
    }

    /// Create with custom configuration.
    pub fn with_config(config: impetus::WorldConfig) -> Self {
        Self {
            physics: impetus::PhysicsWorld::new(config),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
            entity_to_collider: HashMap::new(),
        }
    }

    /// Register a kiran entity with the physics engine.
    /// Call this after inserting RigidBody + Collider components.
    pub fn register(&mut self, entity: Entity, rb: &RigidBody, pos: &PhysicsPosition, collider: &Collider) {
        let body_handle = self.physics.add_body(impetus::BodyDesc {
            body_type: rb.body_type,
            position: pos.position,
            rotation: pos.rotation,
            linear_velocity: [0.0, 0.0, 0.0],
            angular_velocity: 0.0,
            linear_damping: rb.linear_damping,
            angular_damping: rb.angular_damping,
            fixed_rotation: rb.fixed_rotation,
            gravity_scale: rb.gravity_scale,
        });

        let collider_handle = self.physics.add_collider(
            body_handle,
            impetus::ColliderDesc {
                shape: collider.shape.clone(),
                offset: collider.offset,
                material: collider.material.clone(),
                is_sensor: collider.is_sensor,
                mass: collider.mass,
            },
        );

        self.entity_to_body.insert(entity, body_handle);
        self.body_to_entity.insert(body_handle, entity);
        self.entity_to_collider.insert(entity, collider_handle);
    }

    /// Unregister a kiran entity from the physics engine.
    pub fn unregister(&mut self, entity: Entity) {
        if let Some(body_handle) = self.entity_to_body.remove(&entity) {
            let _ = self.physics.remove_body(body_handle);
            self.body_to_entity.remove(&body_handle);
        }
        self.entity_to_collider.remove(&entity);
    }

    /// Apply a force to an entity's physics body.
    pub fn apply_force(&mut self, entity: Entity, force: impetus::Force) {
        if let Some(&handle) = self.entity_to_body.get(&entity) {
            self.physics.apply_force(handle, force);
        }
    }

    /// Apply an impulse to an entity's physics body.
    pub fn apply_impulse(&mut self, entity: Entity, impulse: impetus::Impulse) {
        if let Some(&handle) = self.entity_to_body.get(&entity) {
            self.physics.apply_impulse(handle, impulse);
        }
    }

    /// Get the impetus body handle for a kiran entity.
    pub fn body_handle(&self, entity: Entity) -> Option<impetus::BodyHandle> {
        self.entity_to_body.get(&entity).copied()
    }

    /// Get the kiran entity ID for an impetus body handle.
    pub fn entity_for_body(&self, handle: impetus::BodyHandle) -> Option<Entity> {
        self.body_to_entity.get(&handle).copied()
    }

    /// Get collision events from the last step, mapped to kiran entity IDs.
    pub fn collision_events(&self) -> Vec<PhysicsCollisionEvent> {
        self.physics
            .collision_events()
            .iter()
            .filter_map(|event| match event {
                impetus::CollisionEvent::Started {
                    collider_a,
                    collider_b,
                } => {
                    let entity_a = self.find_entity_for_collider(*collider_a)?;
                    let entity_b = self.find_entity_for_collider(*collider_b)?;
                    Some(PhysicsCollisionEvent::Started {
                        entity_a,
                        entity_b,
                    })
                }
                impetus::CollisionEvent::Stopped {
                    collider_a,
                    collider_b,
                } => {
                    let entity_a = self.find_entity_for_collider(*collider_a)?;
                    let entity_b = self.find_entity_for_collider(*collider_b)?;
                    Some(PhysicsCollisionEvent::Stopped {
                        entity_a,
                        entity_b,
                    })
                }
            })
            .collect()
    }

    fn find_entity_for_collider(&self, collider: impetus::ColliderHandle) -> Option<Entity> {
        self.entity_to_collider
            .iter()
            .find(|(_, ch)| **ch == collider)
            .map(|(eid, _)| *eid)
    }
}

impl Default for PhysicsEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Collision events (kiran-facing)
// ---------------------------------------------------------------------------

/// Physics collision event — uses kiran entity IDs instead of impetus handles.
#[derive(Debug, Clone)]
pub enum PhysicsCollisionEvent {
    Started { entity_a: Entity, entity_b: Entity },
    Stopped { entity_a: Entity, entity_b: Entity },
}

// ---------------------------------------------------------------------------
// System function
// ---------------------------------------------------------------------------

/// Step the physics simulation and sync positions back to kiran components.
///
/// Call this from your game loop:
/// ```ignore
/// while clock.consume_fixed() {
///     physics_step(&mut world, clock.fixed_timestep);
/// }
/// ```
pub fn physics_step(world: &mut World) {
    // Step impetus
    let events = {
        let engine = match world.get_resource_mut::<PhysicsEngine>() {
            Some(e) => e,
            None => return,
        };
        engine.physics.step();

        // Collect collision events
        let events = engine.collision_events();

        // Read back positions from impetus into a buffer
        // (can't mutate world components while borrowing engine)
        type BodyUpdate = (Entity, [f64; 3], f64, [f64; 3], f64);
        let updates: Vec<BodyUpdate> = engine
            .entity_to_body
            .iter()
            .filter_map(|(&entity, &body_handle)| {
                let state = engine.physics.get_body_state(body_handle).ok()?;
                Some((
                    entity,
                    state.position,
                    state.rotation,
                    state.linear_velocity,
                    state.angular_velocity,
                ))
            })
            .collect();

        // Write positions back
        for (entity, position, rotation, linear_vel, angular_vel) in updates {
            if let Some(pos) = world.get_component_mut::<PhysicsPosition>(entity) {
                pos.position = position;
                pos.rotation = rotation;
            }
            if let Some(vel) = world.get_component_mut::<Velocity>(entity) {
                vel.linear = linear_vel;
                vel.angular = angular_vel;
            }
        }

        events
    };

    // Publish collision events to kiran event bus
    if let Some(bus) = world.get_resource_mut::<kiran_core::EventBus>() {
        for event in events {
            bus.publish(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kiran_core::World;

    #[test]
    fn create_physics_engine() {
        let engine = PhysicsEngine::new();
        assert_eq!(engine.physics.body_count(), 0);
    }

    fn test_entity(index: u32) -> Entity {
        Entity::new(index, 0)
    }

    #[test]
    fn register_entity() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(42);

        engine.register(e, &RigidBody::dynamic(), &PhysicsPosition {
            position: [0.0, 10.0, 0.0],
            rotation: 0.0,
        }, &Collider::ball(0.5));

        assert_eq!(engine.physics.body_count(), 1);
        assert!(engine.body_handle(e).is_some());
        assert_eq!(engine.entity_for_body(engine.body_handle(e).unwrap()), Some(e));
    }

    #[test]
    fn unregister_entity() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(1);
        engine.register(e, &RigidBody::dynamic(), &PhysicsPosition::default(), &Collider::ball(1.0));
        assert_eq!(engine.physics.body_count(), 1);

        engine.unregister(e);
        assert_eq!(engine.physics.body_count(), 0);
        assert!(engine.body_handle(e).is_none());
    }

    #[test]
    fn physics_step_updates_position() {
        let mut world = World::new();
        world.insert_resource(PhysicsEngine::new());
        world.insert_resource(kiran_core::EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(entity, PhysicsPosition {
                position: [0.0, 10.0, 0.0],
                rotation: 0.0,
            })
            .unwrap();
        world
            .insert_component(entity, Velocity::default())
            .unwrap();

        {
            let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
            engine.register(entity, &RigidBody::dynamic(), &PhysicsPosition {
                position: [0.0, 10.0, 0.0],
                rotation: 0.0,
            }, &Collider::ball(0.5));
        }

        for _ in 0..60 {
            physics_step(&mut world);
        }

        let pos = world.get_component::<PhysicsPosition>(entity).unwrap();
        assert!(pos.position[1] < 10.0, "body should have fallen under gravity");
    }

    #[test]
    fn component_builders() {
        let rb = RigidBody::dynamic()
            .with_damping(0.1, 0.05)
            .with_fixed_rotation()
            .with_gravity_scale(0.5);
        assert_eq!(rb.linear_damping, 0.1);
        assert!(rb.fixed_rotation);
        assert_eq!(rb.gravity_scale, Some(0.5));

        let col = Collider::cuboid(1.0, 2.0, 3.0)
            .with_material(impetus::PhysicsMaterial::rubber())
            .with_offset([0.0, 1.0, 0.0])
            .sensor();
        assert!(col.is_sensor);
        assert_eq!(col.offset, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn apply_force_to_entity() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(1);
        engine.register(e, &RigidBody::dynamic(), &PhysicsPosition::default(), &Collider::ball(1.0));

        engine.apply_force(e, impetus::Force::new(10.0, 0.0, 0.0));
        engine.apply_impulse(e, impetus::Impulse::new(0.0, 5.0, 0.0));
        engine.physics.step();
    }
}
