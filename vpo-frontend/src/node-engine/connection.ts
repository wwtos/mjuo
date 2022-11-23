import { deepEqual } from "fast-equals";
import { DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";
import { NodeIndex } from "./node_index";


export type MidiSocketType = DiscriminatedUnion<"variant", {
    Default: {},
    Dynamic: { data: number }
}>;

export type StreamSocketType = DiscriminatedUnion<"variant", {
    Audio: {},
    Gate: {},
    Gain: {},
    Detune: {},
    Dynamic: { data: number },
}>;

export type ValueSocketType = DiscriminatedUnion<"variant", {
    Default: {},
    Gain: {},
    Frequency: {},
    Resonance: {},
    Gate: {},
    Attack: {},
    Decay: {},
    Sustain: {},
    Release: {},
    Speed: {},
    State: {},
    UiState: {},
    Dynamic: { data: number },
}>;

export type NodeRefSocketType = DiscriminatedUnion<"variant", {
    Button: {},
    Dynamic: { data: number },
}>;

export type Primitive = DiscriminatedUnion<"variant", {
    Float: { data: number },
    Int: { data: number },
    Boolean: { data: boolean },
    String: { data: string },
}>;

export type SocketType = DiscriminatedUnion<"variant", {
    Stream: { data: StreamSocketType },
    Midi: { data: MidiSocketType },
    Value: { data: ValueSocketType },
    NodeRef: { data: NodeRefSocketType },
    MethodCall: { data: Primitive[] },
}>;

export const SocketType = {
    toKey(socketType: SocketType) {
        return socketType.variant + "," + match(socketType, {
            Stream: ({ data: stream }) => matchOrElse(
                stream, {
                    Dynamic: ({ data }) => stream.variant + data,
                },  () => stream.variant
            ),
            Midi: ({ data: midi }) => matchOrElse(
                midi, {
                    Dynamic: ({ data }) => midi.variant + data,
                },  () => midi.variant
            ),
            Value: ({ data: value }) => matchOrElse(
                value, {
                    Dynamic: ({ data }) => value.variant + data,
                },  () => value.variant
            ),
            NodeRef:  ({ data: nodeRef }) => matchOrElse(
                nodeRef, {
                    Dynamic: ({ data }) => nodeRef.variant + data,
                },  () => nodeRef.variant
            ),
            MethodCall: args => args.toString()
        });
    },
    areEqual(socketType1: SocketType, socketType2: SocketType): boolean {
        return deepEqual(socketType1, socketType2);
    },
}

export function socketToKey(socket: SocketType, direction: SocketDirection) {
    return SocketType.toKey(socket) + ":" + direction + match(socket, {
        Stream: ({ data: stream }) => matchOrElse(stream, {
            Dynamic: ({ data: uid }) => ":" + uid,
        },  () => "_"),
        Midi: ({ data: midi }) => matchOrElse(midi, {
            Dynamic: ({ data: uid }) => ":" + uid,
        },  () => "_"),
        Value: ({ data: value }) => matchOrElse(value, {
            Dynamic: ({ data: uid }) => ":" + uid,
        },  () => "_"),
        NodeRef: ({ data: nodeRef }) => matchOrElse(nodeRef, {
            Dynamic: ({ data: uid }) => ":" + uid,
        },  () => "_"),
        MethodCall: () => ""
    });
}

export enum SocketDirection {
    Input = 0,
    Output = 1
};

export interface Connection {
    from_socket_type: SocketType;
    from_node: NodeIndex;
    to_socket_type: SocketType;
    to_node: NodeIndex;
}

export const Connection = {
    getKey(connection: Connection): string {
        return SocketType.toKey(connection.from_socket_type) + ":" +
            NodeIndex.toKey(connection.from_node) + "->" +
            SocketType.toKey(connection.to_socket_type) + ":" +
            NodeIndex.toKey(connection.to_node);
    }
}

export interface InputSideConnection {
    from_socket_type: SocketType;
    from_node: NodeIndex;
    to_socket_type: SocketType;
}

export interface OutputSideConnection {
    from_socket_type: SocketType;
    to_node: NodeIndex;
    to_socket_type: SocketType;
}
