use std::collections::BTreeMap;

use clocked::midi::MidiMessage;

use crate::{
    node::{NodeIndex, NodeState},
    nodes::NodeVariant,
    resources::Resources,
    state::{FromNodeEngine, IoNodes, ToNodeEngine},
    traversal::buffered_traverser::{BufferedTraverser, StepResult},
};

#[derive(Debug)]
pub struct NodeEngine {
    pub current_time: i64,
    io_nodes: Option<IoNodes>,
    new_states: Vec<(NodeIndex, serde_json::Value)>,
    current_graph_state: Option<BTreeMap<NodeIndex, NodeState>>,
    traverser: Option<BufferedTraverser>,
}

impl NodeEngine {
    pub fn uninitialized() -> NodeEngine {
        NodeEngine {
            current_time: 0,
            io_nodes: None,
            new_states: vec![],
            current_graph_state: None,
            traverser: None,
        }
    }

    pub fn initialized(&self) -> bool {
        self.traverser.is_some()
    }

    pub fn new(traverser: BufferedTraverser, io_nodes: IoNodes) -> NodeEngine {
        NodeEngine {
            current_time: 0,
            io_nodes: Some(io_nodes),
            new_states: vec![],
            current_graph_state: None,
            traverser: Some(traverser),
        }
    }

    pub fn apply_state_updates(&mut self, updates: Vec<ToNodeEngine>) {}

    pub fn step(
        &mut self,
        midi_in: Vec<Vec<MidiMessage>>,
        resources: &Resources,
        out: &mut [f32],
    ) -> Vec<FromNodeEngine> {
        let mut messages_out = vec![];

        if let (Some(traverser), Some(io_nodes)) = (self.traverser.as_mut(), self.io_nodes.as_mut()) {
            if !midi_in.is_empty() {
                let input_node = traverser.get_node_mut(io_nodes.input);

                match input_node {
                    Some(NodeVariant::InputsNode(node)) => {
                        node.set_midis(midi_in);
                    }
                    _ => {
                        unreachable!("Root input node is not midi input node")
                    }
                }
            }

            let StepResult {
                errors_and_warnings,
                state_changes,
                requested_state_updates,
                request_for_graph_state,
            } = traverser.step(
                resources,
                self.new_states.drain(..).collect(),
                self.current_graph_state.as_ref(),
            );

            self.current_time += out.len() as i64;
            self.current_graph_state = None;

            if !errors_and_warnings.errors.is_empty() || !errors_and_warnings.warnings.is_empty() {
                println!(
                    "errors: {:?}\nwarnings: {:?}",
                    errors_and_warnings.errors, errors_and_warnings.warnings
                );
            }

            let output_node = traverser.get_node_mut(io_nodes.output);

            let output = match output_node {
                Some(NodeVariant::OutputsNode(node)) => node.get_streams(),
                _ => {
                    unreachable!("Root input midi node is not midi node")
                }
            };

            out.copy_from_slice(&output[0][0]);

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
