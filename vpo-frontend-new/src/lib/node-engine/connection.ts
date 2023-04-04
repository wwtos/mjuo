import { deepEqual } from "fast-equals";
import { Index } from "../ddgg/gen_vec";
import type { VertexIndex } from "../ddgg/graph";
import { type DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";
import type { NodeConnection } from "./node_graph";


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
    });
}

export enum SocketDirection {
    Input = 0,
    Output = 1
};

export interface Connection {
    fromNode: VertexIndex;
    toNode: VertexIndex;
    data: NodeConnection;
}

export const Connection = {
    getKey(connection: Connection): string {
        return SocketType.toKey(connection.data.fromSocketType) + ":" +
            Index.toKey(connection.fromNode) + "->" +
            SocketType.toKey(connection.data.toSocketType) + ":" +
            Index.toKey(connection.toNode);
    }
}

export interface InputSideConnection {
    fromSocketType: SocketType;
    fromNode: VertexIndex;
    toSocketType: SocketType;
}

export interface OutputSideConnection {
    fromSocketType: SocketType;
    toNode: VertexIndex;
    toSocketType: SocketType;
}
