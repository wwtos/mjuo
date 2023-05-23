use serde_json::{json, Value};

use super::prelude::*;

#[derive(Debug, Clone)]
enum MemoryMode {
    Loading,
    Setting,
    MapSetting,
    WaitingForNodeStates { map_setting: bool },
}

#[derive(Debug, Clone)]
pub struct MemoryNode {
    tracking: Vec<NodeIndex>,
    memory: Vec<(NodeIndex, Value)>,
    mode: MemoryMode,
    activated: bool,
    state_changed: bool,
}

impl NodeRuntime for MemoryNode {
    fn init(&mut self, state: NodeInitState, _child_graph: Option<NodeGraphAndIo>) -> NodeResult<InitResult> {
        let NodeInitState { state: node_state, .. } = state;

        self.memory = serde_json::from_value(node_state.other.get("memory").unwrap_or(&Value::Null).clone())
            .unwrap_or_else(|_| vec![]);
        self.tracking = serde_json::from_value(node_state.other.get("tracking").unwrap_or(&Value::Null).clone())
            .unwrap_or_else(|_| vec![]);

        InitResult::nothing()
    }

    fn has_state(&self) -> bool {
        true
    }

    fn set_state(&mut self, node_state: serde_json::Value) {
        self.memory =
            serde_json::from_value(node_state.get("memory").unwrap_or(&Value::Null).clone()).unwrap_or_else(|_| vec![]);
        self.tracking = serde_json::from_value(node_state.get("tracking").unwrap_or(&Value::Null).clone())
            .unwrap_or_else(|_| vec![]);

        self.state_changed = true;
    }

    fn accept_value_inputs(&mut self, values_in: &[Option<Primitive>]) {
        if let Some(true) = values_in[0].as_ref().and_then(|x| x.as_boolean()) {
            self.activated = true;
        }

        if let Some(true) = values_in[1].as_ref().and_then(|x| x.as_boolean()) {
            self.mode = MemoryMode::Loading;
        } else if let Some(true) = values_in[2].as_ref().and_then(|x| x.as_boolean()) {
            self.mode = MemoryMode::Setting;
        } else if let Some(true) = values_in[3].as_ref().and_then(|x| x.as_boolean()) {
            self.mode = MemoryMode::MapSetting;
        }
    }

    fn process(
        &mut self,
        state: NodeProcessState,
        _streams_in: &[&[f32]],
        _streams_out: &mut [&mut [f32]],
    ) -> NodeResult<()> {
        if self.activated {
            match self.mode {
                MemoryMode::Loading => {
                    println!("loading");
                    (state.state.enqueue_state_updates)(self.memory.clone());
                }
                MemoryMode::Setting => {
                    (state.state.request_node_states)();

                    self.mode = MemoryMode::WaitingForNodeStates { map_setting: false };
                }
                MemoryMode::MapSetting => {
                    (state.state.request_node_states)();

                    self.mode = MemoryMode::WaitingForNodeStates { map_setting: true };
                }
                _ => {}
            }

            self.state_changed = true;
        }

        if let Some(node_states) = state.state.states {
            if let MemoryMode::WaitingForNodeStates { map_setting } = self.mode {
                if map_setting {
                    self.tracking.clear();
                    self.memory.clear();

                    for (node_index, node_state) in node_states {
                        if node_state.counted_during_mapset {
                            self.tracking.push(*node_index);
                            self.memory.push((*node_index, node_state.value.clone()));
                        }
                    }

                    self.mode = MemoryMode::MapSetting;
                } else {
                    self.memory.clear();

                    for node_index in &self.tracking {
                        if let Some(node_state) = node_states.get(&node_index) {
                            self.memory.push((*node_index, node_state.value.clone()));
                        }
                    }

                    self.mode = MemoryMode::Setting;
                }

                self.state_changed = true;
            }
        }

        ProcessResult::nothing()
    }

    fn get_state(&self) -> Option<NodeState> {
        if self.state_changed {
            Some(NodeState {
                counted_during_mapset: false,
                value: Value::Null,
                other: json!({
                    "memory": self.memory,
                    "tracking": self.tracking
                }),
            })
        } else {
            None
        }
    }

    fn finish(&mut self) {
        self.state_changed = false;
        self.activated = false;
    }
}

impl Node for MemoryNode {
    fn get_io(_props: HashMap<String, Property>, register: &mut dyn FnMut(&str) -> u32) -> NodeIo {
        NodeIo::simple(vec![
            value_input(register("activate"), Primitive::Boolean(false)),
            value_input(register("load_mode"), Primitive::Boolean(false)),
            value_input(register("set_mode"), Primitive::Boolean(false)),
            value_input(register("map_set_mode"), Primitive::Boolean(false)),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MemoryNode {
            tracking: Vec::with_capacity(256),
            memory: Vec::with_capacity(256),
            mode: MemoryMode::Loading,
            activated: false,
            state_changed: false,
        }
    }
}
