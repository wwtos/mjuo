use std::path::PathBuf;

use serde_json::json;

use crate::io::clocked::DeviceManager;

#[derive(Debug)]
pub struct GlobalState {
    pub active_project: Option<PathBuf>,
    pub import_folder: Option<PathBuf>,
    pub device_manager: DeviceManager,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            active_project: None,
            import_folder: None,
            device_manager: DeviceManager::new(),
        }
    }

    pub fn project_directory(&self) -> Option<PathBuf> {
        self.active_project
            .as_ref()
            .and_then(|project| project.parent())
            .map(|dir| dir.into())
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "activeProject": self.active_project,
            "devices": self.device_manager.devices_as_json(),
        })
    }
}
