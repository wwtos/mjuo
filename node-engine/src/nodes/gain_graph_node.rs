use serde::{Deserialize, Serialize};
use serde_json;

use crate::connection::{SocketType, StreamSocketType, ValueType};
use crate::errors::{Error, ErrorType};
use crate::node::Node;

#[derive(Debug, Serialize, Deserialize)]
pub struct GainGraphNode {}

impl GainGraphNode {
    pub fn new() -> Self {
        GainGraphNode {}
    }
}

impl Node for GainGraphNode {
    fn list_input_sockets(&self) -> Vec<SocketType> {
        vec![
            SocketType::Stream(StreamSocketType::Audio),
            SocketType::Value(ValueType::Gain),
        ]
    }

    fn list_output_sockets(&self) -> Vec<SocketType> {
        vec![SocketType::Stream(StreamSocketType::Audio)]
    }

    fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        match serde_json::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::new(error.to_string(), ErrorType::ParserError)),
        }
    }

    fn deserialize_from_json(json: serde_json::Value) -> Self {
        serde_json::from_value(json).unwrap()
    }
}
