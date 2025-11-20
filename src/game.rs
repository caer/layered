//! Game-specific code for the Layered game.

pub mod assets;
pub mod config;
pub mod level_loader;
pub mod phase;
pub mod state;

// Re-export main simulation loop
pub use state::simulation_loop;
