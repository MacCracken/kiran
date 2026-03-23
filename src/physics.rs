//! Impetus physics engine bridge
//!
//! Connects the impetus physics engine to kiran's ECS. Provides:
//! - Physics components (`RigidBody`, `Collider`, `Velocity`, `PhysicsPosition`)
//! - A `PhysicsEngine` resource wrapping `impetus::PhysicsWorld`
//! - A `physics_step` system function for the game loop

use std::collections::HashMap;

use crate::world::{Entity, EventBus, World};

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
    pub collision_layer: u32,
    pub collision_mask: u32,
}

impl Collider {
    pub fn ball(radius: f64) -> Self {
        Self {
            shape: impetus::ColliderShape::Ball { radius },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
            collision_layer: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
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
            collision_layer: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
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
            collision_layer: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
        }
    }

    /// Segment collider — a line between two points.
    pub fn segment(a: [f64; 3], b: [f64; 3]) -> Self {
        Self {
            shape: impetus::ColliderShape::Segment { a, b },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
            collision_layer: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
        }
    }

    /// Convex hull from a set of points.
    pub fn convex_hull(points: Vec<[f64; 3]>) -> Self {
        Self {
            shape: impetus::ColliderShape::ConvexHull { points },
            offset: [0.0, 0.0, 0.0],
            material: impetus::PhysicsMaterial::default(),
            is_sensor: false,
            mass: None,
            collision_layer: 0xFFFF_FFFF,
            collision_mask: 0xFFFF_FFFF,
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

    pub fn with_layer(mut self, layer: u32) -> Self {
        self.collision_layer = layer;
        self
    }

    pub fn with_mask(mut self, mask: u32) -> Self {
        self.collision_mask = mask;
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
// Position component (f64 precision for physics)
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
    /// Maps kiran entity -> impetus BodyHandle
    entity_to_body: HashMap<Entity, impetus::BodyHandle>,
    /// Maps impetus BodyHandle -> kiran entity
    body_to_entity: HashMap<impetus::BodyHandle, Entity>,
    /// Maps kiran entity -> impetus ColliderHandle
    entity_to_collider: HashMap<Entity, impetus::ColliderHandle>,
    /// Maps impetus ColliderHandle -> kiran entity (reverse lookup, O(1))
    collider_to_entity: HashMap<impetus::ColliderHandle, Entity>,
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
            collider_to_entity: HashMap::new(),
        }
    }

    /// Register a kiran entity with the physics engine.
    pub fn register(
        &mut self,
        entity: Entity,
        rb: &RigidBody,
        pos: &PhysicsPosition,
        collider: &Collider,
    ) {
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
                collision_layer: collider.collision_layer,
                collision_mask: collider.collision_mask,
            },
        );

