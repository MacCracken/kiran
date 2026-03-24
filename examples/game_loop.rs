//! Example: Minimal game loop with scheduler, input, and profiling.

use kiran::input::{InputEvent, InputState, KeyCode};
use kiran::profiler::FrameProfiler;
use kiran::scene::{Name, Position};
use kiran::world::{EventBus, FnSystem, SystemStage};
use kiran::{GameClock, Scheduler, World};

fn main() {
    // Set up world
    let mut world = World::new();
    world.insert_resource(GameClock::with_timestep(1.0 / 60.0));
    world.insert_resource(EventBus::new());
    world.insert_resource(InputState::new());
    world.insert_resource(FrameProfiler::default());

    // Spawn a player
    let player = world.spawn();
    world
        .insert_component(player, Name("Player".into()))
        .unwrap();
    world
        .insert_component(player, Position(hisab::Vec3::ZERO))
        .unwrap();

    // Build scheduler
    let mut scheduler = Scheduler::new();

    scheduler.add_system(Box::new(FnSystem::new(
        "clock_tick",
        SystemStage::Input,
        |world: &mut World| {
            let clock = world.get_resource_mut::<GameClock>().unwrap();
            clock.tick(1.0 / 60.0);
        },
    )));

    scheduler.add_system(Box::new(FnSystem::new(
        "input_clear",
        SystemStage::Input,
        |world: &mut World| {
            let input = world.get_resource_mut::<InputState>().unwrap();
            input.clear_frame();
        },
    )));

    scheduler.add_system(Box::new(FnSystem::new(
        "move_player",
        SystemStage::GameLogic,
        move |world: &mut World| {
            let speed = 5.0 / 60.0; // 5 units per second
            let input = world.get_resource::<InputState>().unwrap();
            let dx = if input.is_key_pressed(KeyCode::D) {
                speed
            } else if input.is_key_pressed(KeyCode::A) {
                -speed
            } else {
                0.0
            };

            if let Some(pos) = world.get_component_mut::<Position>(player) {
                pos.0.x += dx;
            }
        },
    )));

    // Simulate 10 frames
    println!("Simulating 10 frames...");
    for frame in 0..10 {
        // Simulate pressing D on frames 3-7
        if (3..=7).contains(&frame) {
            let input = world.get_resource_mut::<InputState>().unwrap();
            input.process_event(&InputEvent::KeyPressed(KeyCode::D));
        }

        // Profile the frame
        let profiler = world.get_resource_mut::<FrameProfiler>().unwrap();
        profiler.begin_frame();

        scheduler.run(&mut world);

        let profiler = world.get_resource_mut::<FrameProfiler>().unwrap();
        profiler.end_frame();
    }

    // Print results
    let pos = world.get_component::<Position>(player).unwrap();
    println!(
        "Player position: ({:.2}, {:.2}, {:.2})",
        pos.0.x, pos.0.y, pos.0.z
    );

    let profiler = world.get_resource::<FrameProfiler>().unwrap();
    println!(
        "Profiler: {} frames, avg {:.3}ms, {} slow",
        profiler.total_frames,
        profiler.average_frame_time().as_secs_f64() * 1000.0,
        profiler.slow_frame_count
    );

    let clock = world.get_resource::<GameClock>().unwrap();
    println!(
        "Clock: frame {}, elapsed {:.3}s",
        clock.frame, clock.elapsed
    );
}
