use std::path::Path;

use rfd::AsyncFileDialog;
use snafu::ResultExt;

use crate::{
    engine::ToAudioThread,
    errors::EngineError,
    io::load,
    routes::{prelude::*, RouteReturn},
    util::{send_global_state_updates, send_graph_updates, send_resource_updates},
};

pub async fn route<'a>(state: RouteState<'a>) -> Result<RouteReturn, EngineError> {
    let file = AsyncFileDialog::new().pick_file().await;
    let resources = &mut *state.resources_lock.write().unwrap();

    if let Some(file) = file {
        let path = file.path();

        state.global_state.active_project = Some(path.into());

        state.state.clear_history();
        load(Path::new(path), state.state, resources, state.state.get_sound_config())?;

        send_global_state_updates(&state.global_state, state.to_server)?;
        send_graph_updates(state.state, state.state.get_root_graph_index(), state.to_server)?;
        send_resource_updates(resources, state.to_server)?;

        return Ok(RouteReturn {
            engine_updates: vec![ToAudioThread::NewTraverser(
                state
                    .state
                    .get_traverser(resources)
                    .whatever_context("could not create traverser")?,
            )],
            new_project: true,
        });
    }

    Ok(RouteReturn::default())
}
