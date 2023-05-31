use std::collections::BTreeMap;

use rhai::Engine;

use crate::{
    connection::MidiBundle,
    global_state::Resources,
    node::{NodeIndex, NodeState},
    nodes::variants::NodeVariant,
    state::{FromNodeEngine, IoNodes, NodeEngineUpdate},
    traversal::buffered_traverser::{BufferedTraverser, TraverserResult},
};

#[derive(Debug)]
pub struct NodeEngine {
    pub current_time: i64,
    traverser: Option<BufferedTraverser>,
    io_nodes: Option<IoNodes>,
    scripting_engine: Engine,
    new_states: Vec<(NodeIndex, serde_json::Value)>,
    current_graph_state: Option<BTreeMap<NodeIndex, NodeState>>,
}

impl NodeEngine {
    pub fn new(traverser: BufferedTraverser, scripting_engine: Engine, io_nodes: IoNodes) -> NodeEngine {
        NodeEngine {
            current_time: 0,
            traverser: Some(traverser),
            io_nodes: Some(io_nodes),
            scripting_engine,
            new_states: vec![],
            current_graph_state: None,
        }
    }

    pub fn uninitialized() -> NodeEngine {
        let engine = rhai::Engine::new_raw();

        NodeEngine {
            current_time: 0,
            traverser: None,
            io_nodes: None,
            scripting_engine: engine,
            new_states: vec![],
            current_graph_state: None,
        }
    }

    pub fn initialized(&self) -> bool {
        self.traverser.is_some()
    }

    pub fn apply_state_updates(&mut self, updates: Vec<NodeEngineUpdate>) {
        for update in updates {
            match update {
                NodeEngineUpdate::NewNodeEngine(engine) => {
                    self.traverser = engine.traverser;
                    self.io_nodes = engine.io_nodes;
                    self.current_time = engine.current_time;
                }
                NodeEngineUpdate::NewDefaults(defaults) => {
                    if let Some(traverser) = &mut self.traverser {
                        for (node_index, socket, value) in defaults {
                            traverser.input_value_default(node_index, socket, value).unwrap();
                        }
                    }
                }
                NodeEngineUpdate::NewNodeState(incoming) => {
                    self.new_states.extend(incoming.into_iter());
                }
                NodeEngineUpdate::CurrentNodeStates(state) => {
                    self.current_graph_state = Some(state);
                }
            }
        }
    }

    pub fn step(&mut self, midi_in: MidiBundle, resources: &Resources, out: &mut [f32]) -> Vec<FromNodeEngine> {
        let mut messages_out = vec![];

        if let (Some(traverser), Some(io_nodes)) = (self.traverser.as_mut(), self.io_nodes.as_mut()) {
            if !midi_in.is_empty() {
                let input_node = traverser.get_node_mut(io_nodes.input);

                match input_node {
                    Some(NodeVariant::MidiInNode(node)) => {
                        node.set_midi_output(midi_in);
                    }
                    _ => {
                        unreachable!("Root input node is not midi input node")
                    }
                }
            }

            let TraverserResult {
                errors_and_warnings,
                state_changes,
                requested_state_updates,
                request_for_graph_state,
            } = traverser.traverse(
                self.current_time,
                &self.scripting_engine,
                resources,
                self.new_states.drain(..).collect(),
                self.current_graph_state.as_ref(),
            );
            self.current_time += out.len() as i64;
            self.current_graph_state = None;

            let output_node = traverser.get_node_mut(io_nodes.output);

            let output = match output_node {
                Some(NodeVariant::OutputNode(node)) => node.get_values_received(),
                _ => {
                    unreachable!("Root input midi node is not midi node")
                }
            };

            out.copy_from_slice(&output);

            if !state_changes.is_empty() {
                messages_out.push(FromNodeEngine::NodeStateUpdates(state_changes));
            }

            if !requested_state_updates.is_empty() {
                messages_out.push(FromNodeEngine::RequestedStateUpdates(requested_state_updates));
            }

            if request_for_graph_state {
                messages_out.push(FromNodeEngine::GraphStateRequested);
            }
        }

        messages_out
    }
}
