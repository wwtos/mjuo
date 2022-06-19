use serde::{Deserialize, Serialize};
use sound_engine::SoundConfig;

use crate::{errors::NodeError, node::Node};

#[cfg(test)]
use crate::graph_tests::TestNode;

use super::{
    envelope::EnvelopeNode, gain::GainGraphNode, midi_input::MidiInNode,
    midi_to_values::MidiToValuesNode, oscillator::OscillatorNode, output::OutputNode,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "content")]
pub enum NodeVariant {
    GainGraphNode(GainGraphNode),
    OutputNode(OutputNode),
    OscillatorNode(OscillatorNode),
    MidiInNode(MidiInNode),
    MidiToValuesNode(MidiToValuesNode),
    EnvelopeNode(EnvelopeNode),
    #[cfg(test)]
    TestNode(TestNode),
}

pub fn new_variant(node_type: &str, config: &SoundConfig) -> Result<NodeVariant, NodeError> {
    match node_type {
        "GainGraphNode" => Ok(NodeVariant::GainGraphNode(GainGraphNode::default())),
        "OscillatorNode" => Ok(NodeVariant::OscillatorNode(OscillatorNode::default())),
        "MidiToValuesNode" => Ok(NodeVariant::MidiToValuesNode(MidiToValuesNode::default())),
        "EnvelopeNode" => Ok(NodeVariant::EnvelopeNode(EnvelopeNode::new(config))),
        #[cfg(test)]
        "TestNode" => Ok(NodeVariant::TestNode(TestNode::default())),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}

pub fn variant_to_name(variant: &NodeVariant) -> String {
    match variant {
        NodeVariant::GainGraphNode(_) => "Gain graph node".to_string(),
        NodeVariant::OutputNode(_) => "Output node".to_string(),
        NodeVariant::OscillatorNode(_) => "Oscillator node".to_string(),
        NodeVariant::MidiInNode(_) => "Midi in node".to_string(),
        NodeVariant::MidiToValuesNode(_) => "Midi to values node".to_string(),
        NodeVariant::EnvelopeNode(_) => "Envelope node".to_string(),
        #[cfg(test)]
        NodeVariant::TestNode(_) => "Test node".to_string(),
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
            Self::EnvelopeNode(node) => node as &dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &dyn Node,
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
            Self::EnvelopeNode(node) => node as &mut dyn Node,
            #[cfg(test)]
            Self::TestNode(node) => node as &mut dyn Node,
        }
    }
}