        self.entity_to_body.insert(entity, body_handle);
        self.body_to_entity.insert(body_handle, entity);
        self.entity_to_collider.insert(entity, collider_handle);
        self.collider_to_entity.insert(collider_handle, entity);
    }

    /// Unregister a kiran entity from the physics engine.
    pub fn unregister(&mut self, entity: Entity) {
        if let Some(body_handle) = self.entity_to_body.remove(&entity) {
            let _ = self.physics.remove_body(body_handle);
            self.body_to_entity.remove(&body_handle);
        }
        if let Some(collider_handle) = self.entity_to_collider.remove(&entity) {
            self.collider_to_entity.remove(&collider_handle);
        }
    }

    /// Number of registered entities.
    pub fn entity_count(&self) -> usize {
        self.entity_to_body.len()
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

    /// Cast a ray and return the first entity hit.
    pub fn raycast(
        &self,
        origin: [f64; 3],
        direction: [f64; 3],
        max_dist: f64,
    ) -> Option<RaycastHit> {
        let hit = self.physics.raycast(origin, direction, max_dist)?;
        let entity = self.find_entity_for_collider(hit.collider)?;
        Some(RaycastHit {
            entity,
            point: hit.point,
            normal: hit.normal,
            distance: hit.distance,
        })
    }

    /// Spawn a particle in the physics world.
    pub fn spawn_particle(&mut self, particle: impetus::Particle) -> impetus::ParticleHandle {
        self.physics.spawn_particle(particle)
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
                    Some(PhysicsCollisionEvent::Started { entity_a, entity_b })
                }
                impetus::CollisionEvent::Stopped {
                    collider_a,
                    collider_b,
                } => {
                    let entity_a = self.find_entity_for_collider(*collider_a)?;
                    let entity_b = self.find_entity_for_collider(*collider_b)?;
                    Some(PhysicsCollisionEvent::Stopped { entity_a, entity_b })
                }
                _ => None,
            })
            .collect()
    }

    fn find_entity_for_collider(&self, collider: impetus::ColliderHandle) -> Option<Entity> {
        self.collider_to_entity.get(&collider).copied()
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
// Debug rendering
// ---------------------------------------------------------------------------

/// A debug wireframe shape for visualization.
#[derive(Debug, Clone)]
pub struct DebugShape {
    pub entity: Entity,
    pub kind: DebugShapeKind,
    pub position: [f64; 3],
    pub rotation: f64,
}

/// The kind of debug wireframe shape.
#[derive(Debug, Clone)]
pub enum DebugShapeKind {
    Circle { radius: f64 },
    Box { half_extents: [f64; 3] },
    Capsule { half_height: f64, radius: f64 },
    Segment { a: [f64; 3], b: [f64; 3] },
}

impl PhysicsEngine {
    /// Generate debug wireframe shapes for all registered colliders.
    pub fn debug_shapes(&self, world: &World) -> Vec<DebugShape> {
        let mut shapes = Vec::new();

        for &entity in self.entity_to_body.keys() {
            let Some(collider) = world.get_component::<Collider>(entity) else {
                continue;
            };

            let (position, rotation) = world
                .get_component::<PhysicsPosition>(entity)
                .map(|p| (p.position, p.rotation))
                .unwrap_or(([0.0, 0.0, 0.0], 0.0));

            let kind = match &collider.shape {
                impetus::ColliderShape::Ball { radius } => {
                    DebugShapeKind::Circle { radius: *radius }
                }
                impetus::ColliderShape::Box { half_extents } => DebugShapeKind::Box {
                    half_extents: *half_extents,
                },
                impetus::ColliderShape::Capsule {
                    half_height,
                    radius,
                } => DebugShapeKind::Capsule {
                    half_height: *half_height,
                    radius: *radius,
                },
                impetus::ColliderShape::Segment { a, b } => {
                    DebugShapeKind::Segment { a: *a, b: *b }
                }
                _ => continue,
            };

            shapes.push(DebugShape {
                entity,
                kind,
                position,
                rotation,
            });
        }

        shapes
    }
}

// ---------------------------------------------------------------------------
// Raycast result
// ---------------------------------------------------------------------------

/// Result of a raycast query, mapped to kiran entity IDs.
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub entity: Entity,
    pub point: [f64; 3],
    pub normal: [f64; 3],
    pub distance: f64,
}

// ---------------------------------------------------------------------------
// System function
// ---------------------------------------------------------------------------

