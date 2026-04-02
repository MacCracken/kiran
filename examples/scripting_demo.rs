//! Example: Create a ScriptEngine, attach scripts to entities, pass messages.

use kiran::World;
use kiran::script::{Script, ScriptConfig, ScriptEngine, ScriptMessage, run_scripts};

fn main() {
    let mut world = World::new();

    // Create a script engine with custom config
    let config = ScriptConfig {
        timeout_ms: 32,
        max_memory: 8 * 1024 * 1024,
    };
    println!(
        "ScriptEngine — timeout: {}ms, max_memory: {}MB",
        config.timeout_ms,
        config.max_memory / (1024 * 1024)
    );

    let mut engine = ScriptEngine::new(config);
    println!("WASM backend available: {}", engine.wasm_available());

    // Spawn an NPC with a script
    let npc = world.spawn();
    world
        .insert_component(
            npc,
            Script::new("scripts/npc_ai.lua").with_state(r#"{"hp": 100, "state": "idle"}"#),
        )
        .unwrap();

    // Spawn a door with a script
    let door = world.spawn();
    world
        .insert_component(
            door,
            Script::new("scripts/door.lua").with_state(r#"{"open": false}"#),
        )
        .unwrap();

    // Send targeted messages
    engine.send(
        ScriptMessage::new("damage", r#"{"amount": 25}"#)
            .from_entity(door)
            .to_entity(npc),
    );
    engine.send(
        ScriptMessage::new("interact", r#"{"action": "open"}"#)
            .from_entity(npc)
            .to_entity(door),
    );

    // Also push an outbox message (simulating script output)
    engine.push_outbox(ScriptMessage::new(
        "spawn_particle",
        r#"{"effect": "sparks"}"#,
    ));

    world.insert_resource(engine);

    // Run the script system — delivers inbox messages to script state
    run_scripts(&mut world);

    // Check results
    let npc_script = world.get_component::<Script>(npc).unwrap();
    println!("\nNPC script state after message: {}", npc_script.state);

    let door_script = world.get_component::<Script>(door).unwrap();
    println!("Door script state after message: {}", door_script.state);

    let engine = world.get_resource::<ScriptEngine>().unwrap();
    println!("Scripts executed this frame: {}", engine.exec_count());

    let outbox = world
        .get_resource_mut::<ScriptEngine>()
        .unwrap()
        .drain_outbox();
    println!("Outbox messages: {}", outbox.len());
    for msg in &outbox {
        println!("  -> kind={}, payload={}", msg.kind, msg.payload);
    }
}
