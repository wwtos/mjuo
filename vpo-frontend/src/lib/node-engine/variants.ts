export interface NodeVariant {
    internal: string,
    category: string
}

export const variants: NodeVariant[] = [
    {
        internal: "GainNode",
        category: "audio"
    },
    {
        internal: "OscillatorNode",
        category: "audio"
    },
    {
        internal: "MidiToValuesNode",
        category: "midi"
    },
    {
        internal: "EnvelopeNode",
        category: "base"
    },
    {
        internal: "BiquadFilterNode",
        category: "audio"
    },
    {
        internal: "MixerNode",
        category: "audio"
    },
    {
        internal: "ExpressionNode",
        category: "scripting"
    },
    {
        internal: "FunctionNode",
        category: "base"
    },
    {
        internal: "StreamExpressionNode",
        category: "scripting"
    },
    {
        internal: "PolyphonicNode",
        category: "base"
    },
    {
        internal: "MidiFilterNode",
        category: "scripting"
    },
    {
        internal: "WavetableNode",
        category: "audio"
    },
    {
        internal: "PortamentoNode",
        category: "base"
    },
    {
        internal: "ToggleNode",
        category: "ui"
    },
    {
        internal: "RankPlayerNode",
        category: "audio"
    },
    {
        internal: "NoteMergerNode",
        category: "midi"
    },
    {
        internal: "MidiTransposeNode",
        category: "midi"
    },
    {
        internal: "WavetableSequencerNode",
        category: "base"
    },
    {
        internal: "MemoryNode",
        category: "base"
    },
    {
        internal: "MidiSwitchNode",
        category: "midi"
    },
    {
        internal: "MidiToValueNode",
        category: "scripting"
    },
    {
        internal: "UpDownMixerNode",
        category: "base"
    },
    {
        internal: "InputsNode",
        category: "base"
    },
    {
        internal: "OutputsNode",
        category: "base"
    },
    {
        internal: "ReverbNode",
        category: "audio"
    }
];