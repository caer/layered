use macroquad::prelude::*;

use crate::color;

use super::{Tile, TileMap};

impl TileMap {

    /// TODO: Utilities for interacting with tiles via a cursor.
    pub fn cursor_interaction(&mut self) {

        // TODO: Track variables between calls.
        let mut mouse_pos = Vec2::from(mouse_position());
        let mut mouse_select_a = None;
        let mut mouse_select_b = None;
        let mut active_layer = 0;

        // Capture mouse position in unit space.
        let cursor_point = self
            .view_to_grid(mouse_pos.x, mouse_pos.y, active_layer)
            .round();
        let (cursor_x, cursor_y) = (cursor_point.x as usize, cursor_point.y as usize);
        let cursor_on_grid =
            cursor_point.x >= 0.0 && cursor_x < self.width && cursor_point.y >= 0.0 && cursor_y < self.height;

        // Highlight regions of tiles on click.
        if is_mouse_button_released(MouseButton::Left) {
            match (
                cursor_on_grid,
                mouse_select_a.as_mut(),
                mouse_select_b.as_mut(),
            ) {
                // Set first point.
                (true, None, None) | (true, Some(..), Some(..)) => {
                    mouse_select_a = Some(Vec2::new(cursor_x as f32, cursor_y as f32));
                    mouse_select_b = None;
                }

                // Set second point.
                (true, Some(selection_a), None) => {
                    let a_x = selection_a.x.min(cursor_x as f32);
                    let a_y = selection_a.y.min(cursor_y as f32);
                    let b_x = selection_a.x.max(cursor_x as f32);
                    let b_y = selection_a.y.max(cursor_y as f32);

                    selection_a.x = a_x;
                    selection_a.y = a_y;
                    mouse_select_b = Some(Vec2::new(b_x, b_y));
                }

                // Clear selection.
                (false, _, _) => {
                    mouse_select_a = None;
                    mouse_select_b = None;
                }

                // Shouldn't be valid state.
                (true, None, Some(_)) => unreachable!(),
            }
        }

        // Delete tiles on right-click.
        if is_mouse_button_released(MouseButton::Right) {
            // Delete regions of tiles.
            if let (Some(a), Some(b)) = (mouse_select_a, mouse_select_b) {
                for x in a.x.floor() as usize..=b.x.ceil() as usize {
                    if x >= self.width {
                        continue;
                    }

                    for y in a.y.floor() as usize..=b.y.floor() as usize {
                        if y >= self.height {
                            continue;
                        }

                        self.set_tile(x, y, active_layer, Tile::Empty);
                    }
                }

                mouse_select_a = None;
                mouse_select_b = None;

            // Delete single tiles if no region is highlighted.
            } else if cursor_on_grid {
                self.set_tile(cursor_x, cursor_y, active_layer, Tile::Empty);
            }
        }

        // Apply regional tile highlights.
        match (mouse_select_a, mouse_select_b) {
            // Selection in-progress.
            (Some(a), None) => {
                let a_x = a.x.min(cursor_x as f32);
                let a_y = a.y.min(cursor_y as f32);
                let b_x = a.x.max(cursor_x as f32);
                let b_y = a.y.max(cursor_y as f32);

                for x in a_x.floor() as usize..=b_x.ceil() as usize {
                    if x >= self.width {
                        continue;
                    }

                    for y in a_y.floor() as usize..=b_y.floor() as usize {
                        if y >= self.height {
                            continue;
                        }

                        if let Some(Tile::Filled {
                            texture,
                            height_offset,
                            blend_color,
                        }) = self.get_tile(x, y, active_layer)
                        {
                            *height_offset = Some(0.05);
                            *blend_color = Some(color::ACCENT_2);
                        }
                    }
                }
            }

            // Selection complete.
            (Some(a), Some(b)) => {
                for x in a.x.floor() as usize..=b.x.ceil() as usize {
                    if x >= self.width {
                        continue;
                    }

                    for y in a.y.floor() as usize..=b.y.floor() as usize {
                        if y >= self.height {
                            continue;
                        }

                        if let Some(Tile::Filled {
                            texture,
                            height_offset,
                            blend_color,
                        }) = self.get_tile(x, y, active_layer)
                        {
                            *height_offset = Some(0.1);
                            *blend_color = Some(color::ACCENT_1);
                        }
                    }
                }
            }

            _ => (),
        }

        // Highlight the tile the mouse is hovering over and draw a sprite onto it.
        if cursor_on_grid {
            // Highlight the tile.
            if let Some(Tile::Filled {
                texture,
                height_offset,
                blend_color,
            }) = self.get_tile(cursor_x, cursor_y, active_layer)
            {
                *height_offset = Some(0.2);
                *blend_color = Some(color::ACCENT_1);
            };
        }
    }
}