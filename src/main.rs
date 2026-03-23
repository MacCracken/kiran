//! Kiran CLI — run or check scenes from the command line.

use anyhow::Result;
use clap::{Parser, Subcommand};
use kiran::World;
use kiran::scene::{LightComponent, Name, Position, Tags, load_scene, spawn_scene};

#[derive(Parser)]
#[command(name = "kiran", version, about = "Kiran — AI-native game engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load and run a scene (currently loads, spawns, and prints a summary).
    Run {
        /// Path to a TOML scene file.
        scene: String,
    },
    /// Validate a scene file without running it.
    Check {
        /// Path to a TOML scene file.
        scene: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { scene: path } => run_scene(&path),
        Commands::Check { scene: path } => check_scene(&path),
    }
}

fn run_scene(path: &str) -> Result<()> {
    let toml_str = std::fs::read_to_string(path)?;
    let scene = load_scene(&toml_str)?;

    println!("Scene: {}", scene.name);
    if !scene.description.is_empty() {
        println!("  {}", scene.description);
    }

    let mut world = World::new();
    let entities = spawn_scene(&mut world, &scene)?;

    println!("Spawned {} entities:", entities.len());
    for entity in &entities {
        let name = world
            .get_component::<Name>(*entity)
            .map(|n| n.0.as_str())
            .unwrap_or("(unnamed)");
        let pos = world.get_component::<Position>(*entity);
        let light = world.get_component::<LightComponent>(*entity);
        let tags = world.get_component::<Tags>(*entity);

        print!("  [{entity}] {name}");
        if let Some(p) = pos {
            print!(" @ ({}, {}, {})", p.0.x, p.0.y, p.0.z);
        }
        if let Some(l) = light {
            print!(" light={}", l.intensity);
        }
        if let Some(t) = tags
            && !t.0.is_empty()
        {
            print!(" tags=[{}]", t.0.join(", "));
        }
        println!();
    }

    println!("World: {} live entities", world.entity_count());
    Ok(())
}

fn check_scene(path: &str) -> Result<()> {
    let toml_str = std::fs::read_to_string(path)?;
    let scene = load_scene(&toml_str)?;

    println!("Scene '{}' is valid.", scene.name);
    println!("  Entities: {}", scene.entities.len());
    let lights = scene
        .entities
        .iter()
        .filter(|e| e.light_intensity.is_some())
        .count();
    println!("  Lights:   {lights}");
    let tagged = scene.entities.iter().filter(|e| !e.tags.is_empty()).count();
    println!("  Tagged:   {tagged}");
    Ok(())
}
