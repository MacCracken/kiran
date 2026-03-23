//! Kiran — AI-native game engine for AGNOS
//!
//! Modular game engine built in Rust. Composes AGNOS shared crates for
//! physics (impetus), math (hisab), audio (dhvani), and rendering (aethersafta).

pub mod input;
pub mod render;
pub mod scene;
pub mod world;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "physics")]
pub mod physics;

// Re-export key types at crate root
pub use world::{
    Entity, EventBus, FnSystem, GameClock, KiranError, Result, Scheduler, System, SystemStage,
    World,
};
