use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketRegistry {
    name_to_socket_type: HashMap<String, u32>,
    #[serde(skip)]
    uid_counter: u32,
}

impl SocketRegistry {
    pub fn new() -> SocketRegistry {
        SocketRegistry {
            name_to_socket_type: HashMap::new(),
            uid_counter: 0,
        }
    }

    pub fn register_socket(&mut self, name: &str) -> u32 {
        if let Some(uid) = self.name_to_socket_type.get(name) {
            *uid
        } else {
            let uid = self.uid_counter;
            self.uid_counter += 1;

            self.name_to_socket_type.insert(name.to_string(), uid);

            uid
        }
    }
}

impl Default for SocketRegistry {
    fn default() -> Self {
        Self::new()
    }
}
