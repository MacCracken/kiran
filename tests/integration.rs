//! Integration tests exercising cross-module usage.

use kiran::World;
use kiran::input::{InputEvent, InputState, KeyCode, MouseButton};
use kiran::render::{Camera, DrawCommand, NullRenderer, RenderConfig, Renderer};
use kiran::scene::{LightComponent, Name, Position, Tags, load_scene, spawn_scene};
use kiran::world::{EventBus, GameClock};

#[test]
fn scene_load_spawn_query() {
    let toml_str = r#"
name = "Integration Test"
description = "Cross-module test scene"

[[entities]]
name = "Hero"
position = [5.0, 0.0, 3.0]
tags = ["player", "controllable"]

[[entities]]
name = "Torch"
position = [0.0, 2.0, 0.0]
light_intensity = 0.8

[[entities]]
name = "Boulder"
position = [10.0, 0.0, -5.0]
"#;

    let scene = load_scene(toml_str).unwrap();
    let mut world = World::new();
    let entities = spawn_scene(&mut world, &scene).unwrap();

    assert_eq!(world.entity_count(), 3);

    // Verify hero
    let hero = entities[0];
    assert_eq!(world.get_component::<Name>(hero).unwrap().0, "Hero");
    let pos = world.get_component::<Position>(hero).unwrap();
    assert_eq!(pos.0.x, 5.0);
    let tags = world.get_component::<Tags>(hero).unwrap();
    assert!(tags.0.contains(&"player".to_string()));

    // Verify torch has light
    let torch = entities[1];
    let light = world.get_component::<LightComponent>(torch).unwrap();
    assert!((light.intensity - 0.8).abs() < f32::EPSILON);

    // Verify boulder has no light or tags
    let boulder = entities[2];
    assert!(world.get_component::<LightComponent>(boulder).is_none());
    assert!(world.get_component::<Tags>(boulder).is_none());
}

#[test]
fn world_lifecycle_spawn_despawn() {
    let mut world = World::new();

    // Spawn entities
    let e1 = world.spawn();
    let e2 = world.spawn();
    let e3 = world.spawn();
    assert_eq!(world.entity_count(), 3);

    // Add components
    world.insert_component(e1, Name("A".into())).unwrap();
    world.insert_component(e2, Name("B".into())).unwrap();
    world.insert_component(e3, Name("C".into())).unwrap();

    // Despawn middle entity
    world.despawn(e2).unwrap();
    assert_eq!(world.entity_count(), 2);
    assert!(world.get_component::<Name>(e2).is_none());

    // Remaining entities still have components
    assert_eq!(world.get_component::<Name>(e1).unwrap().0, "A");
    assert_eq!(world.get_component::<Name>(e3).unwrap().0, "C");

    // Recycled entity gets new generation
    let e4 = world.spawn();
    assert_eq!(e4.index(), e2.index()); // reuses slot
    assert_eq!(e4.generation(), 1); // bumped generation
}

#[test]
fn clock_drives_fixed_updates() {
    let mut clock = GameClock::with_timestep(1.0 / 60.0);
    let mut fixed_count = 0;

    // Simulate 100ms at 16.67ms per frame (6 frames)
    for _ in 0..6 {
        clock.tick(1.0 / 60.0);
        while clock.consume_fixed() {
            fixed_count += 1;
        }
    }

    assert_eq!(fixed_count, 6);
    assert_eq!(clock.frame, 6);
    assert!((clock.elapsed - 6.0 / 60.0).abs() < 1e-10);
}

#[test]
fn input_state_multi_frame() {
    let mut state = InputState::new();

    // Frame 1: press W and Space
    state.process_event(&InputEvent::KeyPressed(KeyCode::W));
    state.process_event(&InputEvent::KeyPressed(KeyCode::Space));
    assert!(state.is_key_just_pressed(KeyCode::W));
    assert!(state.is_key_pressed(KeyCode::Space));

    // Frame 2: W still held, Space released
    state.clear_frame();
    state.process_event(&InputEvent::KeyReleased(KeyCode::Space));
    assert!(state.is_key_pressed(KeyCode::W));
    assert!(!state.is_key_just_pressed(KeyCode::W)); // not "just" anymore
    assert!(state.is_key_just_released(KeyCode::Space));

    // Frame 3: mouse input
    state.clear_frame();
    state.process_event(&InputEvent::MouseMoved { x: 640.0, y: 360.0 });
    state.process_event(&InputEvent::MouseButtonPressed(MouseButton::Left));
    assert_eq!(state.mouse_position(), (640.0, 360.0));
    assert!(state.is_mouse_button_pressed(MouseButton::Left));
}

