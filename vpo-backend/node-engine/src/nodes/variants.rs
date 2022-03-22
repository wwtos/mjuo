use serde::{Serialize, Deserialize};

use crate::{node::Node, nodes::gain_graph_node::GainGraphNode, errors::NodeError};

#[cfg(test)]
use crate::graph_tests::TestNode;

use super::{output::OutputNode, oscillator::OscillatorNode, midi_input::MidiInNode, midi_to_values::MidiToValuesNode};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode),
    OutputNode(OutputNode),
    OscillatorNode(OscillatorNode),
    MidiInNode(MidiInNode),
    MidiToValuesNode(MidiToValuesNode),
    #[cfg(test)]
    TestNode(TestNode)
}

pub fn new_variant(node_type: &str) -> Result<NodeVariant, NodeError> {
    match node_type {
        "GainGraphNode" => Ok(NodeVariant::GainGraphNode(GainGraphNode::default())),
        "OscillatorNode" => Ok(NodeVariant::OscillatorNode(OscillatorNode::default())),
        "MidiToValuesNode" => Ok(NodeVariant::MidiToValuesNode(MidiToValuesNode::default())),
        #[cfg(test)]
        "TestNode" => Ok(NodeVariant::TestNode(TestNode::default())),
        _ => Err(NodeError::NodeTypeDoesNotExist)
    }
}

pub fn variant_to_name(variant: &NodeVariant) -> String {
    match variant {
        NodeVariant::GainGraphNode(_) => "Gain graph node".to_string(),
        NodeVariant::OutputNode(_) => "Output node".to_string(),
        NodeVariant::OscillatorNode(_) => "Oscillator node".to_string(),
        NodeVariant::MidiInNode(_) => "Midi in node".to_string(),
        NodeVariant::MidiToValuesNode(_) => "Midi to values node".to_string(),
        #[cfg(test)]
        NodeVariant::TestNode(_) => "Test node".to_string()
    }
}

impl<'a> AsRef<dyn Node + 'a> for NodeVariant {
    fn as_ref(&self) -> &(dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &dyn Node,
            Self::OutputNode(node) => node as &dyn Node,
            Self::OscillatorNode(node) => node as &dyn Node,
            Self::MidiInNode(node) => node as &dyn Node,
            Self::MidiToValuesNode(node) => node as &dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &dyn Node
        }
    }
}

impl<'a> AsMut<dyn Node + 'a> for NodeVariant {
    fn as_mut(&mut self) -> &mut (dyn Node + 'a) {
        match self {
            Self::GainGraphNode(node) => node as &mut dyn Node,
            Self::OutputNode(node) => node as &mut dyn Node,
            Self::OscillatorNode(node) => node as &mut dyn Node,
            Self::MidiInNode(node) => node as &mut dyn Node,
            Self::MidiToValuesNode(node) => node as &mut dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &mut dyn Node
        }
    }
}
