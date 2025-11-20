//! Game phase management.

/// System phase enum for managing game state transitions.
pub enum SystemPhase {
    /// Splash screen, with duration in seconds.
    Splash(f64),

    /// Transition to the layer with `index`.
    LayerTransition(f64, f64, usize, bool),

    /// Active layer.
    Layer,
}