#[test]
fn null_renderer_full_frame() {
    let mut renderer = NullRenderer::new();
    renderer.init(&RenderConfig::default()).unwrap();

    let camera = Camera::default();

    renderer.begin_frame().unwrap();
    renderer
        .submit(DrawCommand::Clear([0.1, 0.1, 0.2, 1.0]))
        .unwrap();
    renderer.submit(DrawCommand::SetCamera(camera)).unwrap();
    renderer.end_frame().unwrap();

    assert_eq!(renderer.frame_count, 1);
    assert_eq!(renderer.last_frame_command_count(), 2);

    renderer.shutdown().unwrap();
    assert!(!renderer.initialized);
}

#[test]
fn event_bus_cross_system() {
    #[derive(Debug, PartialEq)]
    struct DamageEvent {
        target: u64,
        amount: f32,
    }

    #[derive(Debug, PartialEq)]
    struct DeathEvent {
        entity: u64,
    }

    let mut world = World::new();
    world.insert_resource(EventBus::new());

    // System A publishes damage events
    {
        let bus = world.get_resource_mut::<EventBus>().unwrap();
        bus.publish(DamageEvent {
            target: 1,
            amount: 50.0,
        });
        bus.publish(DamageEvent {
            target: 2,
            amount: 100.0,
        });
    }

    // System B processes damage, publishes death
    {
        let bus = world.get_resource_mut::<EventBus>().unwrap();
        let damages = bus.drain::<DamageEvent>();
        assert_eq!(damages.len(), 2);

        for d in &damages {
            if d.amount >= 100.0 {
                bus.publish(DeathEvent { entity: d.target });
            }
        }
    }

    // System C processes deaths
    {
        let bus = world.get_resource_mut::<EventBus>().unwrap();
        let deaths = bus.drain::<DeathEvent>();
        assert_eq!(deaths.len(), 1);
        assert_eq!(deaths[0].entity, 2);
    }
}

#[test]
fn scene_and_world_resources() {
    let mut world = World::new();
    world.insert_resource(GameClock::with_timestep(1.0 / 120.0));

    let scene = load_scene(
        r#"
name = "Resource Test"
[[entities]]
name = "Player"
position = [0.0, 0.0, 0.0]
"#,
    )
    .unwrap();

    let entities = spawn_scene(&mut world, &scene).unwrap();
    assert_eq!(entities.len(), 1);

    // Clock resource is independent of scene entities
    let clock = world.get_resource::<GameClock>().unwrap();
    assert!((clock.fixed_timestep - 1.0 / 120.0).abs() < 1e-10);
}

#[test]
fn scene_toml_roundtrip() {
    use kiran::scene::{EntityDef, SceneDefinition};

    let original = SceneDefinition {
        name: "Roundtrip".into(),
        description: "Testing serialization".into(),
        prefabs: vec![],
        entities: vec![
            EntityDef {
                name: "A".into(),
                position: [1.0, 2.0, 3.0],
                light_intensity: Some(0.5),
                tags: vec!["tag1".into()],
                material: None,
                children: vec![],
                prefab: None,
                sound: None,
                physics: None,
            },
            EntityDef {
                name: "B".into(),
                position: [0.0, 0.0, 0.0],
                light_intensity: None,
                tags: vec![],
                material: None,
                children: vec![],
                prefab: None,
                sound: None,
                physics: None,
            },
        ],
    };

    let toml_str = toml::to_string(&original).unwrap();
    let restored = load_scene(&toml_str).unwrap();

    assert_eq!(restored.name, "Roundtrip");
    assert_eq!(restored.entities.len(), 2);
    assert_eq!(restored.entities[0].light_intensity, Some(0.5));
    assert_eq!(restored.entities[1].tags.len(), 0);
}

#[test]
fn input_serde_roundtrip() {
    let events = vec![
        InputEvent::KeyPressed(KeyCode::W),
        InputEvent::MouseMoved { x: 100.0, y: 200.0 },
        InputEvent::MouseScroll { dx: 0.0, dy: -3.0 },
    ];

    for event in &events {
        let json = serde_json::to_string(event).unwrap();
        let decoded: InputEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(*event, decoded);
    }
}

