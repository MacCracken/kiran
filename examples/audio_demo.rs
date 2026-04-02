//! Example: Create an AudioEngine, attach SoundSource to entities, configure mix buses.

use kiran::World;
use kiran::audio::{AudioEngine, AudioListener, MixBus, MixBusVolumes, SoundSource};
use kiran::scene::Name;

fn main() {
    let mut world = World::new();

    // Create the audio engine at 48 kHz
    let mut engine = AudioEngine::new(48000);
    println!(
        "Audio engine: {}Hz, buffer={}",
        engine.sample_rate(),
        engine.buffer_size()
    );

    // Set up mix bus volumes
    let mut buses = MixBusVolumes::new();
    buses.set(MixBus::Music, 0.6);
    buses.set(MixBus::SFX, 0.9);
    buses.set(MixBus::Ambient, 0.4);
    println!(
        "Mix buses — Music: {:.1}, SFX: {:.1}, Ambient: {:.1}",
        buses.get(MixBus::Music),
        buses.get(MixBus::SFX),
        buses.get(MixBus::Ambient)
    );

    // Spawn a listener (camera)
    let camera = world.spawn();
    world.insert_component(camera, AudioListener).unwrap();
    world
        .insert_component(camera, Name("Camera".into()))
        .unwrap();
    engine.set_listener(camera);
    println!("Listener set on entity {camera}");

    // Spawn a music source (non-spatial, loops)
    let music = world.spawn();
    world.insert_component(music, Name("BGM".into())).unwrap();
    world
        .insert_component(
            music,
            SoundSource::new("music/theme.ogg")
                .non_spatial()
                .with_looping()
                .with_bus(MixBus::Music)
                .with_volume(0.8),
        )
        .unwrap();

    // Spawn a spatial SFX source
    let sword = world.spawn();
    world.insert_component(sword, Name("Sword".into())).unwrap();
    world
        .insert_component(
            sword,
            SoundSource::new("sfx/slash.wav")
                .with_volume(1.0)
                .with_max_distance(30.0)
                .with_bus(MixBus::SFX),
        )
        .unwrap();

    // Demonstrate spatial gain at various distances
    println!("\nSpatial gain falloff (max_distance=30):");
    for dist in [0.0, 10.0, 20.0, 30.0, 50.0] {
        let gain = AudioEngine::spatial_gain(dist, 30.0);
        let effective = gain * buses.effective(MixBus::SFX);
        println!("  distance {dist:>5.1}: raw gain={gain:.2}, effective={effective:.2}");
    }

    // Advance the audio clock a few buffers
    for _ in 0..10 {
        engine.advance();
    }
    println!(
        "\nAfter 10 buffer advances: playback position = {:.4}s",
        engine.position_secs()
    );
    println!("World has {} entities", world.entity_count());
}
