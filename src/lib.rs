//! Kiran — AI-native game engine for AGNOS
//!
//! Modular game engine built in Rust. Composes 46 optional AGNOS crates across
//! 16 feature gates: rendering, audio, voice, physics, fluids, dynamics, ai,
//! behavior, scripting, multiplayer, navigation, biology, chemistry, astronomy,
//! world, and `full` (all of the above). Core math provided by hisab.

pub mod input;
pub mod render;
pub mod scene;
pub mod world;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "behavior")]
pub mod personality;

#[cfg(feature = "physics")]
pub mod physics;

#[cfg(feature = "rendering")]
pub mod gpu;

#[cfg(feature = "multiplayer")]
pub mod net;

#[cfg(feature = "fluids")]
pub mod fluids;

#[cfg(feature = "audio")]
pub mod acoustics;

#[cfg(feature = "voice")]
pub mod voice;

#[cfg(feature = "media")]
pub mod media;

#[cfg(feature = "dynamics")]
pub mod dynamics;

#[cfg(feature = "biology")]
pub mod biology;

#[cfg(feature = "chemistry")]
pub mod chemistry;

#[cfg(feature = "astronomy")]
pub mod astronomy;

#[cfg(feature = "world")]
pub mod lore;

pub mod animation;
pub mod archetype;
pub mod asset;
pub mod gizmos;
pub mod job;
pub mod pool;
pub mod profiler;
pub mod reload;
pub mod script;
pub mod state;

#[cfg(feature = "navigation")]
pub mod nav;

// Re-export key types at crate root
pub use world::{
    Bundle, ChangeTracker, Commands, Entity, EventBus, FnSystem, GameClock, KiranError, Result,
    Scheduler, System, SystemStage, World,
};
