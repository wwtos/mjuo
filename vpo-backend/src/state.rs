use std::path::PathBuf;

pub struct GlobalState {
    pub active_project: Option<PathBuf>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState { active_project: None }
    }
}
