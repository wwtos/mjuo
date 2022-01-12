use serde::{Deserialize, Serialize};
use serde_json;

use crate::errors::{Error, ErrorType};
use crate::node::{Node, SocketType, StreamSocketType, ValueType};

#[derive(Debug, Serialize, Deserialize)]
pub struct GainGraphNode {}

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

    fn accept_stream_input(&mut self, _socket_type: StreamSocketType, _value: f32) {}

    fn get_stream_output(&mut self, _socket_type: StreamSocketType) -> f32 {
        0_f32
    }

    fn serialize_to_json(&self) -> Result<serde_json::Value, Error> {
        match serde_json::to_value(self) {
            Ok(result) => Ok(result),
            Err(error) => Err(Error::new(error.to_string(), ErrorType::ParserError)),
        }
    }

    fn deserialize_from_json(json: serde_json::Value) -> Self
    where
        Self: Sized,
    {
        serde_json::from_value(json).unwrap()
    }
}
