use serde_json::Value;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct ToggleNode {
    state: bool,
    updated: bool,
    first_time: bool,
}

impl NodeRuntime for ToggleNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        self.first_time = true;

        if let Some(new_state) = state.state.value.as_bool() {
            self.state = new_state;
            self.updated = true;
        }

        InitResult::nothing()
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(new_state) = values_in[0].as_ref().and_then(|x| x.as_boolean()) {
            if !self.first_time {
                self.state = new_state;
                self.updated = true;
            }
        }
    }

    fn set_state(&mut self, state: serde_json::Value) {
        self.state = state.as_bool().unwrap_or(false);

        self.updated = true;
    }

    fn get_state(&self) -> Option<NodeState> {
        if self.updated {
            Some(NodeState {
                counted_during_mapset: self.state,
                value: Value::Bool(self.state),
                other: Value::Null,
            })
        } else {
            None
        }
    }

    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {
        if self.updated || self.first_time {
            values_out[0] = Some(Primitive::Boolean(self.state));
        }
    }

    fn has_state(&self) -> bool {
        true
    }

    fn finish(&mut self) {
        self.updated = false;
        self.first_time = false;
    }
}

impl Node for ToggleNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        ToggleNode {
            state: false,
            updated: false,
            first_time: true,
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