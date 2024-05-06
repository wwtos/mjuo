use crate::{routes::prelude::*, util::send_project_state_updates};

pub fn route(ctx: RouteCtx) -> Result<RouteReturn, EngineError> {
    ctx.global_state.device_manager.rescan_devices();

    send_project_state_updates(&ctx.state, &ctx.global_state, ctx.to_server)?;

    Ok(RouteReturn::default())
}
