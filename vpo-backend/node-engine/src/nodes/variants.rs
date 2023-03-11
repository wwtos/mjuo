use enum_dispatch::enum_dispatch;

use sound_engine::SoundConfig;

use crate::connection::{MidiBundle, Primitive};
use crate::errors::NodeResult;
use crate::errors::{NodeError, NodeOk};
use crate::node::NodeGraphAndIo;
use crate::node::{InitResult, Node, NodeInitState, NodeProcessState};

use super::button::ButtonNode;
use super::function_node::FunctionNode;
use super::inputs::InputsNode;
use super::midi_filter::MidiFilterNode;
use super::mono_sample_player::MonoSamplePlayerNode;
use super::outputs::OutputsNode;
use super::polyphonic::PolyphonicNode;
use super::portamento::PortamentoNode;
use super::rank_player::RankPlayerNode;
use super::stream_expression::StreamExpressionNode;
use super::wavetable::WavetableNode;
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
    MidiFilterNode,
    MonoSamplePlayerNode,
    WavetableNode,
    PortamentoNode,
    ButtonNode,
    RankPlayerNode,
}

impl Default for NodeVariant {
    fn default() -> Self {
        NodeVariant::DummyNode(DummyNode::default())
    }
}

pub fn new_variant(node_type: &str, config: &SoundConfig) -> Result<NodeVariant, NodeError> {
    match node_type {
        "OutputNode" => Ok(NodeVariant::OutputNode(OutputNode::default())),
        "MidiInNode" => Ok(NodeVariant::MidiInNode(MidiInNode::default())),
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
        "PolyphonicNode" => Ok(NodeVariant::PolyphonicNode(PolyphonicNode::new(config))),
        "MidiFilterNode" => Ok(NodeVariant::MidiFilterNode(MidiFilterNode::new())),
        "MonoSamplePlayerNode" => Ok(NodeVariant::MonoSamplePlayerNode(MonoSamplePlayerNode::default())),
        "WavetableNode" => Ok(NodeVariant::WavetableNode(WavetableNode::new(config))),
        "PortamentoNode" => Ok(NodeVariant::PortamentoNode(PortamentoNode::new(config))),
        "ButtonNode" => Ok(NodeVariant::ButtonNode(ButtonNode::new())),
        "RankPlayerNode" => Ok(NodeVariant::RankPlayerNode(RankPlayerNode::default())),
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
        NodeVariant::MidiFilterNode(_) => "MidiFilterNode".to_string(),
        NodeVariant::MonoSamplePlayerNode(_) => "MonoSamplePlayerNode".to_string(),
        NodeVariant::WavetableNode(_) => "WavetableNode".to_string(),
        NodeVariant::PortamentoNode(_) => "PortamentoNode".to_string(),
        NodeVariant::ButtonNode(_) => "ButtonNode".to_string(),
        NodeVariant::RankPlayerNode(_) => "RankPlayerNode".to_string(),
    }
}
