use std::collections::HashMap;
use crate::property::Property;
use crate::socket_registry::SocketRegistry;
use crate::node::InitResult;
use crate::errors::ErrorsAndWarnings;
use crate::connection::MidiSocketType;
use crate::connection::ValueSocketType;
use crate::connection::Primitive;
use crate::connection::StreamSocketType;
use sound_engine::midi::messages::MidiData;

use enum_dispatch::enum_dispatch;
use sound_engine::SoundConfig;

use crate::{errors::NodeError, node::Node};

#[cfg(test)]
use crate::graph_tests::TestNode;

use super::{
    biquad_filter::BiquadFilterNode, envelope::EnvelopeNode, gain::GainGraphNode,
    midi_input::MidiInNode, midi_to_values::MidiToValuesNode, mixer::MixerNode,
    oscillator::OscillatorNode, output::OutputNode, expression::ExpressionNode,
};

#[enum_dispatch]
#[derive(Debug)]
pub enum NodeVariant {
    GainGraphNode,
    OutputNode,
    OscillatorNode,
    MidiInNode,
    MidiToValuesNode,
    EnvelopeNode,
    BiquadFilterNode,
    MixerNode,
    ExpressionNode,
    #[cfg(test)]
    TestNode,
}

pub fn new_variant(node_type: &str, config: &SoundConfig) -> Result<NodeVariant, NodeError> {
    match node_type {
        "GainGraphNode" => Ok(NodeVariant::GainGraphNode(GainGraphNode::default())),
        "OscillatorNode" => Ok(NodeVariant::OscillatorNode(OscillatorNode::default())),
        "MidiToValuesNode" => Ok(NodeVariant::MidiToValuesNode(MidiToValuesNode::default())),
        "EnvelopeNode" => Ok(NodeVariant::EnvelopeNode(EnvelopeNode::new(config))),
        "BiquadFilterNode" => Ok(NodeVariant::BiquadFilterNode(BiquadFilterNode::new(config))),
        "MixerNode" => Ok(NodeVariant::MixerNode(MixerNode::default())),
        "ExpressionNode" => Ok(NodeVariant::ExpressionNode(ExpressionNode::new())),
        #[cfg(test)]
        "TestNode" => Ok(NodeVariant::TestNode(TestNode::default())),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}

pub fn variant_to_name(variant: &NodeVariant) -> String {
    match variant {
        NodeVariant::GainGraphNode(_) => "gainGraphNode".to_string(),
        NodeVariant::OutputNode(_) => "outputNode".to_string(),
        NodeVariant::OscillatorNode(_) => "oscillatorNode".to_string(),
        NodeVariant::MidiInNode(_) => "midiInNode".to_string(),
        NodeVariant::MidiToValuesNode(_) => "midiToValuesNode".to_string(),
        NodeVariant::EnvelopeNode(_) => "envelopeNode".to_string(),
        NodeVariant::BiquadFilterNode(_) => "biquadFilterNode".to_string(),
        NodeVariant::MixerNode(_) => "mixerNode".to_string(),
        NodeVariant::ExpressionNode(_) => "expressionNode".to_string(),
        #[cfg(test)]
        NodeVariant::TestNode(_) => "testNode".to_string(),
    }
}
