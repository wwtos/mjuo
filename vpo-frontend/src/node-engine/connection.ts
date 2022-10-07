import type { NodeIndex } from "./node";
import { socketRegistry } from "../node-editor/state";
import { i18n } from '../i18n';
import { deepEqual } from "fast-equals";
import { makeTaggedUnion, MemberType, none } from "safety-match";
import { DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";


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

export class Connection {
    fromSocketType: SocketType;
    fromNode: NodeIndex;
    toSocketType: SocketType;
    toNode: NodeIndex;

    constructor(fromSocketType: SocketType, fromNode: NodeIndex, toSocketType: SocketType, toNode: NodeIndex) {
        this.fromSocketType = fromSocketType;
        this.fromNode = fromNode;
        this.toSocketType = toSocketType;
        this.toNode = toNode;
    }

    toJSON() {
        return {
            from_socket_type: this.fromSocketType,
            from_node: this.fromNode,
            to_socket_type: this.toSocketType,
            to_node: this.toNode
        }
    }

    getKey(): string {
        return SocketType.toKey(this.fromSocketType) + ":" +
            this.fromNode.toKey() + "->" +
            SocketType.toKey(this.toSocketType) + ":" +
            this.toNode.toKey();
    }
}

export class InputSideConnection {
    fromSocketType: SocketType;
    fromNode: NodeIndex;
    toSocketType: SocketType;

    constructor(fromSocketType: SocketType, fromNode: NodeIndex, toSocketType: SocketType) {
        this.fromSocketType = fromSocketType;
        this.fromNode = fromNode;
        this.toSocketType = toSocketType;
    }
}

export class OutputSideConnection {
    fromSocketType: SocketType;
    toNode: NodeIndex;
    toSocketType: SocketType;

    constructor(fromSocketType: SocketType, toNode: NodeIndex, toSocketType: SocketType) {
        this.fromSocketType = fromSocketType;
        this.toNode = toNode;
        this.toSocketType = toSocketType;
    }
}
