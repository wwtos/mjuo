import type { NodeIndex } from "./node";
import { socketRegistry } from "../node-editor/state";
import { i18n } from '../i18n';
import { deepEqual } from "fast-equals";
import {makeTaggedUnion, MemberType, none} from "safety-match";

export const MidiSocketType = makeTaggedUnion({
    Default: none,
    Dynamic: (uid) => uid,
});

export function deserializeMidiSocketType(json: any): MemberType<typeof MidiSocketType> {
    switch (json.variant) {
        case "Default":
            return MidiSocketType.Default;
        case "Dynamic":
            return MidiSocketType.Dynamic(json.data);
    }

    throw "Failed to parse json";
};

export const StreamSocketType = makeTaggedUnion({
    Audio: none,
    Gate: none,
    Gain: none,
    Detune: none,
    Dynamic: (uid: number) => uid,
});

export function deserializeStreamSocketType(json: any): MemberType<typeof StreamSocketType> {
    switch (json.variant) {
        case "Audio":
            return StreamSocketType.Audio;
        case "Gate":
            return StreamSocketType.Gate;
        case "Gain":
            return StreamSocketType.Gain;
        case "Detune":
            return StreamSocketType.Detune;
        case "Dynamic":
            return StreamSocketType.Dynamic(json.data);
    }

    throw "Failed to parse json";
};

export const ValueSocketType = makeTaggedUnion({
    Default: none,
    Gain: none,
    Frequency: none,
    Resonance: none,
    Gate: none,
    Attack: none,
    Decay: none,
    Sustain: none,
    Release: none,
    Dynamic: (uid: number) => uid,
});

export function deserializeValueSocketType(json: any): MemberType<typeof ValueSocketType> {
    switch (json.variant) {
        case "Default":
            return ValueSocketType.Default;
        case "Gain":
            return ValueSocketType.Gain;
        case "Frequency":
            return ValueSocketType.Frequency;
        case "Resonance":
            return ValueSocketType.Resonance;
        case "Gate":
            return ValueSocketType.Gate;
        case "Attack":
            return ValueSocketType.Attack;
        case "Decay":
            return ValueSocketType.Decay;
        case "Sustain":
            return ValueSocketType.Sustain;
        case "Release":
            return ValueSocketType.Release;
        case "Dynamic":
            return ValueSocketType.Dynamic(json.data);
    }

    throw "Failed to parse json";
};

export const NodeRefSocketType = makeTaggedUnion({
    Button: none,
    Dynamic: (uid) => uid,
});

export function deserializeNodeRefSocketType(json: any): MemberType<typeof NodeRefSocketType> {
    switch (json.variant) {
        case "Button":
            return NodeRefSocketType.Button;
        case "Dynamic":
            return NodeRefSocketType.Dynamic(json.data);
    }

    throw "Failed to parse json";
};

export const Primitive = makeTaggedUnion({
    Float: (number: number) => number,
    Int: (number: number) => number,
    Boolean: (bool: boolean) => bool,
    String: (string: String) => string,
});

export function deserializePrimitive(json): MemberType<typeof Primitive> {
    return Primitive[json.variant](json.data);
};

export const SocketType = makeTaggedUnion({
    Stream: (streamType: MemberType<typeof StreamSocketType>) => streamType,
    Midi: (midiType: MemberType<typeof MidiSocketType>) => midiType,
    Value: (valueType: MemberType<typeof ValueSocketType>) => valueType,
    NodeRef: (nodeRef: MemberType<typeof NodeRefSocketType>) => nodeRef,
    MethodCall: (args: MemberType<typeof Primitive>[]) => args
});

// TODO: faster implementation
export function areSocketTypesEqual(socketType1: MemberType<typeof SocketType>, socketType2: MemberType<typeof SocketType>): boolean {
    return deepEqual(socketType1, socketType2);
}

