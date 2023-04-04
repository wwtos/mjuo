export interface NodeVariant {
    name: string,
    internal: string,
    category: string
}

export const variants: NodeVariant[] = [
    {
        name: i18n.t("nodes.GainGraphNode"),
        internal: "GainGraphNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.OscillatorNode"),
        internal: "OscillatorNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.MidiToValuesNode"),
        internal: "MidiToValuesNode",
        category: "midi"
    },
    {
        name: i18n.t("nodes.EnvelopeNode"),
        internal: "EnvelopeNode",
        category: "base"
    },
    {
        name: i18n.t("nodes.BiquadFilterNode"),
        internal: "BiquadFilterNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.MixerNode"),
        internal: "MixerNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.ExpressionNode"),
        internal: "ExpressionNode",
        category: "scripting"
    },
    {
        name: i18n.t("nodes.FunctionNode"),
        internal: "FunctionNode",
        category: "base"
    },
    {
        name: i18n.t("nodes.StreamExpressionNode"),
        internal: "StreamExpressionNode",
        category: "scripting"
    },
    {
        name: i18n.t("nodes.PolyphonicNode"),
        internal: "PolyphonicNode",
        category: "base"
    },
    {
        name: i18n.t("nodes.MidiFilterNode"),
        internal: "MidiFilterNode",
        category: "midi"
    },
    {
        name: i18n.t("nodes.MonoSamplePlayerNode"),
        internal: "MonoSamplePlayerNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.WavetableNode"),
        internal: "WavetableNode",
        category: "audio"
    },
    {
        name: i18n.t("nodes.PortamentoNode"),
        internal: "PortamentoNode",
        category: "base"
    },
    {
        name: i18n.t("nodes.ButtonNode"),
        internal: "ButtonNode",
        category: "ui"
    },
    {
        name: i18n.t("nodes.RankPlayer"),
        internal: "RankPlayerNode",
        category: "audio"
    }
];