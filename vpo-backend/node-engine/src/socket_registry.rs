use std::collections::HashMap;
use std::mem;

use crate::{errors::NodeError, connection::NodeRefSocketType};
use serde::{Serialize, Deserialize};
use serde_json::Value;

use crate::connection::{
    MidiSocketType, SocketType, StreamSocketType, ValueSocketType,
};

#[derive(Serialize, Deserialize)]
pub struct RegistryValue {
    pub template: String,
    pub socket_type: SocketType,
    pub associated_data: Value,
}

pub struct SocketRegistry {
    name_to_socket_type: HashMap<String, RegistryValue>,
    uid_counter: u64,
}

impl SocketRegistry {
    pub fn new() -> SocketRegistry {
        SocketRegistry {
            name_to_socket_type: HashMap::new(),
            uid_counter: 0,
        }
    }

    pub fn register_socket(
        &mut self,
        name: String,
        socket_type: SocketType,
        template: String,
        associated_data: Option<Value>,
    ) -> Result<SocketType, NodeError> {
        if let Some(registry_value) = self.name_to_socket_type.get(&name) {
            if mem::discriminant(&socket_type) != mem::discriminant(&registry_value.socket_type) {
                return Err(NodeError::RegistryCollision(name));
            }

            Ok(registry_value.socket_type.clone())
        } else {
            let uid = self.uid_counter;
            self.uid_counter += 1;

            let new_socket_type = match socket_type {
                SocketType::Stream(_) => SocketType::Stream(StreamSocketType::Dynamic(uid)),
                SocketType::Midi(_) => SocketType::Midi(MidiSocketType::Dynamic(uid)),
                SocketType::Value(_) => SocketType::Value(ValueSocketType::Dynamic(uid)),
                SocketType::NodeRef(_) => SocketType::NodeRef(NodeRefSocketType::Dynamic(uid)),
                SocketType::MethodCall(_) => todo!(),
            };

            self.name_to_socket_type.insert(
                name,
                RegistryValue {
                    template: template,
                    socket_type: new_socket_type.clone(),
                    associated_data: match associated_data {
                        Some(value) => value,
                        None => Value::Null,
                    },
                },
            );

            Ok(new_socket_type)
        }
    }

    
}
