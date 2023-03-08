import { InputSideConnection, MidiSocketType, NodeRefSocketType, OutputSideConnection,
         Primitive, SocketDirection, SocketType, StreamSocketType, ValueSocketType } from "./connection";
import type { Property, PropertyType } from "./property";
import type { MidiData } from "../sound-engine/midi/messages";
import { type DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";
import type { Index } from "../ddgg/gen_vec";

export const TITLE_HEIGHT = 30;
export const SOCKET_HEIGHT = 36;
export const SOCKET_OFFSET = 26;
export const NODE_WIDTH = 270;


export type NodeRow = DiscriminatedUnion<"variant", {
    StreamInput: { data: [StreamSocketType, number, boolean] },
    MidiInput: { data: [MidiSocketType, MidiData[], boolean] },
    ValueInput: { data: [ValueSocketType, Primitive, boolean] },
    NodeRefInput: { data: [NodeRefSocketType, boolean] },
    StreamOutput: { data: [StreamSocketType, number, boolean] },
    MidiOutput: { data: [MidiSocketType, MidiData[], boolean] },
    ValueOutput: { data: [ValueSocketType, Primitive, boolean] },
    NodeRefOutput: { data: [NodeRefSocketType, boolean] },
    Property: { data: [string, PropertyType, Property] },
    InnerGraph: { data: undefined },
}>;

type SocketTypeAndDirection = {socketType: SocketType, direction: SocketDirection, polyphonic: boolean};

export const NodeRow = {
    getTypeAndDirection: (
        nodeRow: NodeRow
    ): SocketTypeAndDirection | undefined => {
        return matchOrElse(nodeRow, {
            StreamInput: ({ data: [type, _, polyphonic] }): SocketTypeAndDirection => ({
                socketType: { variant: "Stream", data: type },
                direction: SocketDirection.Input,
                polyphonic
            }),
            MidiInput: ({ data: [type, _, polyphonic] }) => ({
                socketType: { variant: "Midi", data: type },
                direction: SocketDirection.Input,
                polyphonic
            }),
            ValueInput: ({ data: [type, _, polyphonic] }) => ({
                socketType: { variant: "Value", data: type },
                direction: SocketDirection.Input,
                polyphonic
            }),
            NodeRefInput: ({ data: [type, polyphonic] }) => ({
                socketType: { variant: "NodeRef", data: type },
                direction: SocketDirection.Input,
                polyphonic
            }),
            StreamOutput: ({ data: [type, _, polyphonic] }) => ({
                socketType: { variant: "Stream", data: type },
                direction: SocketDirection.Output,
                polyphonic
            }),
            MidiOutput: ({ data: [type, _, polyphonic] }) => ({
                socketType: { variant: "Midi", data: type },
                direction: SocketDirection.Output,
                polyphonic
            }),
            ValueOutput: ({ data: [type, _, polyphonic] }) => ({
                socketType: { variant: "Value", data: type },
                direction: SocketDirection.Output,
                polyphonic
            }),
            NodeRefOutput: ({ data: [type, polyphonic] }) => ({
                socketType: { variant: "NodeRef", data: type },
                direction: SocketDirection.Output,
                polyphonic
            }),
        },  () => undefined);
    },
    fromTypeAndDirection: (
        type: SocketType,
        direction: SocketDirection,
        defaultValue: any,
        polyphonic: boolean,
    ): NodeRow => {
        if (direction === SocketDirection.Input) {
            return match(type, {
                Stream: ({ data: streamSocketType }): NodeRow => ({
                    variant: "StreamInput",
                    data: [streamSocketType, defaultValue, polyphonic]
                }),
                Midi: ({ data: midiSocketType }): NodeRow => ({
                    variant: "MidiInput",
                    data: [midiSocketType, defaultValue, polyphonic]
                }),
                Value: ({ data: valueSocketType }): NodeRow => ({
                    variant: "ValueInput", 
                    data: [valueSocketType, defaultValue, polyphonic]
                }),
                NodeRef: ({ data: nodeRefSocketType }): NodeRow => ({
                    variant: "NodeRefInput",
                    data: [nodeRefSocketType, polyphonic]
                }),
                MethodCall: (_params) => {
                    throw new Error("why do I still have this")
                }
            });
        } else {
            return match(type, {
                Stream: ({ data: streamSocketType }): NodeRow => ({
                    variant: "StreamInput",
                    data: [streamSocketType, defaultValue, polyphonic]
                }),
                Midi: ({ data: midiSocketType }): NodeRow => ({
                    variant: "MidiInput",
                    data: [midiSocketType, defaultValue, polyphonic]
                }),
                Value: ({ data: valueSocketType }): NodeRow => ({
                    variant: "ValueInput", 
                    data: [valueSocketType, defaultValue, polyphonic]
                }),
                NodeRef: ({ data: nodeRefSocketType }): NodeRow => ({
                    variant: "NodeRefInput",
                    data: [nodeRefSocketType, polyphonic]
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
    },
    getHeight(nodeRow: NodeRow): number {
        return SOCKET_HEIGHT;
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
    connectedInputs: InputSideConnection[];
    connectedOutputs: OutputSideConnection[];
    nodeRows: NodeRow[];
    defaultOverrides: NodeRow[];
    properties: { [key: string]: Property };
    uiData: UiData;
    childGraphIndex: Index | null;
}
export interface GenerationalNode {
    node: NodeWrapper;
    generation: number;
}