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
    state_changed: bool,
}

impl NodeRuntime for MemoryNode {
    fn init(&mut self, params: NodeInitParams) -> NodeResult<InitResult> {
        let NodeInitParams { node_state, .. } = params;

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

    fn process<'a>(
        &mut self,
        context: NodeProcessContext,
        ins: Ins<'a>,
        _outs: Outs<'a>,
        _midi_store: &mut MidiStore,
        _resources: &[Resource],
    ) {
        self.state_changed = false;

        if let Some(()) = ins.value(1)[0].as_bang() {
            self.mode = MemoryMode::Loading;
        } else if let Some(()) = ins.value(2)[0].as_bang() {
            self.mode = MemoryMode::Setting;
        } else if let Some(()) = ins.value(3)[0].as_bang() {
            self.mode = MemoryMode::MapSetting;
        }

        if let Some(()) = ins.value(0)[0].as_bang() {
            match self.mode {
                MemoryMode::Loading => {
                    (context.external_state.enqueue_state_updates)(self.memory.clone());
                }
                MemoryMode::Setting => {
                    (context.external_state.request_node_states)();

                    self.mode = MemoryMode::WaitingForNodeStates { map_setting: false };
                }
                MemoryMode::MapSetting => {
                    (context.external_state.request_node_states)();

                    self.mode = MemoryMode::WaitingForNodeStates { map_setting: true };
                }
                _ => {}
            }

            self.state_changed = true;
        }

        if let Some(node_states) = context.external_state.states {
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
}

impl Node for MemoryNode {
    fn get_io(_context: NodeGetIoContext, _props: SeaHashMap<String, Property>) -> NodeIo {
        NodeIo::simple(vec![
            value_input("activate", Primitive::Bang, 1),
            value_input("load_mode", Primitive::Bang, 1),
            value_input("set_mode", Primitive::Bang, 1),
            value_input("map_set_mode", Primitive::Bang, 1),
        ])
    }

    fn new(_sound_config: &SoundConfig) -> Self {
        MemoryNode {
            tracking: Vec::with_capacity(256),
            memory: Vec::with_capacity(256),
            mode: MemoryMode::Loading,
            state_changed: false,
        }
    }
}