export function deserializeSocketType (json: any) {
    switch (json["variant"]) {
        case "Stream":
            if (json.data.variant === "Dynamic") {
                return SocketType.Stream(StreamSocketType.Dynamic(json.data.data));
            }

            return SocketType.Stream(StreamSocketType[json.data.variant]);
        case "Midi":
            if (json.data.variant === "Dynamic") {
                return SocketType.Midi(MidiSocketType.Dynamic(json.data.data));
            }

            return SocketType.Midi(MidiSocketType[json.data.variant]);
        case "Value":
            if (json.data.variant === "Dynamic") {
                return SocketType.Value(ValueSocketType.Dynamic(json.data.data));
            }

            return SocketType.Value(ValueSocketType[json.data.variant]);
        case "NodeRef":
            if (json.data.variant === "Dynamic") {
                return SocketType.NodeRef(NodeRefSocketType.Dynamic(json.data.data));
            }

            return SocketType.NodeRef(NodeRefSocketType[json.data.variant]);
        case "MethodCall":
            return SocketType.MethodCall(json.data.variant);
    }

    throw "Failed to parse json";
}

export function socketTypeToKey(socketType: MemberType<typeof SocketType>) {
    return socketType.variant + "," + socketType.match({
        Stream: stream => stream.match({
            Dynamic: uid => stream.variant + uid,
            _: () => stream.variant,
        }),
        Midi: midi => midi.match({
            Dynamic: uid => midi.variant + uid,
            _: () => midi.variant,
        }),
        Value: value => value.match({
            Dynamic: uid => value.variant + uid,
            _: () => value.variant,
        }),
        NodeRef: nodeRef => nodeRef.match({
            Dynamic: uid => nodeRef.variant + uid,
            _: () => nodeRef.variant,
        }),
        MethodCall: args => args.toString()
    });
};

export function socketToKey(socket: MemberType<typeof SocketType>, direction: SocketDirection) {
    return socketTypeToKey(socket) + ":" + direction + socket.match({
        Stream: (stream) => stream.match({
            Dynamic: (uid) => ":" + uid,
            _: () => "_"
        }),
        Midi: (midi) => midi.match({
            Dynamic: (uid) => ":" + uid,
            _: () => "_"
        }),
        Value: (value) => value.match({
            Dynamic: (uid) => ":" + uid,
            _: () => "_"
        }),
        NodeRef: (nodeRef) => nodeRef.match({
            Dynamic: (uid) => ":" + uid,
            _: () => "_"
        }),
        _: () => ""
    });
}

export enum SocketDirection {
    Input = 0,
    Output = 1
};

export class Connection {
    fromSocketType: MemberType<typeof SocketType>;
    fromNode: NodeIndex;
    toSocketType: MemberType<typeof SocketType>;
    toNode: NodeIndex;

    constructor(fromSocketType: MemberType<typeof SocketType>, fromNode: NodeIndex, toSocketType: MemberType<typeof SocketType>, toNode: NodeIndex) {
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
        return socketTypeToKey(this.fromSocketType) + ":" +
               this.fromNode.toKey() + "->" +
               socketTypeToKey(this.toSocketType) + ":" +
               this.toNode.toKey();
    }
}

export class InputSideConnection {
    fromSocketType: MemberType<typeof SocketType>;
    fromNode: NodeIndex;
    toSocketType: MemberType<typeof SocketType>;

    constructor (fromSocketType: MemberType<typeof SocketType>, fromNode: NodeIndex, toSocketType: MemberType<typeof SocketType>) {
        this.fromSocketType = fromSocketType;
        this.fromNode = fromNode;
        this.toSocketType = toSocketType;
    }
}

export class OutputSideConnection {
    fromSocketType: MemberType<typeof SocketType>;
    toNode: NodeIndex;
    toSocketType: MemberType<typeof SocketType>;

    constructor (fromSocketType: MemberType<typeof SocketType>, toNode: NodeIndex, toSocketType: MemberType<typeof SocketType>) {
        this.fromSocketType = fromSocketType;
        this.toNode = toNode;
        this.toSocketType = toSocketType;
    }
}
