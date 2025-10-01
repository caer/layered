//! Color palette.
use palette::Srgb;

/// Type used for in-memory colors across the crate.
pub type Color = palette::rgb::Rgba<Srgb, u8>;

/// "Default" color, typically only
/// meaningful for blend masks.
pub const DEFAULT: Color = Color::new(255, 255, 255, 255);

/// Primary accent.
pub const ACCENT_1: Color = Color::new(228, 140, 53, 255);

/// Secondary accent.
pub const ACCENT_2: Color = Color::new(81, 156, 160, 255);

/// Tertiary accent.
pub const ACCENT_3: Color = Color::new(204, 116, 167, 255);

/// Background.
pub const BACKGROUND: Color = Color::new(38, 38, 34, 255);

/// Converts a [`palette`] color to
/// a [`macroquad::color::Color`].
pub const fn as_macroquad_color(color: Color) -> macroquad::color::Color {
    macroquad::color::Color::new(
        color.color.red as f32 / 255.0,
        color.color.green as f32 / 255.0,
        color.color.blue as f32 / 255.0,
        color.alpha as f32 / 255.0,
    )
}
