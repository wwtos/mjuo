use std::path::Path;

use node_engine::state::NodeEngineUpdate;
use rfd::AsyncFileDialog;
use snafu::ResultExt;

use crate::{
    errors::EngineError,
    io::load,
    routes::{prelude::*, RouteReturn},
    util::{send_global_state_updates, send_graph_updates},
};

pub async fn route<'a>(state: RouteState<'a>) -> Result<RouteReturn, EngineError> {
    let file = AsyncFileDialog::new().pick_file().await;
    let resources = &mut *state.resources_lock.write().unwrap();

    if let Some(file) = file {
        let path = file.path();

        state.global_state.active_project = Some(path.into());

        state.state.clear_history();
        load(Path::new(path), state.state, state.global_state, resources)?;

        send_global_state_updates(state.global_state, state.to_server)?;
        send_graph_updates(state.state, state.state.get_root_graph_index(), state.to_server)?;

        return Ok(RouteReturn {
            engine_updates: vec![NodeEngineUpdate::NewNodeEngine(
                state
                    .state
                    .get_engine(state.global_state, resources)
                    .whatever_context("could not create traverser")?,
            )],
            new_project: true,
        });
    }

    Ok(RouteReturn::default())
}
