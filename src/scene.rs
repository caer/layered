//! Scene containing coda-driven actors.

use codas_flow::{stage::Stage, Flow};
use codas_macros::export_coda;

pub mod actor;

export_coda!("src/scene/coda.md");

// Default flow capacities for a scene.
const FLOW_CAPACITY: usize = 256;

/// A scene of [`Actors`] on a [`TileMap`].
pub struct Scene {
    /// Events flowing to the actors.
    actor_stage: Stage<SceneData>,

    /// Events flowing to the scene.
    scene_stage: Stage<SceneData>,
}

impl Scene {
    /// Returns a new scene with `width` and `height`.
    pub fn new(_width: usize, _height: usize) -> Self {
        // Open the flows.
        let (_, [actor_sub]) = Flow::new(FLOW_CAPACITY);
        let actor_stage = Stage::from(actor_sub);
        let (_, [scene_sub]) = Flow::new(FLOW_CAPACITY);
        let scene_stage = Stage::from(scene_sub);

        Self {
            actor_stage,
            scene_stage,
        }
    }

    /// Runs the scene for one simulation step.
    pub fn step(&mut self) {
        let x = 0.0f32;
        let y = 0.0f32;

        // Publish input events.
        // if is_key_released(KeyCode::W) {
        //     y += 1.0;
        // }
        // if is_key_released(KeyCode::S) {
        //     y -= 1.0;
        // }
        // if is_key_released(KeyCode::A) {
        //     x -= 1.0;
        // }
        // if is_key_released(KeyCode::D) {
        //     x += 1.0;
        // }
        if x != 0.0 || y != 0.0 {
            let mut flow = self.actor_stage.flow();
            let next = flow.try_next().unwrap();
            next.publish(Input { x, y }.into());
        }

        // Step all stages forward.
        let mut processed = 0;
        processed += self.actor_stage.proc().unwrap_or_default();
        processed += self.scene_stage.proc().unwrap_or_default();

        // If the stages did no work, rest a while.
        if processed == 0 {}
    }

    // Draws a single frame of the current scene state.
    pub fn draw(&mut self) {}
}

#[cfg(test)]
mod tests {
    use codas_flow::stage::Proc;

    use super::{actor::Actor, *};

    #[test]
    fn actors() {
        let mut scene = Scene::new(32, 32);

        scene.actor_stage.add_proc(Actor {
            state: Default::default(),
            state_output: scene.scene_stage.flow(),
        });

        scene
            .scene_stage
            .add_proc(|_: &mut Proc, data: &ActorState| {
                eprintln!("actor proc'd: {data:?}");
            });

        scene.step();
    }
}
