# Refactoring TODO: Decouple Game from Engine

This document outlines refactoring work needed to separate the "Layered" game from the underlying 2.5D graphics engine.

## 1. Extract Game-Specific Assets and Constants from `lib.rs`

**Current Problem:** 
The `lib.rs` file (which should be the engine library) contains all game-specific assets, constants, and game logic:
- Game assets (FLOOR_TILE, WALL_TILE, SPRITE, SPLASH, sounds, tilemaps)
- Game-specific constants (WIDTH=48, HEIGHT=48, layer definitions)
- The entire `simulation_loop()` function containing game-specific logic
- `LayerState` struct with game-specific state management

**Tasks:**
1. [x] Create new directory structure: `src/engine/` and `src/game/`
2. [x] Move engine components to `src/engine/`:
   - [x] `tile.rs` → `src/engine/tile.rs`
   - [x] `color.rs` → `src/engine/color.rs`
   - [x] `asset.rs` → `src/engine/asset.rs`
3. [x] Move game-specific code to `src/game/`:
   - [x] Move `simulation_loop()` to `src/game/mod.rs`
   - [x] Move `LayerState` to `src/game/state.rs`
   - [x] Move `SystemPhase` enum to `src/game/phase.rs`
   - [x] Move all asset constants (lines 29-48 in lib.rs) to `src/game/assets.rs`
   - [x] Move game constants (WIDTH, HEIGHT, FOREGROUND_LAYER, etc.) to `src/game/config.rs`
4. [x] Update `lib.rs` to re-export only engine components
5. [x] Update `main.rs` to import from `game` module
6. [x] Update module declarations and imports throughout codebase

**Success Criteria:**
- `lib.rs` contains only engine exports
- Game-specific code is isolated in `src/game/`
- Engine can be used independently of the Layered game

## 2. Remove Game-Specific Color Semantics from Engine Code

**Current Problem:**
The `tile.rs` module has hardcoded game logic based on specific color meanings:
- `set_tiles_from_bitmap()` method (lines 420-486) assumes specific color semantics:
  - `BACKGROUND` color = walls
  - `ACCENT_1` = objectives
  - `ACCENT_2` = danger/threats  
  - `ACCENT_3` = avatar spawn point
- The `tile_has_color()` and `flood_fill_tiles()` methods are designed around these game-specific colors

**Tasks:**
1. [ ] Create `TileMapBuilder` trait/struct in engine
2. [ ] Refactor `set_tiles_from_bitmap()` to accept a color mapping callback:
   ```rust
   pub fn set_tiles_from_bitmap<F>(
       &mut self,
       bitmap: &DynamicImage,
       layer: i8,
       color_mapper: F,
   ) -> Option<(f32, f32)>
   where
       F: Fn(u32, u32, Rgba<u8>) -> Option<Tile>,
   ```
3. [ ] Move color-to-tile-type mapping logic to `src/game/level_loader.rs`
4. [ ] Make `tile_has_color()` more generic or move to game code
5. [ ] Make `flood_fill_tiles()` accept color parameters instead of using constants
6. [ ] Update game code to provide color mapping functions
7. [ ] Remove hardcoded `color::ACCENT_*` references from engine code

**Success Criteria:**
- Engine code has no knowledge of specific color meanings
- Game provides color interpretation logic
- Other games can use different color schemes without modifying engine

## 3. Separate Input Handling from Engine Rendering

**Current Problem:**
Game-specific input handling is mixed with engine rendering in `lib.rs`:
- `draw_layer()` function (lines 179-497) handles both:
  - Game input (WASD, mouse, space for pan, E for debug, R for reset)
  - Game logic (collision detection, objective tracking, threat system)
  - Engine rendering calls

**Tasks:**
1. [ ] Create `InputSystem` in engine (`src/engine/input.rs`):
   - [ ] Define `InputState` struct to capture raw input
   - [ ] Define `InputEvent` enum for discrete events
   - [ ] Implement `fn capture_input() -> InputState`
2. [ ] Create game input interpreter (`src/game/input.rs`):
   - [ ] Move WASD/mouse interpretation to game code
   - [ ] Move debug toggle (E key) logic to game code
   - [ ] Move reset (R key) logic to game code
   - [ ] Move viewport pan (Space + mouse) to game code
3. [ ] Separate `draw_layer()` into:
   - [ ] `game::update()` - handles input and updates game state
   - [ ] `game::render()` - calls engine rendering functions
4. [ ] Create clear API boundary:
   ```rust
   // Engine provides:
   let input = engine::capture_input();
   // Game interprets:
   game.update(input);
   // Engine renders:
   engine.render(game.render_commands());
   ```

**Success Criteria:**
- Engine only captures and provides raw input
- Game decides how to interpret input
- Rendering code is separate from input handling

## 4. Extract Game Logic (Objectives, Threats, State Machine) from Rendering Code

