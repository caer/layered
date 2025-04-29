use image::imageops::FilterType;
use macroquad::{miniquad::conf::Icon, window::Conf};

/// Application window icon.
const WINDOW_ICON: &[u8] = include_bytes!("../assets/cosy.png");

/// Application entrypoint.
fn main() {
    // Create small (16x16), medium (32x32),
    // and large (64x64) window icons.
    let window_icon = image::load_from_memory(WINDOW_ICON).unwrap();
    let small = window_icon.resize_exact(16, 16, FilterType::Nearest);
    let medium = window_icon.resize_exact(32, 32, FilterType::Nearest);
    let large = window_icon.resize_exact(64, 64, FilterType::Nearest);

    // Launch a new Macroquad window with
    // a custom configuration and our
    // simulation loop.
    //
    // This line is equivalent to what the
    // macroquad::main macro does, but I
    // don't like macros on the main fn. <3
    macroquad::Window::from_config(
        Conf {
            window_title: "Layered".to_owned(),
            high_dpi: true,
            sample_count: 2,
            icon: Some(Icon {
                small: small.as_bytes().try_into().unwrap(),
                medium: medium.as_bytes().try_into().unwrap(),
                big: large.as_bytes().try_into().unwrap(),
            }),
            fullscreen: false,
            ..Default::default()
        },
        layered::simulation_loop(),
    );
}
