use criterion::{Criterion, black_box, criterion_group, criterion_main};
use kiran::World;
use kiran::input::{InputEvent, InputState, KeyCode, MouseButton};
use kiran::reload::apply_scene_diff;
use kiran::render::{
    Camera, DrawCommand, FlyController, FollowController, NullRenderer, OrbitController,
    RenderConfig, Renderer,
};
use kiran::scene::{Name, Position, load_scene, spawn_scene};
use kiran::script::{Script, ScriptEngine, ScriptMessage};
use kiran::world::{EventBus, FnSystem, GameClock, Scheduler, SystemStage};

// ---------------------------------------------------------------------------
// ECS world operations
// ---------------------------------------------------------------------------

fn bench_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("world");

    group.bench_function("spawn_entity", |b| {
        let mut world = World::new();
        b.iter(|| {
            world.spawn();
        })
    });

    group.bench_function("spawn_despawn", |b| {
        let mut world = World::new();
        b.iter(|| {
            let e = world.spawn();
            world.despawn(e).unwrap();
        })
    });

    group.bench_function("insert_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        b.iter(|| {
            world
                .insert_component(entity, black_box(Name("bench".into())))
                .unwrap();
        })
    });

    group.bench_function("get_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        world
            .insert_component(entity, Name("bench".into()))
            .unwrap();
        b.iter(|| {
            black_box(world.get_component::<Name>(entity));
        })
    });

    group.bench_function("has_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        world
            .insert_component(entity, Name("bench".into()))
            .unwrap();
        b.iter(|| {
            black_box(world.has_component::<Name>(entity));
        })
    });

    group.bench_function("remove_component", |b| {
        let mut world = World::new();
        let entity = world.spawn();
        world
            .insert_component(entity, Name("bench".into()))
            .unwrap();
        b.iter(|| {
            world
                .insert_component(entity, black_box(Name("bench".into())))
                .unwrap();
            black_box(world.remove_component::<Name>(entity));
        })
    });

    group.bench_function("entity_count", |b| {
        let mut world = World::new();
        for _ in 0..1000 {
            world.spawn();
        }
        b.iter(|| {
            black_box(world.entity_count());
        })
    });

    group.bench_function("spawn_100_entities", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..100 {
                let e = world.spawn();
                world
                    .insert_component(e, black_box(Position(glam::Vec3::ZERO)))
                    .unwrap();
            }
        })
    });

    group.bench_function("despawn_with_components", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            for _ in 0..iters {
                let mut world = World::new();
                let mut entities = Vec::with_capacity(100);
                for i in 0..100 {
                    let e = world.spawn();
                    world.insert_component(e, Name(format!("E{i}"))).unwrap();
                    world
                        .insert_component(e, Position(glam::Vec3::ZERO))
                        .unwrap();
                    entities.push(e);
                }
                let start = std::time::Instant::now();
                for e in entities {
                    world.despawn(e).unwrap();
                }
                total += start.elapsed();
            }
            total
        })
    });

    group.bench_function("iterate_components", |b| {
        let mut world = World::new();
        let mut entities = Vec::with_capacity(1000);
        for i in 0..1000 {
            let e = world.spawn();
            world
                .insert_component(e, Position(glam::Vec3::new(i as f32, 0.0, 0.0)))
                .unwrap();
            entities.push(e);
        }
        b.iter(|| {
            let mut sum = 0.0f32;
            for &e in &entities {
                if let Some(pos) = world.get_component::<Position>(e) {
                    sum += pos.0.x;
                }
            }
            black_box(sum);
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Scene loading
// ---------------------------------------------------------------------------

fn bench_scene(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene");

    let toml_10 = generate_scene_toml(10);
    let toml_100 = generate_scene_toml(100);

    group.bench_function("load_10_entities", |b| {
        b.iter(|| load_scene(black_box(&toml_10)).unwrap())
    });

    group.bench_function("load_100_entities", |b| {
        b.iter(|| load_scene(black_box(&toml_100)).unwrap())
    });

    group.bench_function("spawn_10_entities", |b| {
        let scene = load_scene(&toml_10).unwrap();
        b.iter(|| {
            let mut world = World::new();
            spawn_scene(&mut world, black_box(&scene)).unwrap();
        })
    });

    group.bench_function("spawn_100_entities", |b| {
        let scene = load_scene(&toml_100).unwrap();
        b.iter(|| {
            let mut world = World::new();
            spawn_scene(&mut world, black_box(&scene)).unwrap();
        })
    });

    group.bench_function("scene_serialize", |b| {
        let scene = load_scene(&toml_100).unwrap();
        b.iter(|| {
            black_box(toml::to_string(black_box(&scene)).unwrap());
        })
    });

    group.finish();
}

fn generate_scene_toml(count: usize) -> String {
    let mut s = String::from("name = \"Bench Scene\"\n");
    for i in 0..count {
        s.push_str(&format!(
            "\n[[entities]]\nname = \"Entity{i}\"\nposition = [{}.0, {}.0, 0.0]\n",
            i % 10,
            i / 10,
        ));
    }
    s
}

// ---------------------------------------------------------------------------
// Input processing
// ---------------------------------------------------------------------------

fn bench_input(c: &mut Criterion) {
    let mut group = c.benchmark_group("input");

    group.bench_function("process_key_event", |b| {
        let mut state = InputState::new();
        b.iter(|| {
            state.process_event(black_box(&InputEvent::KeyPressed(KeyCode::W)));
            state.process_event(black_box(&InputEvent::KeyReleased(KeyCode::W)));
        })
    });

    group.bench_function("query_key_state", |b| {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::W));
        b.iter(|| {
            black_box(state.is_key_pressed(KeyCode::W));
            black_box(state.is_key_just_pressed(KeyCode::W));
        })
    });

    group.bench_function("clear_frame", |b| {
        let mut state = InputState::new();
        state.process_event(&InputEvent::KeyPressed(KeyCode::W));
        state.process_event(&InputEvent::KeyPressed(KeyCode::Space));
        b.iter(|| {
            state.clear_frame();
        })
    });

    group.bench_function("process_10_keys", |b| {
        let keys = [
            KeyCode::W,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::Space,
            KeyCode::LShift,
            KeyCode::E,
            KeyCode::Q,
            KeyCode::F,
            KeyCode::R,
        ];
        let mut state = InputState::new();
        b.iter(|| {
            for &key in &keys {
                state.process_event(black_box(&InputEvent::KeyPressed(key)));
            }
            state.clear_frame();
            for &key in &keys {
                state.process_event(black_box(&InputEvent::KeyReleased(key)));
            }
            state.clear_frame();
        })
    });

    group.bench_function("mouse_button_edge", |b| {
        let mut state = InputState::new();
        b.iter(|| {
            state.process_event(black_box(&InputEvent::MouseButtonPressed(
                MouseButton::Left,
            )));
            black_box(state.is_mouse_button_just_pressed(MouseButton::Left));
            state.clear_frame();
            state.process_event(black_box(&InputEvent::MouseButtonReleased(
                MouseButton::Left,
            )));
            black_box(state.is_mouse_button_just_released(MouseButton::Left));
            state.clear_frame();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");

    group.bench_function("camera_view_projection", |b| {
        let cam = Camera::default();
        b.iter(|| black_box(&cam).view_projection())
    });

    group.bench_function("null_renderer_frame", |b| {
        let mut renderer = NullRenderer::new();
        renderer.init(&RenderConfig::default()).unwrap();
        b.iter(|| {
            renderer.begin_frame().unwrap();
            renderer
                .submit(DrawCommand::Clear(black_box([0.0, 0.0, 0.0, 1.0])))
                .unwrap();
            renderer.end_frame().unwrap();
        })
    });

    group.bench_function("orbit_controller_apply", |b| {
        let orbit = OrbitController::default();
        let mut cam = Camera::default();
        b.iter(|| {
            orbit.apply(black_box(&mut cam));
        })
    });

    group.bench_function("fly_controller_move", |b| {
        let fly = FlyController::default();
        let mut cam = Camera::default();
        b.iter(|| {
            fly.fly(
                black_box(&mut cam),
                black_box(1.0),
                black_box(0.0),
                black_box(0.0),
                black_box(0.016),
            );
        })
    });

    group.bench_function("follow_controller", |b| {
        let follow = FollowController::default();
        let mut cam = Camera::default();
        b.iter(|| {
            follow.follow(
                black_box(&mut cam),
                black_box(glam::Vec3::new(10.0, 0.0, 5.0)),
                black_box(0.016),
            );
        })
    });

    group.bench_function("null_renderer_10_commands", |b| {
        let mut renderer = NullRenderer::new();
        renderer.init(&RenderConfig::default()).unwrap();
        b.iter(|| {
            renderer.begin_frame().unwrap();
            for _ in 0..10 {
                renderer
                    .submit(DrawCommand::Clear(black_box([0.0, 0.0, 0.0, 1.0])))
                    .unwrap();
            }
            renderer.end_frame().unwrap();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

fn bench_clock(c: &mut Criterion) {
    let mut group = c.benchmark_group("clock");

    group.bench_function("tick", |b| {
        let mut clock = GameClock::with_timestep(1.0 / 60.0);
        b.iter(|| clock.tick(black_box(0.016)))
    });

    group.bench_function("consume_fixed", |b| {
        let mut clock = GameClock::with_timestep(1.0 / 60.0);
        clock.tick(0.033);
        b.iter(|| {
            clock.tick(black_box(0.016));
            while clock.consume_fixed() {}
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

fn bench_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("events");

    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestEvent(u64);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct OtherEvent(f64);

    group.bench_function("publish_100", |b| {
        let mut bus = EventBus::new();
        b.iter(|| {
            for i in 0..100 {
                bus.publish(TestEvent(black_box(i)));
            }
            bus.clear();
        })
    });

    group.bench_function("publish_drain_100", |b| {
        let mut bus = EventBus::new();
        b.iter(|| {
            for i in 0..100 {
                bus.publish(TestEvent(black_box(i)));
            }
            let _ = bus.drain::<TestEvent>();
        })
    });

    group.bench_function("multi_type_publish_drain", |b| {
        let mut bus = EventBus::new();
        b.iter(|| {
            for i in 0..50 {
                bus.publish(TestEvent(black_box(i)));
                bus.publish(OtherEvent(black_box(i as f64)));
            }
            let _ = bus.drain::<TestEvent>();
            let _ = bus.drain::<OtherEvent>();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Full game loop simulation
// ---------------------------------------------------------------------------

fn bench_game_loop(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_loop");

    group.bench_function("tick_10_entities", |b| {
        let toml = generate_scene_toml(10);
        let scene = load_scene(&toml).unwrap();
        let mut world = World::new();
        spawn_scene(&mut world, &scene).unwrap();
        world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
        world.insert_resource(EventBus::new());
        world.insert_resource(InputState::new());

        b.iter(|| {
            let clock = world.get_resource_mut::<GameClock>().unwrap();
            clock.tick(black_box(1.0 / 60.0));

            let input = world.get_resource_mut::<InputState>().unwrap();
            input.clear_frame();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

fn bench_scheduler(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");

    group.bench_function("run_4_systems", |b| {
        let mut scheduler = Scheduler::new();
        scheduler.add_system(Box::new(FnSystem::new("input", SystemStage::Input, |_| {})));
        scheduler.add_system(Box::new(FnSystem::new(
            "physics",
            SystemStage::Physics,
            |_| {},
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "logic",
            SystemStage::GameLogic,
            |_| {},
        )));
        scheduler.add_system(Box::new(FnSystem::new(
            "render",
            SystemStage::Render,
            |_| {},
        )));

        let mut world = World::new();
        // Pre-sort
        scheduler.run(&mut world);

        b.iter(|| {
            scheduler.run(black_box(&mut world));
        })
    });

    group.bench_function("run_10_systems", |b| {
        let mut scheduler = Scheduler::new();
        for i in 0..10 {
            let stage = match i % 4 {
                0 => SystemStage::Input,
                1 => SystemStage::Physics,
                2 => SystemStage::GameLogic,
                _ => SystemStage::Render,
            };
            scheduler.add_system(Box::new(FnSystem::new(format!("sys_{i}"), stage, |_| {})));
        }

        let mut world = World::new();
        scheduler.run(&mut world);

        b.iter(|| {
            scheduler.run(black_box(&mut world));
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Hierarchy spawning
// ---------------------------------------------------------------------------

fn bench_hierarchy(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchy");

    group.bench_function("spawn_10_with_children", |b| {
        let toml_str = r#"
name = "Hierarchy Bench"
[[entities]]
name = "P0"
[[entities.children]]
name = "C0"
[[entities.children]]
name = "C1"
[[entities]]
name = "P1"
[[entities.children]]
name = "C2"
[[entities.children]]
name = "C3"
[[entities]]
name = "P2"
[[entities.children]]
name = "C4"
[[entities.children]]
name = "C5"
[[entities]]
name = "P3"
[[entities.children]]
name = "C6"
[[entities.children]]
name = "C7"
[[entities]]
name = "P4"
[[entities.children]]
name = "C8"
[[entities.children]]
name = "C9"
"#;
        let scene = load_scene(toml_str).unwrap();
        b.iter(|| {
            let mut world = World::new();
            spawn_scene(&mut world, black_box(&scene)).unwrap();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

fn bench_resources(c: &mut Criterion) {
    let mut group = c.benchmark_group("resources");

    group.bench_function("get_resource", |b| {
        let mut world = World::new();
        world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
        b.iter(|| {
            black_box(world.get_resource::<GameClock>());
        })
    });

    group.bench_function("get_resource_mut", |b| {
        let mut world = World::new();
        world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
        b.iter(|| {
            black_box(world.get_resource_mut::<GameClock>());
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Scene diff
// ---------------------------------------------------------------------------

fn bench_reload(c: &mut Criterion) {
    let mut group = c.benchmark_group("reload");

    group.bench_function("diff_update_10", |b| {
        let toml_v1 = generate_scene_toml(10);
        let scene_v1 = load_scene(&toml_v1).unwrap();

        // V2: same entities, different positions
        let mut toml_v2 = String::from("name = \"Bench Scene\"\n");
        for i in 0..10 {
            toml_v2.push_str(&format!(
                "\n[[entities]]\nname = \"Entity{i}\"\nposition = [{}.0, {}.0, 99.0]\n",
                i % 10 + 1,
                i / 10 + 1,
            ));
        }
        let scene_v2 = load_scene(&toml_v2).unwrap();

        b.iter(|| {
            let mut world = World::new();
            let entities = spawn_scene(&mut world, &scene_v1).unwrap();
            apply_scene_diff(&mut world, &entities, black_box(&scene_v2)).unwrap();
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Script messaging
// ---------------------------------------------------------------------------

fn bench_script(c: &mut Criterion) {
    let mut group = c.benchmark_group("script");

    group.bench_function("send_100_messages", |b| {
        let mut engine = ScriptEngine::default();
        b.iter(|| {
            for i in 0..100 {
                engine.send(ScriptMessage::new(
                    black_box("update"),
                    black_box(format!("{i}")),
                ));
            }
            engine.drain_inbox();
        })
    });

    group.bench_function("run_10_scripted_entities", |b| {
        b.iter_custom(|iters| {
            let mut total = std::time::Duration::ZERO;
            for _ in 0..iters {
                let mut world = World::new();
                let mut engine = ScriptEngine::default();

                let mut entities = Vec::new();
                for i in 0..10 {
                    let e = world.spawn();
                    world
                        .insert_component(e, Script::new(format!("s{i}.wasm")))
                        .unwrap();
                    engine.send(ScriptMessage::new("tick", "{}").to_entity(e));
                    entities.push(e);
                }

                world.insert_resource(engine);
                let start = std::time::Instant::now();
                kiran::script::run_scripts(&mut world);
                total += start.elapsed();
            }
            total
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_world,
    bench_scene,
    bench_input,
    bench_render,
    bench_clock,
    bench_events,
    bench_game_loop,
    bench_scheduler,
    bench_hierarchy,
    bench_resources,
    bench_reload,
    bench_script,
);
criterion_main!(benches);