**Current Problem:**
Complex game-specific logic is embedded throughout rendering code:
- Objective/checkpoint system (lines 376-396 in lib.rs)
- Threat radius calculation and propagation (lines 398-471 in lib.rs)
- `SystemPhase` enum and phase transitions (lines 550-559, 77-175 in lib.rs)
- Level progression logic mixed with rendering

**Tasks:**
1. [ ] Create game systems in `src/game/systems/`:
   - [ ] `objective.rs` - ObjectiveSystem for tracking and validating objectives
     - [ ] Move objective detection logic
     - [ ] Move checkpoint system
     - [ ] Move line validation logic
   - [ ] `threat.rs` - ThreatSystem for threat radius and detection
     - [ ] Move threat radius calculation
     - [ ] Move threat propagation with occlusion
     - [ ] Move threat timing logic
   - [ ] `phase.rs` - PhaseSystem for level transitions
     - [ ] Move SystemPhase enum
     - [ ] Move phase transition logic
     - [ ] Move splash screen timing
2. [ ] Define clean system interfaces:
   ```rust
   trait GameSystem {
       fn update(&mut self, delta_time: f32, map: &TileMap);
       fn render_commands(&self) -> Vec<RenderCommand>;
   }
   ```
3. [ ] Create `RenderCommand` enum in engine for game→engine communication
4. [ ] Refactor `simulation_loop()` to orchestrate systems:
   ```rust
   loop {
       input = engine.capture_input();
       for system in systems {
           system.update(delta_time, input, &map);
       }
       engine.render(collect_render_commands(&systems));
   }
   ```

**Success Criteria:**
- Game logic is organized into separate, testable systems
- Rendering code only handles drawing, not game logic
- Clear separation between game state updates and rendering

## 5. Create a Plugin/Configuration System for Game-Specific Tile Behavior

**Current Problem:**
The engine's `TileMap` has hardcoded knowledge of:
- How to interpret bitmaps as game levels
- Specific tile types (wall vs floor semantics)
- Game-specific viewport initialization (line 149: `viewport_offset.y -= (view_size.y / height as f32) * 3.0`)
- Game-specific collision detection integrated into the map

**Tasks:**
1. [ ] Create `TileMapConfig` struct in engine:
   ```rust
   pub struct TileMapConfig {
       pub viewport_offset_calculator: Box<dyn Fn(usize, usize) -> Vec2>,
       pub collision_detector: Box<dyn Fn(&TileMap, f32, f32, i8) -> bool>,
       pub custom_initializer: Option<Box<dyn Fn(&mut TileMap)>>,
   }
   ```
2. [ ] Add plugin hooks to `TileMap`:
   - [ ] `on_tile_render` - customize tile rendering
   - [ ] `on_collision_check` - customize collision detection
   - [ ] `on_viewport_init` - customize initial viewport
3. [ ] Refactor `TileMap::new()` to accept optional config:
   ```rust
   pub fn new(width: usize, height: usize) -> Self
   pub fn with_config(width: usize, height: usize, config: TileMapConfig) -> Self
   ```
4. [ ] Move hardcoded viewport offset calculation to game config
5. [ ] Make collision detection pluggable
6. [ ] Remove assumptions about tile semantics from engine

**Success Criteria:**
- TileMap is configurable without modifying engine code
- Games can provide custom behavior through configuration
- Engine has sensible defaults but allows customization

## Additional Improvements (Future Considerations)

### Documentation
1. [ ] Add engine API documentation with examples
2. [ ] Create tutorial for building a game with the engine
3. [ ] Document color system and tile rendering pipeline

### Testing
1. [ ] Add unit tests for engine components
2. [ ] Add integration tests for game systems
3. [ ] Separate test fixtures for engine vs game

### Performance
1. [ ] Profile rendering pipeline
2. [ ] Consider spatial partitioning for large maps
3. [ ] Optimize tile rendering batch operations

### API Design
1. [ ] Consider making engine available as separate crate
2. [ ] Design public API with semver in mind
3. [ ] Add feature flags for optional engine features

## Implementation Order

Recommended implementation sequence:

1. **Phase 1:** Refactoring #1 (Extract Assets/Constants)
   - Provides foundation for all other work
   - Low risk, high impact on code organization

2. **Phase 2:** Refactoring #2 (Color Semantics)
   - Core architectural improvement
   - Requires Phase 1 to be complete

3. **Phase 3:** Refactoring #4 (Extract Game Logic)
   - Improves maintainability significantly
   - Can be done incrementally (one system at a time)

4. **Phase 4:** Refactoring #3 (Input Handling)
   - Benefits from cleaner game logic separation
   - Completes the input→update→render cycle separation

5. **Phase 5:** Refactoring #5 (Plugin System)
   - Polish and extensibility
   - Can be done last as it's "nice-to-have"