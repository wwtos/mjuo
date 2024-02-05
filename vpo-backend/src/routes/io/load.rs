use std::path::Path;

use log::info;
use node_engine::{
    io_routing::IoRoutes,
    state::{ActionInvalidation, GraphState},
};
use rfd::AsyncFileDialog;
use snafu::ResultExt;
use sound_engine::SoundConfig;

use crate::{
    engine::ToAudioThread,
    errors::EngineError,
    io::load_state,
    routes::{prelude::*, RouteReturn},
    util::{send_graph_updates, send_project_state_updates, send_resource_updates},
};

pub async fn route<'a>(mut ctx: RouteState<'a>) -> Result<RouteReturn, EngineError> {
    let file = AsyncFileDialog::new().pick_file().await;
    let resources = &mut *ctx.resources_lock.write().unwrap();

    if let Some(file) = file {
        let path = file.path();

        ctx.global_state.active_project = Some(path.into());

        // reset everything
        ctx.to_audio_thread.send(ToAudioThread::Reset).unwrap();
        *ctx.state = GraphState::new(SoundConfig::default());
        ctx.global_state.device_manager.reset();
        resources.reset();

        load_state(Path::new(path), ctx.state.get_sound_config(), &mut ctx.state, resources)?;

        send_project_state_updates(&ctx.state, &ctx.global_state, ctx.to_server)?;
        send_graph_updates(ctx.state, ctx.state.get_root_graph_index(), ctx.to_server)?;
        send_resource_updates(resources, ctx.to_server)?;

        // handle new audio devices
        let new_rules = ctx.state.get_route_rules();

        info!("Connecting devices...");
        let mut to_audio_thread = state_invalidations(
            &mut ctx.state,
            vec![ActionInvalidation::NewRouteRules {
                last_rules: IoRoutes::default(),
                new_rules,
            }],
            &mut ctx.global_state.device_manager,
            resources,
        )?;

        to_audio_thread.push(ToAudioThread::NewTraverser(
            ctx.state
                .create_traverser(resources)
                .whatever_context("could not create traverser")?
                .1,
        ));

        return Ok(RouteReturn {
            engine_updates: to_audio_thread,
            new_project: true,
        });
    }

    Ok(RouteReturn::default())
}
