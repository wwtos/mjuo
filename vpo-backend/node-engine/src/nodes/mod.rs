use enum_dispatch::enum_dispatch;

pub mod biquad_filter;
pub mod dummy;
pub mod envelope;
pub mod expression;
pub mod function_node;
pub mod gain;
pub mod inputs;
pub mod memory;
pub mod midi_filter;
pub mod midi_input;
pub mod midi_switch;
pub mod midi_to_value;
pub mod midi_to_values;
pub mod midi_transpose;
pub mod mixer;
pub mod note_merger;
pub mod oscillator;
pub mod output;
pub mod outputs;
pub mod polyphonic;
pub mod portamento;
pub mod prelude;
pub mod rank_player;
pub mod stream_expression;
pub mod toggle;
pub mod util;
pub mod wavetable;
pub mod wavetable_sequencer;

use self::midi_to_value::MidiToValueNode;
use self::{
    biquad_filter::BiquadFilterNode, dummy::DummyNode, envelope::EnvelopeNode, expression::ExpressionNode,
    function_node::FunctionNode, gain::GainNode, inputs::InputsNode, memory::MemoryNode, midi_filter::MidiFilterNode,
    midi_input::MidiInNode, midi_switch::MidiSwitchNode, midi_to_values::MidiToValuesNode,
    midi_transpose::MidiTransposeNode, mixer::MixerNode, note_merger::NoteMergerNode, oscillator::OscillatorNode,
    output::OutputNode, outputs::OutputsNode, polyphonic::PolyphonicNode, portamento::PortamentoNode,
    rank_player::RankPlayerNode, stream_expression::StreamExpressionNode, toggle::ToggleNode, wavetable::WavetableNode,
    wavetable_sequencer::WavetableSequencerNode,
};

use self::prelude::*;

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
    ToggleNode,
    RankPlayerNode,
    NoteMergerNode,
    MidiTransposeNode,
    WavetableSequencerNode,
    MemoryNode,
    MidiSwitchNode,
    MidiToValueNode,
}

impl Default for NodeVariant {
    fn default() -> Self {
        NodeVariant::DummyNode(DummyNode::default())
    }
}

pub fn new_variant(node_type: &str, config: &SoundConfig) -> Result<NodeVariant, NodeError> {
    match node_type {
        "OutputNode" => Ok(OutputNode::new(config).into()),
        "MidiInNode" => Ok(MidiInNode::new(config).into()),
        "GainNode" => Ok(GainNode::new(config).into()),
        "OscillatorNode" => Ok(OscillatorNode::new(config).into()),
        "MidiToValuesNode" => Ok(MidiToValuesNode::new(config).into()),
        "EnvelopeNode" => Ok(EnvelopeNode::new(config).into()),
        "BiquadFilterNode" => Ok(BiquadFilterNode::new(config).into()),
        "MixerNode" => Ok(MixerNode::new(config).into()),
        "ExpressionNode" => Ok(ExpressionNode::new(config).into()),
        "DummyNode" => Ok(DummyNode::new(config).into()),
        "FunctionNode" => Ok(FunctionNode::new(config).into()),
        "InputsNode" => Ok(InputsNode::new(config).into()),
        "OutputsNode" => Ok(OutputsNode::new(config).into()),
        "StreamExpressionNode" => Ok(StreamExpressionNode::new(config).into()),
        "PolyphonicNode" => Ok(PolyphonicNode::new(config).into()),
        "MidiFilterNode" => Ok(MidiFilterNode::new(config).into()),
        "WavetableNode" => Ok(WavetableNode::new(config).into()),
        "PortamentoNode" => Ok(PortamentoNode::new(config).into()),
        "ToggleNode" => Ok(ToggleNode::new(config).into()),
        "RankPlayerNode" => Ok(RankPlayerNode::new(config).into()),
        "NoteMergerNode" => Ok(NoteMergerNode::new(config).into()),
        "MidiTransposeNode" => Ok(MidiTransposeNode::new(config).into()),
        "WavetableSequencerNode" => Ok(WavetableSequencerNode::new(config).into()),
        "MemoryNode" => Ok(MemoryNode::new(config).into()),
        "MidiSwitchNode" => Ok(MidiSwitchNode::new(config).into()),
        "MidiToValueNode" => Ok(MidiToValueNode::new(config).into()),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}

pub fn variant_io(node_type: &str, props: HashMap<String, Property>) -> Result<NodeIo, NodeError> {
    match node_type {
        "OutputNode" => Ok(OutputNode::get_io(props)),
        "MidiInNode" => Ok(MidiInNode::get_io(props)),
        "GainNode" => Ok(GainNode::get_io(props)),
        "OscillatorNode" => Ok(OscillatorNode::get_io(props)),
        "MidiToValuesNode" => Ok(MidiToValuesNode::get_io(props)),
        "EnvelopeNode" => Ok(EnvelopeNode::get_io(props)),
        "BiquadFilterNode" => Ok(BiquadFilterNode::get_io(props)),
        "MixerNode" => Ok(MixerNode::get_io(props)),
        "ExpressionNode" => Ok(ExpressionNode::get_io(props)),
        "DummyNode" => Ok(DummyNode::get_io(props)),
        "FunctionNode" => Ok(FunctionNode::get_io(props)),
        "InputsNode" => Ok(InputsNode::get_io(props)),
        "OutputsNode" => Ok(OutputsNode::get_io(props)),
        "StreamExpressionNode" => Ok(StreamExpressionNode::get_io(props)),
        "PolyphonicNode" => Ok(PolyphonicNode::get_io(props)),
        "MidiFilterNode" => Ok(MidiFilterNode::get_io(props)),
        "WavetableNode" => Ok(WavetableNode::get_io(props)),
        "PortamentoNode" => Ok(PortamentoNode::get_io(props)),
        "ToggleNode" => Ok(ToggleNode::get_io(props)),
        "RankPlayerNode" => Ok(RankPlayerNode::get_io(props)),
        "NoteMergerNode" => Ok(NoteMergerNode::get_io(props)),
        "MidiTransposeNode" => Ok(MidiTransposeNode::get_io(props)),
        "WavetableSequencerNode" => Ok(WavetableSequencerNode::get_io(props)),
        "MemoryNode" => Ok(MemoryNode::get_io(props)),
        "MidiSwitchNode" => Ok(MidiSwitchNode::get_io(props)),
        "MidiToValueNode" => Ok(MidiToValueNode::get_io(props)),
        _ => Err(NodeError::NodeTypeDoesNotExist),
    }
}
