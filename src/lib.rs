#![doc = include_str!("../README.md")]
//! > _Note_: This documentation is auto-generated
//! > from the project's README.md file.

use color::Color;
use glam::Vec2;
use image::{imageops::FilterType, DynamicImage};
use macroquad::{
    audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound},
    color::{GRAY, WHITE},
    input::KeyCode,
    texture::{DrawTextureParams, FilterMode, Texture2D},
};
use tile::{Tile, TileMap, TileTexture};

pub mod asset;
pub mod color;
pub mod tile;

// Map size in grid units.
const WIDTH: usize = 48;
const HEIGHT: usize = 48;

// Map draw layers.
const FOREGROUND_LAYER: i8 = 0;
const BACKGROUND_LAYER: i8 = -1;

// Tiles for drawing.
const FLOOR_TILE: &[u8] = include_bytes!("../assets/cosi-tile-light.png");
const WALL_TILE: &[u8] = include_bytes!("../assets/cosi-tile-dark.png");
const BACKGROUND_TILE: &[u8] = include_bytes!("../assets/cosi-tile-empty.png");
const SPRITE: &[u8] = include_bytes!("../assets/cosy.png");
const SPRITE_BACK: &[u8] = include_bytes!("../assets/cosy-back.png");

// Tilemaps.
const TILEMAPS: &[&[u8]] = &[
    include_bytes!("../assets/layer-1.png"),
    include_bytes!("../assets/layer-2.png"),
    include_bytes!("../assets/layer-3.png"),
    include_bytes!("../assets/layer-4.png"),
    include_bytes!("../assets/layer-5.png"),
];

// Miscellaneous assets.
const SPLASH: &[u8] = include_bytes!("../assets/splash.png");
const AMBIENCE: &[u8] = include_bytes!("../assets/MooMarMouse-itchio-ambience.wav");
const HAPPY_SOUND: &[u8] = include_bytes!("../assets/JDWasabi-itchio-confirm.wav");
const SAD_SOUND: &[u8] = include_bytes!("../assets/JDWasabi-itchio-bubble.wav");

