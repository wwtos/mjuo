use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct MixerNode {
    input_count: i32,
    last_input_count: i32,
    input_sum: f32,
    output_audio: f32,
}

impl Default for MixerNode {
    fn default() -> Self {
        MixerNode {
            input_count: 2,
            last_input_count: 2,
            input_sum: 0.0,
            output_audio: 0.0,
        }
    }
}

impl NodeRuntime for MixerNode {
    fn process(
        &mut self,
        state: NodeProcessState,
        streams_in: &[f32],
        streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
        streams_out[0] = streams_in.iter().sum::<f32>() / streams_in.len() as f32;

        NodeOk::no_warnings(())
    }

    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        if let Some(Property::Integer(input_count)) = state.props.get("input_count") {
            self.input_count = *input_count;
        }

        InitResult::nothing()
    }
}

impl Node for MixerNode {
    fn get_io(props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        let mut node_rows = vec![
            NodeRow::Property("input_count".to_string(), PropertyType::Integer, Property::Integer(2)),
            stream_output(register("audio"), 0.0),
        ];

        if let Some(Property::Integer(input_count)) = props.get("input_count") {
            for i in 0..(*input_count) {
                node_rows.push(NodeRow::Input(
                    Socket::Numbered(register("socket-input-numbered"), i + 1, SocketType::Stream, 1),
                    SocketValue::Stream(0.0),
                ));
            }
        }

        NodeIo::simple(node_rows)
    }
}
