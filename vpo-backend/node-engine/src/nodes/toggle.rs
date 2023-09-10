use serde_json::Value;

use crate::nodes::prelude::*;

use super::util::ProcessState;

#[derive(Debug, Clone)]
pub struct ToggleNode {
    state: bool,
    first_time: bool,
    updated_state: ProcessState<()>,
}

impl NodeRuntime for ToggleNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        self.first_time = true;

        if let Some(new_state) = params.node_state.value.as_bool() {
            self.state = new_state;
        }

        self.updated_state = ProcessState::Unprocessed(());

        InitResult::nothing()
    }

    fn process(
        &mut self,
        _context: NodeProcessContext,
        ins: Ins,
        outs: Outs,
        resources: &[&dyn Any],
    ) -> NodeResult<()> {
        if let Some(new_state) = ins.values[0][0].as_boolean() {
            if !self.first_time {
                self.state = new_state;
                self.updated_state = ProcessState::Unprocessed(());
            }
        }

        if matches!(self.updated_state, ProcessState::Unprocessed(())) || self.first_time {
            outs.values[0][0] = bool(self.state);
        }

        self.updated_state = match self.updated_state {
            ProcessState::Unprocessed(_) => ProcessState::Processed,
            ProcessState::Processed => ProcessState::None,
            ProcessState::None => ProcessState::None,
        };

        self.first_time = false;

        ProcessResult::nothing()
    }

    fn set_state(&mut self, state: serde_json::Value) {
        self.state = state.as_bool().unwrap_or(false);

        self.updated_state = ProcessState::Unprocessed(());
    }

    fn get_state(&self) -> Option<NodeState> {
        if matches!(
            self.updated_state,
            ProcessState::Unprocessed(()) | ProcessState::Processed
        ) {
            Some(NodeState {
                counted_during_mapset: self.state,
                value: Value::Bool(self.state),
                other: Value::Null,
            })
        } else {
            None
        }
    }

    fn has_state(&self) -> bool {
        true
    }
}

impl Node for ToggleNode {
    fn new(_sound_config: &SoundConfig) -> Self {
        ToggleNode {
            state: false,
            updated_state: ProcessState::None,
            first_time: true,
        }
    }

    fn get_io(context: NodeGetIoContext, props: HashMap<String, Property>) -> NodeIo {
        NodeIo {
            node_rows: vec![
                value_input("set_state", Primitive::Boolean(false), 1),
                value_output("state", 1),
                NodeRow::Property("ui_name".into(), PropertyType::String, Property::String("".into())),
            ],
            child_graph_io: None,
        }
    }
}
