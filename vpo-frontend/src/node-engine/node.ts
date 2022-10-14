import { InputSideConnection, MidiSocketType, NodeRefSocketType, OutputSideConnection,
         Primitive, SocketDirection, SocketType, StreamSocketType, ValueSocketType } from "./connection";
import type { Property, PropertyType } from "./property";
import { MidiData } from "../sound-engine/midi/messages";
import { DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";

const TITLE_HEIGHT = 30;
const SOCKET_HEIGHT = 36;
const SOCKET_OFFSET = 26;
const NODE_WIDTH = 200;


export type NodeRow = DiscriminatedUnion<"variant", {
    StreamInput: { data: [StreamSocketType, number] },
    MidiInput: { data: [MidiSocketType, MidiData[]] },
    ValueInput: { data: [ValueSocketType, Primitive] },
    NodeRefInput: { data: NodeRefSocketType },
    StreamOutput: { data: [StreamSocketType, number] },
    MidiOutput: { data: [MidiSocketType, MidiData[]] },
    ValueOutput: { data: [ValueSocketType, Primitive] },
    NodeRefOutput: { data: NodeRefSocketType },
    Property: { data: [string, PropertyType, Property] },
    InnerGraph: { data: undefined },
}>;

type SocketTypeAndDirection = {socketType: SocketType, direction: SocketDirection};

export const NodeRow = {
    getTypeAndDirection: (
        nodeRow: NodeRow
    ): {socketType: SocketType, direction: SocketDirection} | undefined => {
        return matchOrElse(nodeRow, {
            StreamInput: ({ data: [type, _] }): SocketTypeAndDirection => ({
                socketType: { variant: "Stream", data: type },
                direction: SocketDirection.Input
            }),
            MidiInput: ({ data: [type, _] }) => ({
                socketType: { variant: "Midi", data: type },
                direction: SocketDirection.Input
            }),
            ValueInput: ({ data: [type, _] }) => ({
                socketType: { variant: "Value", data: type },
                direction: SocketDirection.Input
            }),
            NodeRefInput: ({ data: type }): SocketTypeAndDirection => ({
                socketType: { variant: "NodeRef", data: type },
                direction: SocketDirection.Input
            }),
            StreamOutput: ({ data: [type, _] }): SocketTypeAndDirection => ({
                socketType: { variant: "Stream", data: type },
                direction: SocketDirection.Output
            }),
            MidiOutput: ({ data: [type, _] }) => ({
                socketType: { variant: "Midi", data: type },
                direction: SocketDirection.Output
            }),
            ValueOutput: ({ data: [type, _] }) => ({
                socketType: { variant: "Value", data: type },
                direction: SocketDirection.Output
            }),
            NodeRefOutput: ({ data: type }): SocketTypeAndDirection => ({
                socketType: { variant: "NodeRef", data: type },
                direction: SocketDirection.Output
            }),
        },  () => undefined);
    },
    fromTypeAndDirection: (
        type: SocketType,
        direction: SocketDirection,
        defaultValue: any
    ): NodeRow => {
        if (direction === SocketDirection.Input) {
            return match(type, {
                Stream: ({ data: streamSocketType }): NodeRow => ({
                    variant: "StreamInput",
                    data: [streamSocketType, defaultValue]
                }),
                Midi: ({ data: midiSocketType }): NodeRow => ({
                    variant: "MidiInput",
                    data: [midiSocketType, defaultValue]
                }),
                Value: ({ data: valueSocketType }): NodeRow => ({
                    variant: "ValueInput", 
                    data: [valueSocketType, defaultValue]
                }),
                NodeRef: ({ data: nodeRefSocketType }): NodeRow => ({
                    variant: "NodeRefInput",
                    data: nodeRefSocketType
                }),
                MethodCall: (_params) => {
                    throw new Error("why do I still have this")
                }
            });
        } else {
            return match(type, {
                Stream: ({ data: streamSocketType }): NodeRow => ({
                    variant: "StreamInput",
                    data: [streamSocketType, defaultValue]
                }),
                Midi: ({ data: midiSocketType }): NodeRow => ({
                    variant: "MidiInput",
                    data: [midiSocketType, defaultValue]
                }),
                Value: ({ data: valueSocketType }): NodeRow => ({
                    variant: "ValueInput", 
                    data: [valueSocketType, defaultValue]
                }),
                NodeRef: ({ data: nodeRefSocketType }): NodeRow => ({
                    variant: "NodeRefInput",
                    data: nodeRefSocketType
                }),
                MethodCall: (_params) => {
                    throw new Error("why do I still have this")
                }
            });
        }
    },
    getDefault(nodeRow: NodeRow): SocketValue {
        return matchOrElse(nodeRow, {
            StreamInput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Stream", data: defaultValue }),
            MidiInput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Midi", data: defaultValue }),
            ValueInput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Primitive", data: defaultValue }),
            NodeRefInput: ({ data: _ }): SocketValue => ({ variant: "None" }),
            StreamOutput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Stream", data: defaultValue }),
            MidiOutput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Midi", data: defaultValue }),
            ValueOutput: ({ data: [_, defaultValue ] }): SocketValue => ({ variant: "Primitive", data: defaultValue }),
            NodeRefOutput: ({ data: _ }): SocketValue => ({ variant: "None" })
        },  () => ({ variant: "None" }));
    }
};


export type SocketValue = DiscriminatedUnion<"variant", {
    Stream: { data: number },
    Primitive: { data: Primitive },
    Midi: { data: MidiData[] },
    None: {}
}>;

export class InitResult {
    did_rows_change: boolean;
    /* Vec<NodeRow> */
    node_rows: Array <NodeRow> ;
    /* Option<HashMap<String, Property>> */
    changed_properties ? : {
        [key: string]: Property
    }

    constructor(obj) {
        for (var i in obj) {
            this[i] = obj[i];
        }
    }
}

export interface UiData {
    x: number;
    y: number;
    selected?: boolean;
    title?: string;
}

export interface Node {
    inputSockets: SocketType[];
    outputSockets: SocketType[];
    usableProperties: {
        [prop: string]: PropertyType
    };
}

export interface NodeWrapper {
    node: Node;
    index: NodeIndex;
    connected_inputs: InputSideConnection[];
    connected_outputs: OutputSideConnection[];
    node_rows: NodeRow[];
    default_overrides: NodeRow[];
    properties: { [key: string]: Property };
    ui_data: UiData;
    child_graph_index: number | null;
}

export interface NodeIndex {
    index: number;
    generation: number;
}

export const NodeIndex = {
    toKey(index: NodeIndex): string {
        return index.index + "," + index.generation;
    },
    toString(index: NodeIndex): string {
        return `NodeIndex { index: ${index.index}, generation: ${index.generation} }`;
    }
};

export interface GenerationalNode {
    node: NodeWrapper;
    generation: number;
}