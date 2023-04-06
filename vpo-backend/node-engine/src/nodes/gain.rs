use crate::nodes::prelude::*;

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

impl NodeRuntime for GainGraphNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::Float(gain)) = state.props.get("default_gain") {
            self.gain = gain.clamp(0.0, 1.0);
        }

        InitResult::nothing()
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

impl Node for GainGraphNode {
    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            stream_input("audio", 0.0),
            stream_input("gain", 0.0),
            stream_output("audio", 0.0),
        ])
    }
}
