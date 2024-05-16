use rfd::AsyncFileDialog;

use crate::{
    errors::EngineError,
    io::save,
    routes::{prelude::*, RouteReturn},
};

pub async fn route<'a>(state: RouteCtx<'a>) -> Result<RouteReturn, EngineError> {
    let file = AsyncFileDialog::new().set_file_name("untitled.mjuo").save_file().await;

    if let Some(file) = file {
        save(state.state, file.path())?;

        state.global_state.active_project = Some(file.path().into());
    }

    Ok(RouteReturn::default())
}