/// Entrypoint for the infinite simulation loop.
pub async fn simulation_loop() {
    macroquad::prelude::clear_background(color::as_macroquad_color(color::BACKGROUND));
    macroquad::prelude::next_frame().await;

    // Play some nice music.
    let sound = load_sound_from_bytes(AMBIENCE).await.unwrap();
    play_sound(
        &sound,
        PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    // Initialize state.
    let mut state = LayerState::new().await;

    // Track system phase.
    const PHASE_TRANSITION: f64 = 0.75;
    let mut phase = SystemPhase::Splash(3.0);
    let mut phase_start = macroquad::prelude::get_time();
    let mut layer_reset = false;
    let mut transition_tint = None;
    let mut transition_color = color::BACKGROUND;

    // Frame loop.
    loop {
        // Track phase timing.
        let time = macroquad::prelude::get_time();
        let elapsed = time - phase_start;

        // Track phase transition.
        match phase {
            SystemPhase::Splash(duration) => {
                let remaining = duration - elapsed;
                draw_splash_screen(&state.splash_texture);

                if remaining <= 0.0 {
                    phase =
                        SystemPhase::LayerTransition(PHASE_TRANSITION, PHASE_TRANSITION, 0, false);
                    phase_start = time;
                }
            }
            SystemPhase::LayerTransition(out_duration, in_duration, layer, draw_current_layer) => {
                let remaining = out_duration + in_duration - elapsed;

                // Complete transition.
                if remaining <= 0.0 {
                    phase = SystemPhase::Layer;
                    phase_start = time;
                    draw_layer(&mut state);

                // Activate new layer and fade in.
                } else if remaining <= in_duration {
                    if !layer_reset {
                        state.activate_layer(layer);
                        layer_reset = true;
                    }

                    draw_layer(&mut state);
                    draw_overlay(transition_color, (remaining / in_duration) as f32);

                    if let Some(tint) = transition_tint {
                        draw_overlay(tint, ((remaining / in_duration) as f32) * 0.25);
                    }

                // Fade out.
                } else {
                    if draw_current_layer {
                        draw_layer(&mut state);
                    } else {
                        draw_splash_screen(&state.splash_texture);
                    }

                    (transition_tint, transition_color) = if state.threatened {
                        (None, color::ACCENT_2)
                    } else if state.remaining_objectives && draw_current_layer {
                        (Some(color::ACCENT_3), color::BACKGROUND)
                    } else {
                        (None, color::BACKGROUND)
                    };

                    draw_overlay(
                        transition_color,
                        1.0 - ((remaining - in_duration) / out_duration) as f32,
                    );

                    if let Some(tint) = transition_tint {
                        draw_overlay(
                            tint,
                            (1.0 - ((remaining - in_duration) / out_duration) as f32) * 0.25,
                        );
                    }
                }
            }
            SystemPhase::Layer => {
                if let Some(next_layer) = draw_layer(&mut state) {
                    let out_duration = if state.threatened {
                        play_sound(
                            &state.sad_sound,
                            PlaySoundParams {
                                looped: false,
                                volume: 1.0,
                            },
                        );
                        PHASE_TRANSITION / 4.0
                    } else {
                        PHASE_TRANSITION / 2.0
                    };

                    phase = SystemPhase::LayerTransition(
                        out_duration,
                        PHASE_TRANSITION,
                        next_layer,
                        true,
                    );

                    phase_start = time;
                    layer_reset = false;
                }
            }
        }

        macroquad::prelude::next_frame().await;
    }
}

/// Draws a layer.
fn draw_layer(state: &mut LayerState) -> Option<usize> {
    // Toggle debugger.
    if macroquad::prelude::is_key_released(KeyCode::E) {
        state.map.draw_debug_info = !state.map.draw_debug_info;
    }

    let mut next_layer = None;

    // Reset world state on objective completion or a keypress.
    if !state.remaining_objectives
        || macroquad::prelude::is_key_released(KeyCode::R)
        || state.threatened
    {
        if state.remaining_objectives {
            next_layer = Some(state.active_tilemap_index);

        // If all objectives are cleared, cycle to the next tilemap.
        } else {
            next_layer = Some((state.active_tilemap_index + 1) % state.tilemaps.len());
        }
    }

    // Reset active tile colors.
    state.map.set_tiles_from_bitmap(
        &state.tilemaps[state.active_tilemap_index],
        state.active_layer,
        state.wall_tile_texture.clone(),
        state.floor_tile_texture.clone(),
        0.75,
    );

    // Calculate mouse delta.
    let new_mouse_pos = Vec2::from(macroquad::prelude::mouse_position());
    let mouse_delta = state.mouse_pos - new_mouse_pos;
    state.mouse_pos = new_mouse_pos;

    // Pan the viewport.
    if macroquad::prelude::is_key_down(KeyCode::Space) {
        state.map.viewport_offset -= mouse_delta;
    }

    // Scale the viewport.
    let mouse_dy = macroquad::prelude::mouse_wheel().1;
    let scale_change = 0.01 * mouse_dy;
    let new_scale = state.map.viewport_scale + scale_change;
    state.map.viewport_scale = new_scale.clamp(1.0f32, 5.0f32);

    // Configure position translation.
    let last_pos = state.cosy_pos;

    // Velocity between frames should be stable.
    let frame_time = macroquad::prelude::get_frame_time();
    let velocity = 22.0 * frame_time;

    // If true, translation will move relative to
    // the viewport instead of the grid system.
    const VIEWPORT_RELATIVE_TRANSLATION: bool = true;

    // Process keyboard input.
    if macroquad::prelude::is_key_down(KeyCode::W) {
        if VIEWPORT_RELATIVE_TRANSLATION {
            state.cosy_pos.0 -= velocity;
        }

        state.cosy_pos.1 -= velocity;
        state.cosy_sprite = SPRITE_BACK;
        state.cosy_flip = true;
    }

    if macroquad::prelude::is_key_down(KeyCode::S) {
        if VIEWPORT_RELATIVE_TRANSLATION {
            state.cosy_pos.0 += velocity;
        }

        state.cosy_pos.1 += velocity;
        state.cosy_sprite = SPRITE;
    }

    if macroquad::prelude::is_key_down(KeyCode::A) {
        if VIEWPORT_RELATIVE_TRANSLATION {
            state.cosy_pos.1 += velocity / 2.0;
            state.cosy_pos.0 -= velocity / 2.0;
        } else {
            state.cosy_pos.0 -= velocity;
        }

        state.cosy_sprite = SPRITE;
        state.cosy_flip = false;
    }

    if macroquad::prelude::is_key_down(KeyCode::D) {
        if VIEWPORT_RELATIVE_TRANSLATION {
            state.cosy_pos.1 -= velocity / 2.0;
            state.cosy_pos.0 += velocity / 2.0;
        } else {
            state.cosy_pos.0 += velocity;
        }

        state.cosy_sprite = SPRITE;
        state.cosy_flip = true;
    }

    // Only permit moves which keep the avatar on the field.
    if state.cosy_pos.0 < 0.0
        || state.cosy_pos.0 > (WIDTH - 1) as f32
        || state.cosy_pos.1 < 0.0
        || state.cosy_pos.1 > (HEIGHT - 1) as f32
    {
        state.cosy_pos = last_pos;

    // Check for wall collisions.
    } else {
        let x = state.cosy_pos.0 as usize;
        let y = state.cosy_pos.1 as usize;
        let neighbor_tiles = vec![(x, y), (x + 1, y), (x, y + 1), (x + 1, y + 1)];

        for (x, y) in neighbor_tiles {
            if let Some(Tile::Filled { texture, .. }) = state.map.get_tile(x, y, state.active_layer)
            {
                if texture == &state.wall_tile_texture {
                    state.cosy_pos = last_pos;
                    break;
                }
            }
        }
    };

    // Highlight completed objective tiles.
    for line in &state.completed_objective_lines {
        for (x, y) in line.iter() {
            state.map.flood_fill_tiles(
                *x,
                *y,
                state.active_layer,
                color::ACCENT_1,
                color::ACCENT_3,
            );

            if let Some(Tile::Filled { blend_color, .. }) =
                state.map.get_tile(*x, *y, state.active_layer)
            {
                *blend_color = Some(color::ACCENT_3);
            };
        }
    }

    // Check if any objectives are remaining on the field.
    state.remaining_objectives = false;
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if state
                .map
                .tile_has_color(x, y, state.active_layer, color::ACCENT_1)
            {
                state.remaining_objectives = true;
                break;
            }
        }
    }

    // Highlight tiles between the sprite and the objective.
    let mut unbroken_line = true;
    let line_points = state.map.tiles_on_line_between(
        state.checkpoint.0,
        state.checkpoint.1,
        state.cosy_pos.0,
        state.cosy_pos.1,
    );
    for (x, y) in line_points.iter().take(line_points.len() - 1).skip(1) {
        if let Some(Tile::Filled {
            texture,
            blend_color,
            ..
        }) = state.map.get_tile(*x, *y, state.active_layer)
        {
            if texture == &state.wall_tile_texture {
                unbroken_line = false;
                break;
            }

            *blend_color = Some(color::ACCENT_3);
        };
    }

    // Detect if the sprite is on an objective tile.
    let mut on_objective = false;
    if state.map.tile_has_color(
        state.cosy_pos.0 as usize,
        state.cosy_pos.1 as usize,
        state.active_layer,
        color::ACCENT_1,
    ) {
        on_objective = true;
    }

    // Mark the objective complete and record a checkpoint.
    if unbroken_line && on_objective {
        state.completed_objective_lines.push(line_points);
        state.checkpoint = (state.cosy_pos.0, state.cosy_pos.1);
        play_sound(
            &state.happy_sound,
            PlaySoundParams {
                looped: false,
                volume: 1.0,
            },
        );
    }

    // Update threat radii.
    let time = macroquad::prelude::get_time();
    if time > state.threat_timestamp + state.threat_interval_seconds {
        state.threat_timestamp = time;
        state.threat_radius += 1;
        state.threat_radius %= state.max_threat_radius as i32;
    }

    // Identify threats.
    let mut rad_origins = vec![];
    let mut rads = vec![];
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            if state
                .map
                .tile_has_color(x, y, state.active_layer, color::ACCENT_2)
            {
                rad_origins.push((x, y));
                rads.push(state.map.tiles_on_radius(
                    x as isize,
                    y as isize,
                    3 + state.threat_radius as isize,
                ));
            }
        }
    }

    // Draw threat radii.
    for (rad_origin, rad_points) in rad_origins.into_iter().zip(rads.into_iter()) {
        for (x, y) in rad_points {
            if let Some(Tile::Filled { texture, .. }) = state.map.get_tile(x, y, state.active_layer)
            {
                if texture == &state.floor_tile_texture {
                    // TODO: Cheesy ray-tracing for occlusion on the wavefront.
                    let mut occluded = false;
                    let line_points = state.map.tiles_on_line_between(
                        rad_origin.0 as f32,
                        rad_origin.1 as f32,
                        x as f32,
                        y as f32,
                    );
                    for (x, y) in line_points.iter().take(line_points.len() - 1).skip(1) {
                        if let Some(Tile::Filled { texture, .. }) =
                            state.map.get_tile(*x, *y, state.active_layer)
                        {
                            if texture == &state.wall_tile_texture {
                                occluded = true;
                                break;
                            }
                        };
                    }

                    // If no occlusion is detected, draw the wavefront.
                    if !occluded {
                        if let Some(Tile::Filled { blend_color, .. }) =
                            state.map.get_tile(x, y, state.active_layer)
                        {
                            *blend_color = Some(color::ACCENT_2);
                        }
                    }
                }
            };
        }
    }

    // Detect if the sprite is on a threat tile.
    if state.map.tile_has_color(
        state.cosy_pos.0 as usize,
        state.cosy_pos.1 as usize,
        state.active_layer,
        color::ACCENT_2,
    ) {
        state.threatened = true;
    }

    // Redraw the map.
    macroquad::prelude::set_default_camera();
    state.map.draw_tiles();

    // Draw the cosi sprite onto the active layer.
    let sprite: Texture2D = Texture2D::from_file_with_format(state.cosy_sprite, None);
    sprite.set_filter(FilterMode::Nearest);
    state.map.draw_sprite(
        &sprite,
        state.cosy_pos.0,
        state.cosy_pos.1,
        0.5,
        state.active_layer,
        state.cosy_flip,
    );

    // Draw controls
    let screen_height = macroquad::prelude::screen_height();
    macroquad::prelude::draw_text("[space + cursor]", 10., screen_height - 80., 20., GRAY);
    macroquad::prelude::draw_text("[w a s d]: mvmnt", 10., screen_height - 60., 20., GRAY);
    macroquad::prelude::draw_text("[r]: reset layer", 10., screen_height - 40., 20., GRAY);
    macroquad::prelude::draw_text("[e]: toggle dbgr", 10., screen_height - 20., 20., GRAY);

    next_layer
}

