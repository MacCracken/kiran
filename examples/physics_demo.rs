//! Example: Spawn entities with rigid bodies and colliders, run physics, query positions.

use kiran::physics::{Collider, PhysicsEngine, PhysicsPosition, RigidBody, physics_step};
use kiran::{GameClock, World};

fn main() {
    let mut world = World::new();
    world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
    world.insert_resource(PhysicsEngine::new());

    // Spawn a static floor
    let floor = world.spawn();
    world.insert_component(floor, RigidBody::fixed()).unwrap();
    world
        .insert_component(floor, Collider::cuboid(50.0, 0.5, 50.0))
        .unwrap();
    world
        .insert_component(floor, PhysicsPosition::default())
        .unwrap();

    // Spawn a dynamic ball above the floor
    let ball = world.spawn();
    world.insert_component(ball, RigidBody::dynamic()).unwrap();
    world.insert_component(ball, Collider::ball(0.5)).unwrap();
    world
        .insert_component(
            ball,
            PhysicsPosition {
                position: [0.0, 10.0, 0.0],
                rotation: 0.0,
            },
        )
        .unwrap();

    // Register both entities with the physics engine
    let floor_rb = world.get_component::<RigidBody>(floor).unwrap().clone();
    let floor_col = world.get_component::<Collider>(floor).unwrap().clone();
    let floor_pos = world
        .get_component::<PhysicsPosition>(floor)
        .unwrap()
        .clone();
    let ball_rb = world.get_component::<RigidBody>(ball).unwrap().clone();
    let ball_col = world.get_component::<Collider>(ball).unwrap().clone();
    let ball_pos = world
        .get_component::<PhysicsPosition>(ball)
        .unwrap()
        .clone();

    let engine = world.get_resource_mut::<PhysicsEngine>().unwrap();
    engine.register(floor, &floor_rb, &floor_pos, &floor_col);
    engine.register(ball, &ball_rb, &ball_pos, &ball_col);

    println!("Physics demo — dropping a ball from y=10.0");
    println!(
        "Registered {} entities with physics engine",
        engine.entity_count()
    );

    // Step physics for 120 frames (~2 seconds at 60fps)
    for frame in 0..120 {
        physics_step(&mut world);
        if frame % 20 == 0 {
            let pos = world.get_component::<PhysicsPosition>(ball).unwrap();
            println!("  frame {:>3}: ball y = {:.4}", frame, pos.position[1]);
        }
    }

    let final_pos = world.get_component::<PhysicsPosition>(ball).unwrap();
    println!(
        "Final ball position: ({:.4}, {:.4}, {:.4})",
        final_pos.position[0], final_pos.position[1], final_pos.position[2]
    );
}
