use rhai::Engine;

use crate::{
    connection::MidiBundle, global_state::Resources, node::NodeIndex, nodes::variants::NodeVariant,
    traversal::buffered_traverser::BufferedTraverser,
};

pub struct NodeEngine {
    current_time: i64,
    traverser: BufferedTraverser,
    output_node: NodeIndex,
    midi_in_node: NodeIndex,
    scripting_engine: Engine,
}

impl NodeEngine {
    pub fn new(
        traverser: BufferedTraverser,
        scripting_engine: Engine,
        midi_in_node: NodeIndex,
        output_node: NodeIndex,
    ) -> NodeEngine {
        NodeEngine {
            current_time: 0,
            traverser,
            output_node,
            midi_in_node,
            scripting_engine,
        }
    }

    pub fn step(&mut self, midi_in: MidiBundle, resources: &Resources, out: &mut [f32]) {
        if !midi_in.is_empty() {
            let midi_in_node = self.traverser.get_node_mut(self.midi_in_node);

            match midi_in_node {
                Some(NodeVariant::MidiInNode(node)) => {
                    node.set_midi_output(midi_in);
                }
                _ => {
                    unreachable!("Root input midi node is not midi node")
                }
            }
        }

        let traversal_errors = self
            .traverser
            .traverse(self.current_time, &self.scripting_engine, resources);
        self.current_time += out.len() as i64;

        let output_node = self.traverser.get_node_mut(self.output_node);

        let output = match output_node {
            Some(NodeVariant::OutputNode(node)) => node.get_values_received(),
            _ => {
                unreachable!("Root input midi node is not midi node")
            }
        };

        out.copy_from_slice(&output)
    }
}
