use std::path::Path;

pub struct GlobalState {
    active_project: Option<Box<Path>>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState { active_project: None }
    }
}
