//! Kiran — AI-native game engine for AGNOS
//!
//! Modular game engine built in Rust. Composes AGNOS shared crates for
//! physics (impetus), math (hisab), audio (dhvani), and rendering (soorat).

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

#[cfg(feature = "rendering")]
pub mod gpu;

#[cfg(feature = "multiplayer")]
pub mod net;

pub mod asset;
pub mod gizmos;
pub mod profiler;
pub mod reload;
pub mod script;
pub mod state;

// Re-export key types at crate root
pub use world::{
    Bundle, ChangeTracker, Commands, Entity, EventBus, FnSystem, GameClock, KiranError, Result,
    Scheduler, System, SystemStage, World,
};
