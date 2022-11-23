use std::collections::HashMap;

use crate::{
    connection::{Primitive, ValueSocketType},
    errors::{NodeError, NodeOk},
    node::{InitResult, Node, NodeInitState, NodeProcessState, NodeRow},
    property::Property,
    ui::ui_node::UiNode,
};

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

impl UiNode for ButtonNode {
    fn has_new_state(&self) -> bool {
        matches!(self.input, ProcessState::Processed)
    }

    fn get_new_state(&self) -> HashMap<String, Property> {
        let mut new_state = HashMap::new();
        new_state.insert("state".to_string(), Property::Bool(self.state));

        new_state
    }

    fn apply_state(&mut self, state: HashMap<String, Property>) {
        // if let Some(state) = state.get("state").and_then(|x| x.as_boolean()) {}
    }
}

impl Node for ButtonNode {
    fn init(&mut self, _state: NodeInitState) -> Result<NodeOk<InitResult>, NodeError> {
        InitResult::simple(vec![
            NodeRow::ValueInput(ValueSocketType::State, Primitive::Boolean(false), false),
            NodeRow::ValueOutput(ValueSocketType::State, Primitive::Boolean(false), false),
        ])
    }

    fn process(&mut self, state: NodeProcessState) -> Result<NodeOk<()>, NodeError> {
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

    fn accept_value_input(&mut self, _socket_type: &ValueSocketType, value: Primitive) {
        self.input = ProcessState::Unprocessed(value.as_boolean().unwrap());
    }

    fn get_value_output(&self, _socket_type: &ValueSocketType) -> Option<Primitive> {
        if matches!(self.input, ProcessState::Processed) {
            Some(Primitive::Boolean(self.state))
        } else {
            None
        }
    }
}