/// Step the physics simulation and sync positions back to kiran components.
///
/// Call this from your game loop:
/// ```ignore
/// while clock.consume_fixed() {
///     physics_step(&mut world);
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
    if let Some(bus) = world.get_resource_mut::<EventBus>() {
        for event in events {
            bus.publish(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        engine.register(
            e,
            &RigidBody::dynamic(),
            &PhysicsPosition {
                position: [0.0, 10.0, 0.0],
                rotation: 0.0,
            },
            &Collider::ball(0.5),
        );

        assert_eq!(engine.physics.body_count(), 1);
        assert!(engine.body_handle(e).is_some());
        assert_eq!(
            engine.entity_for_body(engine.body_handle(e).unwrap()),
            Some(e)
        );
    }

    #[test]
    fn unregister_entity() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(1);
        engine.register(
            e,
            &RigidBody::dynamic(),
            &PhysicsPosition::default(),
            &Collider::ball(1.0),
        );
        assert_eq!(engine.physics.body_count(), 1);

        engine.unregister(e);
        assert_eq!(engine.physics.body_count(), 0);
        assert!(engine.body_handle(e).is_none());
    }

    #[test]
    fn physics_step_updates_position() {
        let mut world = World::new();
        world.insert_resource(PhysicsEngine::new());
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(
                entity,
                PhysicsPosition {
                    position: [0.0, 10.0, 0.0],
                    rotation: 0.0,
                },
            )
            .unwrap();
        world.insert_component(entity, Velocity::default()).unwrap();

        {
            let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
            engine.register(
                entity,
                &RigidBody::dynamic(),
                &PhysicsPosition {
                    position: [0.0, 10.0, 0.0],
                    rotation: 0.0,
                },
                &Collider::ball(0.5),
            );
        }

        for _ in 0..60 {
            physics_step(&mut world);
        }

        let pos = world.get_component::<PhysicsPosition>(entity).unwrap();
        assert!(
            pos.position[1] < 10.0,
            "body should have fallen under gravity"
        );
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
        engine.register(
            e,
            &RigidBody::dynamic(),
            &PhysicsPosition::default(),
            &Collider::ball(1.0),
        );

        engine.apply_force(e, impetus::Force::new(10.0, 0.0, 0.0));
        engine.apply_impulse(e, impetus::Impulse::new(0.0, 5.0, 0.0));
        engine.physics.step();
    }

    #[test]
    fn entity_count() {
        let mut engine = PhysicsEngine::new();
        assert_eq!(engine.entity_count(), 0);

        let e1 = test_entity(1);
        engine.register(
            e1,
            &RigidBody::dynamic(),
            &PhysicsPosition::default(),
            &Collider::ball(1.0),
        );
        assert_eq!(engine.entity_count(), 1);

        let e2 = test_entity(2);
        engine.register(
            e2,
            &RigidBody::fixed(),
            &PhysicsPosition::default(),
            &Collider::cuboid(1.0, 1.0, 1.0),
        );
        assert_eq!(engine.entity_count(), 2);

        engine.unregister(e1);
        assert_eq!(engine.entity_count(), 1);
    }

    #[test]
    fn debug_shapes_basic() {
        let mut world = World::new();
        let mut engine = PhysicsEngine::new();

        let e = world.spawn();
        let rb = RigidBody::dynamic();
        let col = Collider::ball(2.0);
        let pos = PhysicsPosition {
            position: [5.0, 10.0, 0.0],
            rotation: 0.5,
        };

        world.insert_component(e, col.clone()).unwrap();
        world.insert_component(e, pos.clone()).unwrap();

        engine.register(e, &rb, &pos, &col);
        world.insert_resource(engine);

        let engine = world.get_resource::<PhysicsEngine>().unwrap();
        let shapes = engine.debug_shapes(&world);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].entity, e);
        assert_eq!(shapes[0].position, [5.0, 10.0, 0.0]);

        match &shapes[0].kind {
            DebugShapeKind::Circle { radius } => assert_eq!(*radius, 2.0),
            _ => panic!("expected circle"),
        }
    }

    #[test]
    fn debug_shapes_multiple_types() {
        let mut world = World::new();
        let mut engine = PhysicsEngine::new();

        // Ball
        let e1 = world.spawn();
        let rb1 = RigidBody::dynamic();
        let col1 = Collider::ball(1.0);
        let pos1 = PhysicsPosition::default();
        world.insert_component(e1, col1.clone()).unwrap();
        world.insert_component(e1, pos1.clone()).unwrap();
        engine.register(e1, &rb1, &pos1, &col1);

        // Box
        let e2 = world.spawn();
        let rb2 = RigidBody::fixed();
        let col2 = Collider::cuboid(2.0, 3.0, 4.0);
        let pos2 = PhysicsPosition::default();
        world.insert_component(e2, col2.clone()).unwrap();
        world.insert_component(e2, pos2.clone()).unwrap();
        engine.register(e2, &rb2, &pos2, &col2);

        world.insert_resource(engine);

        let engine = world.get_resource::<PhysicsEngine>().unwrap();
        let shapes = engine.debug_shapes(&world);
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn collider_with_layer_mask() {
        let col = Collider::ball(1.0).with_layer(0x01).with_mask(0x02);
        assert_eq!(col.collision_layer, 0x01);
        assert_eq!(col.collision_mask, 0x02);
    }

    #[test]
    fn collider_with_mass_builder() {
        let col = Collider::ball(1.0).with_mass(5.0);
        assert_eq!(col.mass, Some(5.0));
    }

    #[test]
    fn debug_shapes_capsule() {
        let mut world = World::new();
        let mut engine = PhysicsEngine::new();

        let e = world.spawn();
        let rb = RigidBody::dynamic();
        let col = Collider::capsule(0.8, 0.3);
        let pos = PhysicsPosition::default();

        world.insert_component(e, col.clone()).unwrap();
        world.insert_component(e, pos.clone()).unwrap();
        engine.register(e, &rb, &pos, &col);
        world.insert_resource(engine);

        let engine = world.get_resource::<PhysicsEngine>().unwrap();
        let shapes = engine.debug_shapes(&world);
        assert_eq!(shapes.len(), 1);

        match &shapes[0].kind {
            DebugShapeKind::Capsule {
                half_height,
                radius,
            } => {
                assert_eq!(*half_height, 0.8);
                assert_eq!(*radius, 0.3);
            }
            _ => panic!("expected capsule"),
        }
    }

    #[test]
    fn debug_shapes_empty_engine() {
        let world = World::new();
        let engine = PhysicsEngine::new();
        let shapes = engine.debug_shapes(&world);
        assert!(shapes.is_empty());
    }

    #[test]
    fn debug_shapes_no_collider_component() {
        let mut world = World::new();
        let mut engine = PhysicsEngine::new();

        let e = world.spawn();
        let rb = RigidBody::dynamic();
        let col = Collider::ball(1.0);
        let pos = PhysicsPosition::default();

        // Register with engine but don't store Collider as ECS component
        world.insert_component(e, pos.clone()).unwrap();
        engine.register(e, &rb, &pos, &col);
        world.insert_resource(engine);

        let engine = world.get_resource::<PhysicsEngine>().unwrap();
        let shapes = engine.debug_shapes(&world);
        // No collider component → no debug shape
        assert!(shapes.is_empty());
    }

    #[test]
    fn raycast_miss() {
        let engine = PhysicsEngine::new();
        // No bodies → no hit
        let hit = engine.raycast([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 100.0);
        assert!(hit.is_none());
    }

    #[test]
    fn raycast_hit() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(1);
        engine.register(
            e,
            &RigidBody::fixed(),
            &PhysicsPosition {
                position: [10.0, 0.0, 0.0],
                rotation: 0.0,
            },
            &Collider::ball(1.0),
        );

        // Ray from origin toward +x should hit the ball at x=10
        let hit = engine.raycast([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 100.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.entity, e);
        assert!(hit.distance > 0.0);
        assert!(hit.distance < 100.0);
    }

    #[test]
    fn spawn_particle_basic() {
        let mut engine = PhysicsEngine::new();
        let _handle = engine.spawn_particle(impetus::Particle::new(
            [0.0, 10.0, 0.0],
            [0.0, 0.0, 0.0],
            5.0,
        ));
        // Should not panic
    }

    #[test]
    fn physics_step_updates_velocity() {
        let mut world = World::new();
        world.insert_resource(PhysicsEngine::new());
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        world
            .insert_component(
                entity,
                PhysicsPosition {
                    position: [0.0, 10.0, 0.0],
                    rotation: 0.0,
                },
            )
            .unwrap();
        world.insert_component(entity, Velocity::default()).unwrap();

        {
            let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
            engine.register(
                entity,
                &RigidBody::dynamic(),
                &PhysicsPosition {
                    position: [0.0, 10.0, 0.0],
                    rotation: 0.0,
                },
                &Collider::ball(0.5),
            );
        }

        // Step once — gravity should give velocity
        physics_step(&mut world);

        let vel = world.get_component::<Velocity>(entity).unwrap();
        // Should have negative y velocity (falling)
        assert!(vel.linear[1] < 0.0, "body should have downward velocity");
    }

    #[test]
    fn apply_force_to_nonexistent_entity() {
        let mut engine = PhysicsEngine::new();
        let fake = test_entity(999);
        // Should not panic
        engine.apply_force(fake, impetus::Force::new(1.0, 0.0, 0.0));
        engine.apply_impulse(fake, impetus::Impulse::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn unregister_nonexistent_entity() {
        let mut engine = PhysicsEngine::new();
        let fake = test_entity(999);
        engine.unregister(fake); // should not panic
        assert_eq!(engine.entity_count(), 0);
    }

    #[test]
    fn lookup_unregistered_entity() {
        let engine = PhysicsEngine::new();
        let fake = test_entity(42);
        assert!(engine.body_handle(fake).is_none());
        assert!(engine.entity_for_body(impetus::BodyHandle(999)).is_none());
    }

    #[test]
    fn static_body_does_not_fall() {
        let mut world = World::new();
        world.insert_resource(PhysicsEngine::new());
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        let pos = PhysicsPosition {
            position: [0.0, 5.0, 0.0],
            rotation: 0.0,
        };
        world.insert_component(entity, pos.clone()).unwrap();
        world.insert_component(entity, Velocity::default()).unwrap();

        {
            let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
            engine.register(entity, &RigidBody::fixed(), &pos, &Collider::ball(1.0));
        }

        for _ in 0..60 {
            physics_step(&mut world);
        }

        let final_pos = world.get_component::<PhysicsPosition>(entity).unwrap();
        assert!(
            (final_pos.position[1] - 5.0).abs() < 0.001,
            "static body should not move"
        );
    }

    #[test]
    fn physics_step_no_engine_resource() {
        let mut world = World::new();
        // No PhysicsEngine resource — should not panic
        physics_step(&mut world);
    }

    #[test]
    fn register_multiple_body_types() {
        let mut engine = PhysicsEngine::new();

        let e1 = test_entity(1);
        engine.register(
            e1,
            &RigidBody::dynamic(),
            &PhysicsPosition::default(),
            &Collider::ball(1.0),
        );
        let e2 = test_entity(2);
        engine.register(
            e2,
            &RigidBody::fixed(),
            &PhysicsPosition::default(),
            &Collider::cuboid(1.0, 1.0, 1.0),
        );
        let e3 = test_entity(3);
        engine.register(
            e3,
            &RigidBody::kinematic(),
            &PhysicsPosition::default(),
            &Collider::capsule(0.5, 0.25),
        );

        assert_eq!(engine.entity_count(), 3);
        assert_eq!(engine.physics.body_count(), 3);
    }

    // -- 3D-specific tests --

    #[test]
    fn collider_segment() {
        let col = Collider::segment([0.0, 0.0, 0.0], [10.0, 5.0, 0.0]);
        match &col.shape {
            impetus::ColliderShape::Segment { a, b } => {
                assert_eq!(*a, [0.0, 0.0, 0.0]);
                assert_eq!(*b, [10.0, 5.0, 0.0]);
            }
            _ => panic!("expected segment"),
        }
    }

    #[test]
    fn collider_convex_hull() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let col = Collider::convex_hull(points.clone());
        match &col.shape {
            impetus::ColliderShape::ConvexHull { points: pts } => {
                assert_eq!(pts.len(), 3);
            }
            _ => panic!("expected convex hull"),
        }
    }

    #[test]
    fn register_entity_3d_position() {
        let mut engine = PhysicsEngine::new();
        let e = test_entity(1);
        engine.register(
            e,
            &RigidBody::dynamic(),
            &PhysicsPosition {
                position: [5.0, 10.0, 15.0],
                rotation: 0.0,
            },
            &Collider::ball(1.0),
        );

        let state = engine
            .physics
            .get_body_state(engine.body_handle(e).unwrap())
            .unwrap();
        assert_eq!(state.position[0], 5.0);
        assert_eq!(state.position[1], 10.0);
        assert_eq!(state.position[2], 15.0);
    }

    #[test]
    fn physics_step_3d_gravity() {
        let mut world = World::new();
        world.insert_resource(PhysicsEngine::new());
        world.insert_resource(EventBus::new());

        let entity = world.spawn();
        let pos = PhysicsPosition {
            position: [5.0, 20.0, -3.0],
            rotation: 0.0,
        };
        world.insert_component(entity, pos.clone()).unwrap();
        world.insert_component(entity, Velocity::default()).unwrap();

        {
            let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
            engine.register(entity, &RigidBody::dynamic(), &pos, &Collider::ball(0.5));
        }

        for _ in 0..60 {
            physics_step(&mut world);
        }

        let final_pos = world.get_component::<PhysicsPosition>(entity).unwrap();
        // Y should have fallen, X and Z should be unchanged
        assert!(final_pos.position[1] < 20.0, "should fall under gravity");
        assert!((final_pos.position[0] - 5.0).abs() < 0.01, "X unchanged");
        assert!((final_pos.position[2] - (-3.0)).abs() < 0.01, "Z unchanged");
    }

    #[test]
    fn debug_shapes_segment() {
        let mut world = World::new();
        let mut engine = PhysicsEngine::new();

        let e = world.spawn();
        let rb = RigidBody::fixed();
        let col = Collider::segment([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
        let pos = PhysicsPosition::default();

        world.insert_component(e, col.clone()).unwrap();
        world.insert_component(e, pos.clone()).unwrap();
        engine.register(e, &rb, &pos, &col);
        world.insert_resource(engine);

        let engine = world.get_resource::<PhysicsEngine>().unwrap();
        let shapes = engine.debug_shapes(&world);
        assert_eq!(shapes.len(), 1);

        match &shapes[0].kind {
            DebugShapeKind::Segment { a, b } => {
                assert_eq!(*a, [0.0, 0.0, 0.0]);
                assert_eq!(*b, [10.0, 0.0, 0.0]);
            }
            _ => panic!("expected segment debug shape"),
        }
    }
}