#[test]
fn stress_spawn_despawn_respawn() {
    let mut world = World::new();

    // Spawn 500 entities with components
    let mut entities = Vec::new();
    for i in 0..500 {
        let e = world.spawn();
        world
            .insert_component(e, Name(format!("entity_{i}")))
            .unwrap();
        world
            .insert_component(e, Position(hisab::Vec3::new(i as f32, 0.0, 0.0)))
            .unwrap();
        entities.push(e);
    }
    assert_eq!(world.entity_count(), 500);

    // Despawn every other entity
    for i in (0..500).step_by(2) {
        world.despawn(entities[i]).unwrap();
    }
    assert_eq!(world.entity_count(), 250);

    // Respawn into recycled slots and verify generations
    for _ in 0..250 {
        let e = world.spawn();
        assert_eq!(e.generation(), 1);
        world.insert_component(e, Name("recycled".into())).unwrap();
    }
    assert_eq!(world.entity_count(), 500);
}

#[test]
fn has_component_integration() {
    let mut world = World::new();
    let e = world.spawn();

    assert!(!world.has_component::<Name>(e));
    assert!(!world.has_component::<Position>(e));

    world.insert_component(e, Name("test".into())).unwrap();
    assert!(world.has_component::<Name>(e));
    assert!(!world.has_component::<Position>(e));

    world
        .insert_component(e, Position(hisab::Vec3::ZERO))
        .unwrap();
    assert!(world.has_component::<Name>(e));
    assert!(world.has_component::<Position>(e));

    world.remove_component::<Name>(e);
    assert!(!world.has_component::<Name>(e));
    assert!(world.has_component::<Position>(e));
}

#[test]
fn mouse_button_edge_triggers_multi_frame() {
    let mut state = InputState::new();

    // Frame 1: press left mouse
    state.process_event(&InputEvent::MouseButtonPressed(MouseButton::Left));
    assert!(state.is_mouse_button_just_pressed(MouseButton::Left));
    assert!(state.is_mouse_button_pressed(MouseButton::Left));

    // Frame 2: still held, not "just" anymore
    state.clear_frame();
    assert!(!state.is_mouse_button_just_pressed(MouseButton::Left));
    assert!(state.is_mouse_button_pressed(MouseButton::Left));

    // Frame 3: released
    state.process_event(&InputEvent::MouseButtonReleased(MouseButton::Left));
    assert!(state.is_mouse_button_just_released(MouseButton::Left));
    assert!(!state.is_mouse_button_pressed(MouseButton::Left));
}

#[test]
fn game_loop_simulation() {
    // Simulate a mini game loop: clock tick -> input -> events -> verify
    let mut world = World::new();
    world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
    world.insert_resource(EventBus::new());

    let scene = load_scene(
        r#"
name = "Game Loop Test"
[[entities]]
name = "Player"
position = [0.0, 0.0, 0.0]
"#,
    )
    .unwrap();
    let entities = spawn_scene(&mut world, &scene).unwrap();
    let player = entities[0];

    // Simulate 10 frames
    for frame in 0..10 {
        let clock = world.get_resource_mut::<GameClock>().unwrap();
        clock.tick(1.0 / 60.0);

        // Move player each frame
        let pos = world.get_component_mut::<Position>(player).unwrap();
        pos.0.x += 1.0;
        assert_eq!(pos.0.x, (frame + 1) as f32);
    }

    let clock = world.get_resource::<GameClock>().unwrap();
    assert_eq!(clock.frame, 10);
    let pos = world.get_component::<Position>(player).unwrap();
    assert_eq!(pos.0.x, 10.0);
}