/// Draws `splash_texture` as a full-screen, centered image.
fn draw_splash_screen(splash_texture: &Texture2D) {
    // Resize texture, preserving aspect ratio.
    let screen_width = macroquad::prelude::screen_width();
    let screen_height = macroquad::prelude::screen_height();

    let ratio_width = splash_texture.width() / screen_width;
    let ratio_height = splash_texture.height() / screen_height;

    let (width, height) = if ratio_width > ratio_height {
        (
            screen_width,
            (splash_texture.height() / ratio_width).round(),
        )
    } else {
        (
            (splash_texture.width() / ratio_height).round(),
            screen_height,
        )
    };

    let draw_params = DrawTextureParams {
        dest_size: Some(Vec2::new(width, height)),
        ..Default::default()
    };

    // Center texture on screen.
    let x = (screen_width - width) / 2.0;
    let y = (screen_height - height) / 2.0;

    // Empty the background.
    macroquad::prelude::clear_background(color::as_macroquad_color(color::BACKGROUND));

    // Draw the splash screen.
    macroquad::prelude::draw_texture_ex(splash_texture, x, y, WHITE, draw_params);
}

/// Draws a full-screen rectangle with `color` and `opacity`.
fn draw_overlay(mut color: Color, opacity: f32) {
    let screen_width = macroquad::prelude::screen_width();
    let screen_height = macroquad::prelude::screen_height();
    color.alpha = (opacity * 255.) as u8;
    macroquad::prelude::draw_rectangle(
        0.0,
        0.0,
        screen_width,
        screen_height,
        color::as_macroquad_color(color),
    );
}
/// TODO:
enum SystemPhase {
    /// Splash screen, with duration in seconds.
    Splash(f64),

