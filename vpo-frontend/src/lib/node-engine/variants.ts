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
        category: "midi"
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
        internal: "ButtonNode",
        category: "ui"
    },
    {
        internal: "RankPlayerNode",
        category: "audio"
    },
    {
        internal: "MidiMergerNode",
        category: "midi"
    },
    {
        internal: "MidiTransposeNode",
        category: "midi"
    }
];