#[test]
fn scheduler_scene_input_combined() {
    use kiran::world::{FnSystem, Scheduler, SystemStage};

    let mut world = World::new();

    // Load scene
    let scene = load_scene(
        r#"
name = "Scheduler Integration"
[[entities]]
name = "Player"
position = [0.0, 0.0, 0.0]
"#,
    )
    .unwrap();
    let entities = spawn_scene(&mut world, &scene).unwrap();
    let player = entities[0];

    world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
    world.insert_resource(kiran::input::InputState::new());

    // Build scheduler with real systems
    let mut scheduler = Scheduler::new();
    scheduler.add_system(Box::new(FnSystem::new(
        "clock_tick",
        SystemStage::Input,
        |world: &mut kiran::World| {
            let clock = world.get_resource_mut::<GameClock>().unwrap();
            clock.tick(1.0 / 60.0);
        },
    )));
    scheduler.add_system(Box::new(FnSystem::new(
        "input_clear",
        SystemStage::Input,
        |world: &mut kiran::World| {
            let input = world
                .get_resource_mut::<kiran::input::InputState>()
                .unwrap();
            input.clear_frame();
        },
    )));

    // Run 5 frames
    for _ in 0..5 {
        scheduler.run(&mut world);
    }

    let clock = world.get_resource::<GameClock>().unwrap();
    assert_eq!(clock.frame, 5);
    assert!(world.is_alive(player));
}

#[test]
fn reload_diff_integration() {
    use kiran::reload::apply_scene_diff;

    let mut world = World::new();

    // Initial scene
    let scene_v1 = load_scene(
        r#"
name = "Reload Test"
[[entities]]
name = "Player"
position = [0.0, 0.0, 0.0]
[[entities]]
name = "Enemy"
position = [10.0, 0.0, 0.0]
"#,
    )
    .unwrap();
    let entities = spawn_scene(&mut world, &scene_v1).unwrap();
    assert_eq!(world.entity_count(), 2);

    // Updated scene: player moved, enemy removed, new NPC added
    let scene_v2 = load_scene(
        r#"
name = "Reload Test"
[[entities]]
name = "Player"
position = [5.0, 0.0, 0.0]
[[entities]]
name = "NPC"
position = [20.0, 0.0, 0.0]
"#,
    )
    .unwrap();
    let result = apply_scene_diff(&mut world, &entities, &scene_v2).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(world.entity_count(), 2);

    // Player was updated in place
    let player_name = world.get_component::<Name>(result[0]).unwrap();
    assert_eq!(player_name.0, "Player");
    let player_pos = world.get_component::<Position>(result[0]).unwrap();
    assert_eq!(player_pos.0.x, 5.0);

    // NPC is new
    let npc_name = world.get_component::<Name>(result[1]).unwrap();
    assert_eq!(npc_name.0, "NPC");
}

#[test]
fn entity_from_id_integration() {
    let mut world = World::new();
    let e1 = world.spawn();
    world.despawn(e1).unwrap();
    let e2 = world.spawn(); // recycled, generation=1

    // Reconstruct from id
    let reconstructed = kiran::Entity::from_id(e2.id());
    assert_eq!(reconstructed, e2);
    assert!(world.is_alive(reconstructed));

    // Old entity id doesn't work
    let stale = kiran::Entity::from_id(e1.id());
    assert!(!world.is_alive(stale));
}

#[cfg(feature = "rendering")]
#[test]
fn soorat_renderer_full_data_flow() {
    use kiran::gpu::{SooratRenderer, batch_to_vertices};
    use kiran::render::{DrawCommand, RenderConfig, Renderer, SpriteDesc};

    let mut renderer = SooratRenderer::new();
    renderer.init(&RenderConfig::default()).unwrap();

    // Simulate a game frame: submit sprites via kiran's Renderer trait
    renderer.begin_frame().unwrap();
    renderer
        .submit(DrawCommand::Clear([0.1, 0.2, 0.3, 1.0]))
        .unwrap();

    for i in 0..10 {
        renderer
            .submit(DrawCommand::Sprite(SpriteDesc {
                texture_id: 1,
                x: i as f32 * 50.0,
                y: 100.0,
                width: 32.0,
                height: 32.0,
                rotation: 0.0,
                color: [1.0, 1.0, 1.0, 1.0],
            }))
            .unwrap();
    }

    renderer
        .submit(DrawCommand::SetCamera(Camera::default()))
        .unwrap();
    renderer.end_frame().unwrap();

    // Verify collected data is correct
    assert_eq!(renderer.sprite_count(), 10);
    assert!(renderer.camera().is_some());
    assert!((renderer.clear_color().r - 0.1).abs() < f32::EPSILON);

    // Verify the batch can be converted to GPU-ready vertex data
    let batch = renderer.sprite_batch();
    let (verts, indices) = batch_to_vertices(batch);
    assert_eq!(verts.len(), 40);
    assert_eq!(indices.len(), 60);

    // Verify vertex data is correct size (32 bytes per Vertex2D * 40 verts)
    assert_eq!(std::mem::size_of_val(&verts[0]) * verts.len(), 32 * 40);
}

