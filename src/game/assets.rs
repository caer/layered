//! Game asset constants.

// Tiles for drawing.
pub const FLOOR_TILE: &[u8] = include_bytes!("../../assets/cosi-tile-light.png");
pub const WALL_TILE: &[u8] = include_bytes!("../../assets/cosi-tile-dark.png");
pub const BACKGROUND_TILE: &[u8] = include_bytes!("../../assets/cosi-tile-empty.png");
pub const SPRITE: &[u8] = include_bytes!("../../assets/cosy.png");
pub const SPRITE_BACK: &[u8] = include_bytes!("../../assets/cosy-back.png");

// Tilemaps.
pub const TILEMAPS: &[&[u8]] = &[
    include_bytes!("../../assets/layer-1.png"),
    include_bytes!("../../assets/layer-2.png"),
    include_bytes!("../../assets/layer-3.png"),
    include_bytes!("../../assets/layer-4.png"),
    include_bytes!("../../assets/layer-5.png"),
];

// Miscellaneous assets.
pub const SPLASH: &[u8] = include_bytes!("../../assets/splash.png");
pub const AMBIENCE: &[u8] = include_bytes!("../../assets/MooMarMouse-itchio-ambience.wav");
pub const HAPPY_SOUND: &[u8] = include_bytes!("../../assets/JDWasabi-itchio-confirm.wav");
pub const SAD_SOUND: &[u8] = include_bytes!("../../assets/JDWasabi-itchio-bubble.wav");
