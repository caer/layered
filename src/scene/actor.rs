use codas_flow::{
    stage::{Proc, Procs},
    Flow,
};

use super::{ActorState, Input, SceneData};

/// An actor within a [`Scene`](super::Scene).
pub struct Actor {
    pub state: ActorState,
    pub state_output: Flow<SceneData>,
}

impl Procs<Input> for Actor {
    fn proc(&mut self, _: &mut Proc, data: &Input) {
        if data.x > 0.0 {
            self.state.x += 1;
        } else if data.x < 0.0 && self.state.x > 0 {
            self.state.x -= 1;
        }

        if data.y > 0.0 {
            self.state.y += 1;
        } else if data.y < 0.0 && self.state.y > 0 {
            self.state.y -= 1;
        }

        self.state_output
            .try_next()
            .unwrap()
            .publish(self.state.clone().into());
    }
}
