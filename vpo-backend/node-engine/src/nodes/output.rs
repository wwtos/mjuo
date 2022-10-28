use crate::connection::StreamSocketType;
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeRow};

#[derive(Debug, Clone)]
pub struct OutputNode {
    current_value: f32,
}

impl Default for OutputNode {
    fn default() -> Self {
        OutputNode { current_value: 0.0 }
    }
}

impl Node for OutputNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![NodeRow::StreamInput(StreamSocketType::Audio, 0.0)])
    }

    fn accept_stream_input(&mut self, _socket_type: &StreamSocketType, value: f32) {
        self.current_value = value;
    }

    fn get_stream_output(&self, _socket_type: &StreamSocketType) -> f32 {
        self.current_value
    }
}
