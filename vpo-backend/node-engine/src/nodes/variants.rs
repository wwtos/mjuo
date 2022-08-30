use crate::connection::{MidiSocketType, Primitive, SocketDirection, SocketType, StreamSocketType, ValueSocketType};
use crate::errors::ErrorsAndWarnings;
use crate::node::{InitResult, NodeIndex};
use crate::node_graph::NodeGraph;
use crate::property::Property;
use crate::socket_registry::SocketRegistry;
use crate::traversal::traverser::Traverser;

use rhai::Engine;
use sound_engine::midi::messages::MidiData;
use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use sound_engine::SoundConfig;

use crate::{errors::NodeError, node::Node};

#[cfg(test)]
use crate::graph_tests::TestNode;

use super::function_node::FunctionNode;
use super::inputs::InputsNode;
use super::outputs::OutputsNode;
use super::polyphonic::PolyphonicNode;
use super::stream_expression::StreamExpressionNode;
use super::{
    biquad_filter::BiquadFilterNode, dummy::DummyNode, envelope::EnvelopeNode, expression::ExpressionNode,
    gain::GainGraphNode, midi_input::MidiInNode, midi_to_values::MidiToValuesNode, mixer::MixerNode,
    oscillator::OscillatorNode, output::OutputNode,
};

#[enum_dispatch]
#[derive(Debug, Clone)]
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
    DummyNode,
    FunctionNode,
    InputsNode,
    OutputsNode,
    StreamExpressionNode,
    PolyphonicNode,
    #[cfg(test)]
    TestNode,
}

impl Default for NodeVariant {
    fn default() -> Self {
        NodeVariant::DummyNode(DummyNode::default())
    }
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
        "DummyNode" => Ok(NodeVariant::DummyNode(DummyNode::default())),
        "FunctionNode" => Ok(NodeVariant::FunctionNode(FunctionNode::default())),
        "InputsNode" => Ok(NodeVariant::InputsNode(InputsNode::default())),
        "OutputsNode" => Ok(NodeVariant::OutputsNode(OutputsNode::default())),
        "StreamExpressionNode" => Ok(NodeVariant::StreamExpressionNode(StreamExpressionNode::new())),
        "PolyphonicNode" => Ok(NodeVariant::PolyphonicNode(PolyphonicNode::default())),
        #[cfg(test)]
        "TestNode" => Ok(NodeVariant::TestNode(TestNode::default())),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}

pub fn variant_to_name(variant: &NodeVariant) -> String {
    match variant {
        NodeVariant::GainGraphNode(_) => "GainGraphNode".to_string(),
        NodeVariant::OutputNode(_) => "OutputNode".to_string(),
        NodeVariant::OscillatorNode(_) => "OscillatorNode".to_string(),
        NodeVariant::MidiInNode(_) => "MidiInNode".to_string(),
        NodeVariant::MidiToValuesNode(_) => "MidiToValuesNode".to_string(),
        NodeVariant::EnvelopeNode(_) => "EnvelopeNode".to_string(),
        NodeVariant::BiquadFilterNode(_) => "BiquadFilterNode".to_string(),
        NodeVariant::MixerNode(_) => "MixerNode".to_string(),
        NodeVariant::ExpressionNode(_) => "ExpressionNode".to_string(),
        NodeVariant::DummyNode(_) => "DummyNode".to_string(),
        NodeVariant::FunctionNode(_) => "FunctionNode".to_string(),
        NodeVariant::InputsNode(_) => "InputsNode".to_string(),
        NodeVariant::OutputsNode(_) => "OutputsNode".to_string(),
        NodeVariant::StreamExpressionNode(_) => "StreamExpressionNode".to_string(),
        NodeVariant::PolyphonicNode(_) => "PolyphonicNode".to_string(),
        #[cfg(test)]
        NodeVariant::TestNode(_) => "TestNode".to_string(),
    }
}