    /// Transition to the layer with `index`.
    LayerTransition(f64, f64, usize, bool),

    /// Active layer.
    Layer,
}

/// TODO:
struct LayerState {
    // Textures.
    floor_tile_texture: TileTexture,
    wall_tile_texture: TileTexture,
    background_tile_texture: TileTexture,
    splash_texture: Texture2D,

    // Sounds.
    happy_sound: Sound,
    sad_sound: Sound,

    // Mouse info.
    mouse_pos: Vec2,

    // Tilemap state.
    map: TileMap,
    active_layer: i8,
    tilemaps: Vec<DynamicImage>,
    active_tilemap_index: usize,

    // Track sprite.
    cosy_pos: (f32, f32),
    cosy_sprite: &'static [u8],
    cosy_flip: bool,

    // Track all completed objectives.
    completed_objective_lines: Vec<Vec<(usize, usize)>>,
    checkpoint: (f32, f32),
    remaining_objectives: bool,

    // Track all threats.
    max_threat_radius: isize,
    threat_interval_seconds: f64,
    threat_radius: i32,
    threat_timestamp: f64,
    threatened: bool,
}

impl LayerState {
    /// TODO:
    pub async fn new() -> Self {
        // Load textures.
        let floor_tile_texture = TileTexture::from_bytes(FLOOR_TILE);
        let wall_tile_texture = TileTexture::from_bytes(WALL_TILE);
        let background_tile_texture = TileTexture::from_bytes(BACKGROUND_TILE);
        let splash_texture = Texture2D::from_file_with_format(SPLASH, None);
        splash_texture.set_filter(FilterMode::Linear);

        // Load sounds.
        let happy_sound = load_sound_from_bytes(HAPPY_SOUND).await.unwrap();
        let sad_sound = load_sound_from_bytes(SAD_SOUND).await.unwrap();

        // Load tilemaps.
        let mut tilemaps = vec![];
        for tilemap in TILEMAPS {
            let tilemap = image::load_from_memory(tilemap)
                .unwrap()
                .rotate270()
                .resize_exact(WIDTH as u32, HEIGHT as u32, FilterType::Nearest);
            tilemaps.push(tilemap);
        }

        // Initialize map.
        let mut map = crate::tile::TileMap::new(WIDTH, HEIGHT);
        map.draw_debug_info = false;
        map.viewport_scale = 1.0;

        // Track mouse position between frames.
        let mouse_pos = Vec2::from(macroquad::prelude::mouse_position());

        // Keep track of the layer the user can interact with.
        let active_layer = FOREGROUND_LAYER;
        let active_tilemap_index = 0;

        // Establish cosy's location on the grid.
        let cosy_pos = (0.0, 0.0);
        let cosy_sprite = SPRITE;
        let cosy_flip = false;

        // Track all completed objectives.
        let completed_objective_lines: Vec<Vec<(usize, usize)>> = vec![];
        let checkpoint = cosy_pos;
        let remaining_objectives = true;

        // Track all threats' states.
        let max_threat_radius = (HEIGHT.max(WIDTH) as f32 * 1.25) as isize;
        let threat_interval_seconds = 4.0 / max_threat_radius as f64;
        let threat_radius = 0;
        let threat_timestamp = macroquad::prelude::get_time();
        let threatened = false;

        Self {
            floor_tile_texture,
            wall_tile_texture,
            background_tile_texture,
            splash_texture,
            happy_sound,
            sad_sound,
            mouse_pos,
            map,
            active_layer,
            tilemaps,
            active_tilemap_index,
            cosy_pos,
            cosy_sprite,
            cosy_flip,
            completed_objective_lines,
            checkpoint,
            remaining_objectives,
            max_threat_radius,
            threat_interval_seconds,
            threat_radius,
            threat_timestamp,
            threatened,
        }
    }

