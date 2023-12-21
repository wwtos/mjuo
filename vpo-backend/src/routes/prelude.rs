use node_engine::errors::NodeError;
use node_engine::resources::Resources;
pub use node_engine::state::ActionBundle;
use node_engine::state::{ActionInvalidation, GraphState};

pub use crate::engine::ToAudioThread;
pub use crate::errors::EngineError;
pub use crate::routes::{RouteReturn, RouteState};

pub fn state_invalidations(
    state: &mut GraphState,
    invalidations: Vec<ActionInvalidation>,
    resources: &Resources,
) -> Result<Vec<ToAudioThread>, NodeError> {
    Ok(state
        .invalidations_to_engine_updates(invalidations, resources)?
        .into_iter()
        .map(|x| ToAudioThread::NodeEngineUpdate(x))
        .collect())
}
