#![doc = include_str!("../README.md")]
//! > _Note_: This documentation is auto-generated
//! > from the project's README.md file.

// Engine module and components
pub mod engine;
pub use engine::{asset, color, tile};

// Game module (keep simulation_loop for backward compatibility)
pub mod game;
pub use game::simulation_loop;
