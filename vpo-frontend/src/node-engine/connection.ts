import type { NodeIndex } from "./node";
import { socketRegistry } from "../node-editor/state";
import { i18n } from '../i18n';
import { deepEqual } from "fast-equals";
import { makeTaggedUnion, MemberType, none } from "safety-match";
import { DiscriminatedUnion, match, matchOrElse } from "../util/discriminated-union";


export type MidiSocketType = DiscriminatedUnion<"variant", {
    Default: {},
    Dynamic: { content: number }
}>;

export type StreamSocketType = DiscriminatedUnion<"variant", {
    Audio: {},
    Gate: {},
    Gain: {},
    Detune: {},
    Dynamic: { content: number },
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
    Dynamic: { content: number },
}>;

export type NodeRefSocketType = DiscriminatedUnion<"variant", {
    Button: {},
    Dynamic: { content: number },
}>;

export type Primitive = DiscriminatedUnion<"variant", {
    Float: { content: number },
    Int: { content: number },
    Boolean: { content: boolean },
    String: { content: string },
}>;

export type SocketType = DiscriminatedUnion<"variant", {
    Stream: { content: StreamSocketType },
    Midi: { content: MidiSocketType },
    Value: { content: ValueSocketType },
    NodeRef: { content: NodeRefSocketType },
    MethodCall: { content: Primitive[] },
}>;

export const SocketType = {
    toKey(socketType: SocketType) {
        return socketType.variant + "," + match(socketType, {
            Stream: ({ content: stream }) => matchOrElse(
                stream, {
                    Dynamic: ({ content }) => stream.variant + content,
                },  () => stream.variant
            ),
            Midi: ({ content: midi }) => matchOrElse(
                midi, {
                    Dynamic: ({ content }) => midi.variant + content,
                },  () => midi.variant
            ),
            Value: ({ content: value }) => matchOrElse(
                value, {
                    Dynamic: ({ content }) => value.variant + content,
                },  () => value.variant
            ),
            NodeRef:  ({ content: nodeRef }) => matchOrElse(
                nodeRef, {
                    Dynamic: ({ content }) => nodeRef.variant + content,
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
        Stream: ({ content: stream }) => matchOrElse(stream, {
            Dynamic: ({ content: uid }) => ":" + uid,
        },  () => "_"),
        Midi: ({ content: midi }) => matchOrElse(midi, {
            Dynamic: ({ content: uid }) => ":" + uid,
        },  () => "_"),
        Value: ({ content: value }) => matchOrElse(value, {
            Dynamic: ({ content: uid }) => ":" + uid,
        },  () => "_"),
        NodeRef: ({ content: nodeRef }) => matchOrElse(nodeRef, {
            Dynamic: ({ content: uid }) => ":" + uid,
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
