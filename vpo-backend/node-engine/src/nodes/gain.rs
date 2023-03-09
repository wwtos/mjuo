use crate::connection::StreamSocketType;
use crate::errors::{NodeError, NodeOk};
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow};
use crate::property::Property;

#[derive(Debug, Clone)]
pub struct GainGraphNode {
    gain: f32,
    value: f32,
}

impl Default for GainGraphNode {
    fn default() -> Self {
        GainGraphNode { gain: 0.2, value: 0.0 }
    }
}

impl Node for GainGraphNode {
    fn init(&mut self, state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        if let Some(Property::Float(gain)) = state.props.get("default_gain") {
            self.gain = gain.clamp(0.0, 1.0);
        }

        InitResult::simple(vec![
            NodeRow::StreamInput(StreamSocketType::Audio, 0.0, false),
            NodeRow::StreamInput(StreamSocketType::Gain, 0.0, false),
            NodeRow::StreamOutput(StreamSocketType::Audio, 0.0, false),
        ])
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out[0] = streams_in[0] * streams_in[1];

        NodeOk::no_warnings(())
    }
}
