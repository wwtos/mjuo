use crate::nodes::prelude::*;

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct ButtonNode {
    state: bool,
    input: ProcessState<bool>,
}

impl NodeRuntime for ButtonNode {
    fn init(&mut self, _state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        self.input = ProcessState::Unprocessed(values_in[0].as_ref().and_then(|x| x.as_boolean()).unwrap());
    }

    fn set_ui_state(&mut self, state: serde_json::Value) {
        self.input = ProcessState::Unprocessed(state.as_bool().unwrap());
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        self.input = match self.input {
            ProcessState::Unprocessed(new_value) => {
                self.state = new_value;
                ProcessState::Processed
            }
            ProcessState::Processed => ProcessState::None,
            ProcessState::None => ProcessState::None,
        };

        NodeOk::no_warnings(())
    }

    fn get_ui_state(&self) -> Option<serde_json::Value> {
        if matches!(self.input, ProcessState::Processed) {
            Some(serde_json::Value::Bool(self.state))
        } else {
            None
        }
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        if matches!(self.input, ProcessState::Processed) {
            values_out[0] = Some(Primitive::Boolean(self.state));
        }
    }

    fn has_ui_state(&self) -> bool {
        true
    }
}

impl Node for ButtonNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        ButtonNode {
            state: false,
            input: ProcessState::None,
        }
    }

    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo {
            node_rows: vec![
                value_input(register("set_state"), Primitive::Boolean(false)),
                value_output(register("state")),
                NodeRow::Property("ui_name".into(), PropertyType::String, Property::String("".into())),
            ],
            child_graph_io: None,
        }
    }
}
