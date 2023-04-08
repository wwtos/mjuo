import type { VertexIndex } from "../ddgg/graph";
import { match, type DiscriminatedUnion } from "../util/discriminated-union";
import type { NodeConnection } from "./node_graph";
import type { MidiData } from "$lib/sound-engine/midi/messages";

export interface Connection {
    fromNode: VertexIndex;
    toNode: VertexIndex;
    data: NodeConnection;
}

export interface InputSideConnection {
    fromSocket: Socket;
    fromNode: VertexIndex;
    toSocket: Socket;
}

export interface OutputSideConnection {
    fromSocket: Socket;
    toNode: VertexIndex;
    toSocket: Socket;
}

export type Socket = DiscriminatedUnion<"variant", {
    Simple: { data: [number, SocketType, number] },
    Numbered: { data: [number, number, SocketType, number] }
}>;

export const Socket = {
    socketType(socket: Socket) {
        return match(socket, {
            Simple: ({data: [_, socket_type]}) => socket_type,
            Numbered: ({data: [_, __, socket_type]}) => socket_type,
        });
    }
};

export type SocketType = DiscriminatedUnion<"variant", {
    Stream: {},
    Midi: {},
    Value: {},
    NodeRef: {},
}>;

export type SocketDirection = DiscriminatedUnion<"variant", {
    Input: {},
    Output: {},
}>;

export type Primitive = DiscriminatedUnion<"variant", {
    Float: { data: number },
    Int: { data: number },
    Boolean: { data: boolean },
    String: { data: string },
}>;

export type SocketValue = DiscriminatedUnion<"variant", {
    Stream: { data: number },
    Value: { data: Primitive },
    Midi: { data: MidiData[] },
    None: {}
}>;

