use crate::nodes::prelude::*;

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct ButtonNode {
    state: bool,
    input: ProcessState<bool>,
}

impl ButtonNode {
    pub fn new() -> Self {
        ButtonNode {
            state: false,
            input: ProcessState::None,
        }
    }
}

impl NodeRuntime for ButtonNode {
    fn init(&mut self, state: NodeInitState, child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn process(
        &mut self,
        _state: NodeProcessState,
        _streams_in: &[f32],
        _streams_out: &mut [f32],
    ) -> Result<NodeOk<()>, NodeError> {
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

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        self.input = ProcessState::Unprocessed(values_in[0].clone().unwrap().as_boolean().unwrap());
    }

    fn get_value_outputs(&self, values_out: &mut [Option<Primitive>]) {
        if matches!(self.input, ProcessState::Processed) {
            values_out[0] = Some(Primitive::Boolean(self.state));
        }
    }

    fn linked_to_ui(&self) -> bool {
        true
    }
}

impl Node for ButtonNode {
    fn get_io(props: HashMap<String, Property>) -> NodeIo {
        NodeIo {
            node_rows: vec![
                value_input("state", Primitive::Boolean(false)),
                value_output("state", Primitive::Boolean(false)),
            ],
            child_graph_io: None,
        }
    }
}