#[cfg(feature = "rendering")]
#[test]
fn soorat_renderer_scene_to_sprites() {
    use kiran::gpu::{SooratRenderer, batch_to_vertices};
    use kiran::render::{DrawCommand, RenderConfig, Renderer, SpriteDesc};

    // Load a scene and convert entities to sprites
    let mut world = World::new();
    let scene = load_scene(
        r#"
name = "Render Test"
[[entities]]
name = "Player"
position = [100.0, 200.0, 0.0]
[[entities]]
name = "Enemy"
position = [300.0, 200.0, 0.0]
"#,
    )
    .unwrap();
    let entities = spawn_scene(&mut world, &scene).unwrap();

    // Build sprites from entity positions
    let mut renderer = SooratRenderer::new();
    renderer.init(&RenderConfig::default()).unwrap();
    renderer.begin_frame().unwrap();

    for &entity in &entities {
        if let Some(pos) = world.get_component::<Position>(entity) {
            renderer
                .submit(DrawCommand::Sprite(SpriteDesc {
                    texture_id: 0,
                    x: pos.0.x,
                    y: pos.0.y,
                    width: 32.0,
                    height: 32.0,
                    rotation: 0.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                }))
                .unwrap();
        }
    }

    renderer.end_frame().unwrap();
    assert_eq!(renderer.sprite_count(), 2);

    let (verts, _) = batch_to_vertices(renderer.sprite_batch());
    // First sprite at (100, 200), second at (300, 200)
    assert_eq!(verts[0].position[0], 100.0 + 16.0 - 16.0); // centered
    assert_eq!(verts[4].position[0], 300.0 + 16.0 - 16.0);
}

#[cfg(feature = "rendering")]
#[test]
fn soorat_re_exports_complete() {
    use kiran::gpu::{
        CameraUniforms, Color, FrameStats, LightUniforms, LineBatch, LineVertex,
        SooratWindowConfig, Sprite, SpriteBatch, UvRect, Vertex2D, Vertex3D, batch_to_vertices,
    };

    // Type size checks (no GPU required)
    assert_eq!(std::mem::size_of::<Vertex2D>(), 32);
    assert_eq!(std::mem::size_of::<Vertex3D>(), 48);
    assert_eq!(std::mem::size_of::<CameraUniforms>(), 128);
    assert_eq!(std::mem::size_of::<LightUniforms>(), 48);
    assert!(std::mem::size_of::<Color>() > 0);
    assert!(std::mem::size_of::<FrameStats>() > 0);
    assert!(std::mem::size_of::<LineVertex>() > 0);
    assert!(std::mem::size_of::<UvRect>() > 0);
    assert!(std::mem::size_of::<SooratWindowConfig>() > 0);

    // Sprite batch → vertex conversion
    let mut batch = SpriteBatch::new();
    batch.push(Sprite::new(0.0, 0.0, 10.0, 10.0).with_color(Color::RED));
    let (verts, indices) = batch_to_vertices(&batch);
    assert_eq!(verts.len(), 4);
    assert_eq!(indices.len(), 6);

    // LineBatch (CPU-side, no GPU)
    let mut lines = LineBatch::new();
    lines.line([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], Color::GREEN);
    assert_eq!(lines.line_count(), 1);
    lines.wire_box([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], Color::BLUE);
    assert_eq!(lines.line_count(), 13); // 1 + 12 box edges
}

#[cfg(feature = "rendering")]
#[test]
fn soorat_no_name_collision_with_kiran_material() {
    // kiran::scene::Material and kiran::gpu::SooratMaterial are different types
    use kiran::gpu::SooratMaterial;
    use kiran::scene::Material as SceneMaterial;

    // They're different types — this should compile
    let _scene_mat = SceneMaterial {
        color: [1.0, 0.0, 0.0, 1.0],
        texture: None,
    };
    // SooratMaterial requires GPU device — just check type exists
    assert!(std::mem::size_of::<SooratMaterial>() > 0);
}
