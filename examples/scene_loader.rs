//! Example: Load a TOML scene, spawn entities, walk the hierarchy.

use kiran::scene::{Children, LightComponent, Name, Position, Tags, load_scene, spawn_scene};
use kiran::{GameClock, World};

fn main() {
    let scene_toml = r#"
name = "Example Scene"
description = "A scene demonstrating hierarchy, prefabs, and components"

[[prefabs]]
name = "tree"
tags = ["vegetation"]

[[entities]]
name = "Sun"
position = [0.0, 100.0, 0.0]
light_intensity = 1.5

[[entities]]
name = "Player"
position = [5.0, 0.0, 3.0]
tags = ["controllable", "hero"]
[[entities.children]]
name = "Sword"
position = [0.5, 0.0, 0.0]

[[entities]]
name = "Oak"
position = [10.0, 0.0, -5.0]
prefab = "tree"
"#;

    // Load and spawn
    let scene = load_scene(scene_toml).unwrap();
    let mut world = World::new();
    world.insert_resource(GameClock::with_timestep(1.0 / 60.0));

    let entities = spawn_scene(&mut world, &scene).unwrap();

    println!(
        "Scene: {} ({} top-level entities)",
        scene.name,
        entities.len()
    );
    println!("World: {} total entities", world.entity_count());
    println!();

    // Walk entities and print info
    for &entity in &entities {
        print_entity(&world, entity, 0);
    }
}

fn print_entity(world: &World, entity: kiran::Entity, depth: usize) {
    let indent = "  ".repeat(depth);
    let name = world
        .get_component::<Name>(entity)
        .map(|n| n.0.as_str())
        .unwrap_or("(unnamed)");

    print!("{indent}[{entity}] {name}");

    if let Some(pos) = world.get_component::<Position>(entity) {
        print!(" @ ({:.1}, {:.1}, {:.1})", pos.0.x, pos.0.y, pos.0.z);
    }
    if let Some(light) = world.get_component::<LightComponent>(entity) {
        print!(" light={:.1}", light.intensity);
    }
    if let Some(tags) = world.get_component::<Tags>(entity)
        && !tags.0.is_empty()
    {
        print!(" tags=[{}]", tags.0.join(", "));
    }
    println!();

    // Recurse into children
    if let Some(children) = world.get_component::<Children>(entity) {
        for &child in &children.0 {
            print_entity(world, child, depth + 1);
        }
    }
}