    /// TODO:
    pub fn activate_layer(&mut self, layer: usize) {
        if layer as i8 != self.active_layer {
            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    if let Some(Tile::Filled {
                        blend_color: Some(color),
                        ..
                    }) = self.map.get_tile(x, y, self.active_layer)
                    {
                        if color.red >= 230 && color.green >= 230 && color.blue >= 230 {
                            color.alpha = 26;
                        }
                    }
                }
            }
        }

        self.active_tilemap_index = layer % self.tilemaps.len();
        self.active_layer = layer as i8;

        if layer == 0 {
            self.map.clear();

            for x in 0..WIDTH {
                for y in 0..HEIGHT {
                    self.map.set_tile(
                        x,
                        y,
                        BACKGROUND_LAYER,
                        Tile::Filled {
                            texture: self.background_tile_texture.clone(),
                            height_offset: None,
                            blend_color: None,
                        },
                    );
                }
            }
        }

        self.cosy_pos = self
            .map
            .set_tiles_from_bitmap(
                &self.tilemaps[self.active_tilemap_index],
                self.active_layer,
                self.wall_tile_texture.clone(),
                self.floor_tile_texture.clone(),
                0.75,
            )
            .unwrap();
        self.completed_objective_lines.clear();
        self.checkpoint = self.cosy_pos;
        self.remaining_objectives = true;

        self.threat_radius = 0;
        self.threat_timestamp = macroquad::prelude::get_time();
        self.threatened = false;
    }
}
