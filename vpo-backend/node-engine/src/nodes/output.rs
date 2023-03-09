use crate::connection::StreamSocketType;
use crate::errors::{NodeError, NodeOk, NodeResult};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};

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
        InitResult::simple(vec![NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false)])
    }

    fn process(&mut self, state: NodeProcessState, streams_in: &[f32], streams_out: &mut [f32]) -> NodeResult<()> {
        streams_out[0] = streams_in[0];

        NodeOk::no_warnings(())
    }
}
