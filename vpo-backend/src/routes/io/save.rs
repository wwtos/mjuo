use rfd::AsyncFileDialog;

use crate::{
    errors::EngineError,
    io::save,
    routes::{prelude::*, RouteReturn},
};

pub async fn route<'a>(state: RouteState<'a>) -> Result<RouteReturn, EngineError> {
    let mut new_project = false;

    if let Some(file_path) = &state.global_state.active_project {
        save(state.state, file_path)?;
    } else {
        let file = AsyncFileDialog::new().set_file_name("untitled.mjuo").save_file().await;

        if let Some(file) = file {
            save(state.state, file.path())?;

            state.global_state.active_project = Some(file.path().into());
            new_project = true;
        }
    }

    Ok(RouteReturn {
        new_project,
        engine_updates: vec![],
    })
}
