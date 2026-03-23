//! kiran-ai — AI integration: daimon client, hoosh inference
//!
//! Provides the AGNOS integration layer for the Kiran game engine,
//! registering as a daimon agent and routing LLM requests through hoosh.

pub mod daimon;

pub use daimon::{DaimonClient, DaimonConfig};
