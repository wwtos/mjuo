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

    fn linked_to_ui(&self) -> bool {
        true
    }
}
