use std::collections::HashMap;

use enum_dispatch::enum_dispatch;

use sound_engine::SoundConfig;

use crate::connection::{MidiBundle, Primitive};
use crate::errors::NodeError;
use crate::errors::NodeResult;
use crate::node::{InitResult, Node, NodeGraphAndIo, NodeInitState, NodeIo, NodeProcessState, NodeRuntime, NodeState};
use crate::property::Property;

use super::button::ButtonNode;
use super::function_node::FunctionNode;
use super::inputs::InputsNode;
use super::midi_filter::MidiFilterNode;
use super::outputs::OutputsNode;
use super::polyphonic::PolyphonicNode;
use super::portamento::PortamentoNode;
use super::rank_player::RankPlayerNode;
use super::stream_expression::StreamExpressionNode;
use super::wavetable::WavetableNode;
use super::{
    biquad_filter::BiquadFilterNode, dummy::DummyNode, envelope::EnvelopeNode, expression::ExpressionNode,
    gain::GainNode, memory::MemoryNode, midi_input::MidiInNode, midi_merger::MidiMergerNode,
    midi_to_values::MidiToValuesNode, midi_transpose::MidiTransposeNode, mixer::MixerNode, oscillator::OscillatorNode,
    output::OutputNode, wavetable_sequencer::WavetableSequencerNode,
};

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum NodeVariant {
    GainNode,
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
    WavetableNode,
    PortamentoNode,
    ButtonNode,
    RankPlayerNode,
    MidiMergerNode,
    MidiTransposeNode,
    WavetableSequencerNode,
    MemoryNode,
}

impl Default for NodeVariant {
    fn default() -> Self {
        NodeVariant::DummyNode(DummyNode::default())
    }
}

pub fn new_variant(node_type: &str, config: &SoundConfig) -> Result<NodeVariant, NodeError> {
    match node_type {
        "OutputNode" => Ok(NodeVariant::OutputNode(OutputNode::new(config))),
        "MidiInNode" => Ok(NodeVariant::MidiInNode(MidiInNode::new(config))),
        "GainNode" => Ok(NodeVariant::GainNode(GainNode::new(config))),
        "OscillatorNode" => Ok(NodeVariant::OscillatorNode(OscillatorNode::new(config))),
        "MidiToValuesNode" => Ok(NodeVariant::MidiToValuesNode(MidiToValuesNode::new(config))),
        "EnvelopeNode" => Ok(NodeVariant::EnvelopeNode(EnvelopeNode::new(config))),
        "BiquadFilterNode" => Ok(NodeVariant::BiquadFilterNode(BiquadFilterNode::new(config))),
        "MixerNode" => Ok(NodeVariant::MixerNode(MixerNode::new(config))),
        "ExpressionNode" => Ok(NodeVariant::ExpressionNode(ExpressionNode::new(config))),
        "DummyNode" => Ok(NodeVariant::DummyNode(DummyNode::new(config))),
        "FunctionNode" => Ok(NodeVariant::FunctionNode(FunctionNode::new(config))),
        "InputsNode" => Ok(NodeVariant::InputsNode(InputsNode::new(config))),
        "OutputsNode" => Ok(NodeVariant::OutputsNode(OutputsNode::new(config))),
        "StreamExpressionNode" => Ok(NodeVariant::StreamExpressionNode(StreamExpressionNode::new(config))),
        "PolyphonicNode" => Ok(NodeVariant::PolyphonicNode(PolyphonicNode::new(config))),
        "MidiFilterNode" => Ok(NodeVariant::MidiFilterNode(MidiFilterNode::new(config))),
        "WavetableNode" => Ok(NodeVariant::WavetableNode(WavetableNode::new(config))),
        "PortamentoNode" => Ok(NodeVariant::PortamentoNode(PortamentoNode::new(config))),
        "ButtonNode" => Ok(NodeVariant::ButtonNode(ButtonNode::new(config))),
        "RankPlayerNode" => Ok(NodeVariant::RankPlayerNode(RankPlayerNode::new(config))),
        "MidiMergerNode" => Ok(NodeVariant::MidiMergerNode(MidiMergerNode::new(config))),
        "MidiTransposeNode" => Ok(NodeVariant::MidiTransposeNode(MidiTransposeNode::new(config))),
        "WavetableSequencerNode" => Ok(NodeVariant::WavetableSequencerNode(WavetableSequencerNode::new(config))),
        "MemoryNode" => Ok(MemoryNode::new(config).into()),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}

pub fn variant_io(
    node_type: &str,
    props: HashMap<String, Property>,
    register: &mut dyn FnMut(&str) -> u32,
) -> Result<NodeIo, NodeError> {
    match node_type {
        "OutputNode" => Ok(OutputNode::get_io(props, register)),
        "MidiInNode" => Ok(MidiInNode::get_io(props, register)),
        "GainNode" => Ok(GainNode::get_io(props, register)),
        "OscillatorNode" => Ok(OscillatorNode::get_io(props, register)),
        "MidiToValuesNode" => Ok(MidiToValuesNode::get_io(props, register)),
        "EnvelopeNode" => Ok(EnvelopeNode::get_io(props, register)),
        "BiquadFilterNode" => Ok(BiquadFilterNode::get_io(props, register)),
        "MixerNode" => Ok(MixerNode::get_io(props, register)),
        "ExpressionNode" => Ok(ExpressionNode::get_io(props, register)),
        "DummyNode" => Ok(DummyNode::get_io(props, register)),
        "FunctionNode" => Ok(FunctionNode::get_io(props, register)),
        "InputsNode" => Ok(InputsNode::get_io(props, register)),
        "OutputsNode" => Ok(OutputsNode::get_io(props, register)),
        "StreamExpressionNode" => Ok(StreamExpressionNode::get_io(props, register)),
        "PolyphonicNode" => Ok(PolyphonicNode::get_io(props, register)),
        "MidiFilterNode" => Ok(MidiFilterNode::get_io(props, register)),
        "WavetableNode" => Ok(WavetableNode::get_io(props, register)),
        "PortamentoNode" => Ok(PortamentoNode::get_io(props, register)),
        "ButtonNode" => Ok(ButtonNode::get_io(props, register)),
        "RankPlayerNode" => Ok(RankPlayerNode::get_io(props, register)),
        "MidiMergerNode" => Ok(MidiMergerNode::get_io(props, register)),
        "MidiTransposeNode" => Ok(MidiTransposeNode::get_io(props, register)),
        "WavetableSequencerNode" => Ok(WavetableSequencerNode::get_io(props, register)),
        "MemoryNode" => Ok(MemoryNode::get_io(props, register)),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}
