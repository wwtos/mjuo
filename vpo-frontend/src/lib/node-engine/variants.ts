export interface NodeVariant {
    translationKey: string,
    internal: string,
    category: string
}

export const variants: NodeVariant[] = [
    {
        translationKey: "node-gain",
        internal: "GainGraphNode",
        category: "audio"
    },
    {
        translationKey: "node-oscillator",
        internal: "OscillatorNode",
        category: "audio"
    },
    {
        translationKey: "node-midi-to-values",
        internal: "MidiToValuesNode",
        category: "midi"
    },
    {
        translationKey: "node-envelope",
        internal: "EnvelopeNode",
        category: "base"
    },
    {
        translationKey: "node-filter-biquad",
        internal: "BiquadFilterNode",
        category: "audio"
    },
    {
        translationKey: "node-mixer",
        internal: "MixerNode",
        category: "audio"
    },
    {
        translationKey: "node-expression",
        internal: "ExpressionNode",
        category: "scripting"
    },
    {
        translationKey: "node-function",
        internal: "FunctionNode",
        category: "base"
    },
    {
        translationKey: "node-expression-stream",
        internal: "StreamExpressionNode",
        category: "scripting"
    },
    {
        translationKey: "node-polyphonic",
        internal: "PolyphonicNode",
        category: "base"
    },
    {
        translationKey: "node-filter-midi",
        internal: "MidiFilterNode",
        category: "midi"
    },
    {
        translationKey: "node-pipe-player",
        internal: "PipePlayerNode",
        category: "audio"
    },
    {
        translationKey: "node-wavetable-oscillator",
        internal: "WavetableNode",
        category: "audio"
    },
    {
        translationKey: "node-portamento",
        internal: "PortamentoNode",
        category: "base"
    },
    {
        translationKey: "node-button",
        internal: "ButtonNode",
        category: "ui"
    },
    {
        translationKey: "node-rank-player",
        internal: "RankPlayerNode",
        category: "audio"
    }
];