use serde_json::json;

use crate::nodes::prelude::*;

#[derive(Debug, Clone)]
pub struct ButtonNode {
    state: bool,
    updated: bool,
}

impl NodeRuntime for ButtonNode {
    fn init(&mut self, _state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        InitResult::nothing()
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(new_state) = values_in[0].as_ref().and_then(|x| x.as_boolean()) {
            self.state = new_state;
            self.updated = true;
        }
    }

    fn set_state(&mut self, state: serde_json::Value) {
        self.state = state.get("value").and_then(|x| x.as_bool()).unwrap_or(false);
        self.updated = true;
    }

    fn get_state(&self) -> Option<serde_json::Value> {
        if self.updated {
            Some(json! ({
                "value": self.state
            }))
        } else {
            None
        }
    }

    fn get_value_outputs(&mut self, values_out: &mut [Option<Primitive>]) {
        if self.updated {
            values_out[0] = Some(Primitive::Boolean(self.state));
        }
    }

    fn has_state(&self) -> bool {
        true
    }

    fn finish(&mut self) {
        self.updated = false;
    }
}

impl Node for ButtonNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        ButtonNode {
            state: false,
            updated: false,
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
