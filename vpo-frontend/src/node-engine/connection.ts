import type { NodeIndex } from "./node";
import { i18n } from '../i18n';
import { deepEqual } from "fast-equals";
import {makeTaggedUnion, MemberType, none} from "safety-match";

export const MidiSocketType = makeTaggedUnion({
    Default: none,
    Dynamic: (uid) => uid,
});

export function deserializeMidiSocketType(json: any): MemberType<typeof MidiSocketType> {
    switch (json.type) {
        case "Default":
            return MidiSocketType.Default;
        case "Dynamic":
            return MidiSocketType.Dynamic(json.content);
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
    switch (json.type) {
        case "Audio":
            return StreamSocketType.Audio;
        case "Gate":
            return StreamSocketType.Gate;
        case "Gain":
            return StreamSocketType.Gain;
        case "Detune":
            return StreamSocketType.Detune;
        case "Dynamic":
            return StreamSocketType.Dynamic(json.content);
    }

    throw "Failed to parse json";
};

export const ValueSocketType = makeTaggedUnion({
    Gain: none,
    Frequency: none,
    Gate: none,
    Attack: none,
    Decay: none,
    Sustain: none,
    Release: none,
    Dynamic: (uid: number) => uid,
});

export function deserializeValueSocketType(json: any): MemberType<typeof ValueSocketType> {
    switch (json.type) {
        case "Gain":
            return ValueSocketType.Gain;
        case "Frequency":
            return ValueSocketType.Frequency;
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
            return ValueSocketType.Dynamic(json.content);
    }

    throw "Failed to parse json";
};

export const NodeRefSocketType = makeTaggedUnion({
    Button: none,
    Dynamic: (uid) => uid,
});

export function deserializeNodeRefSocketType(json: any): MemberType<typeof NodeRefSocketType> {
    switch (json.type) {
        case "Button":
            return NodeRefSocketType.Button;
        case "Dynamic":
            return NodeRefSocketType.Dynamic(json.content);
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
    return Primitive[json.type](json.content);
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

export function jsonToSocketType (json: object) {
    switch (json["type"]) {
        case "Stream":
            return SocketType.Stream(StreamSocketType[json["content"]["type"]]);
        case "Midi":
            return SocketType.Midi(MidiSocketType[json["content"]["type"]]);
        case "Value":
            return SocketType.Value(ValueSocketType[json["content"]["type"]]);
        case "MethodCall":
            return SocketType.MethodCall(json["content"]);
    }

    throw "Failed to parse json";
}

export function socketTypeToKey(socketType: MemberType<typeof SocketType>) {
    return socketType.variant + ", " + socketType.match({
        Stream: stream => stream.variant,
        Midi: midi => midi.variant,
        Value: value => value.variant,
        NodeRef: nodeRef => nodeRef.variant,
        MethodCall: args => args.toString()
    });
};

export function socketToKey(socket: MemberType<typeof SocketType>, direction: SocketDirection) {
    return socketTypeToKey(socket) + ":" + direction;
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
               this.fromNode.toKey() + ":" +
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

export function socketTypeToString(socketType: MemberType<typeof SocketType>): string {
    let response = socketType.match({
        Stream: (stream) => stream.match({
            Audio: () => i18n.t("socketType.stream.audio"),
            Gate: () => i18n.t("socketType.stream.gate"),
            Gain: () => i18n.t("socketType.stream.gain"),
            Detune: () => i18n.t("socketType.stream.detune"),
            Dynamic: (uid) => i18n.t("socketType.midi.stream", { uid })
        }),
        Midi: (midi) => midi.match({
            Default: () => i18n.t("socketType.midi.default"),
            Dynamic: (uid) => i18n.t("socketType.midi.dynamic", { uid })
        }),
        Value: (value) => value.match({
            Gain: () => i18n.t("socketType.value.gain"),
            Frequency: () => i18n.t("socketType.value.frequency"),
            Gate: () => i18n.t("socketType.value.gate"),
            Attack: () => i18n.t("socketType.value.attack"),
            Decay: () => i18n.t("socketType.value.decay"),
            Sustain: () => i18n.t("socketType.value.sustain"),
            Release: () => i18n.t("socketType.value.release"),
            Dynamic: (uid) => i18n.t("socketType.value.dynamic", { uid })
        }),
        NodeRef: (nodeRef) => nodeRef.match({
            Button: () => i18n.t("socketType.noderef.button"),
            Dynamic: (uid) => i18n.t("socketType.noderef.dynamic", { uid })
        }),
        MethodCall: () => "Method call",
    });

    return response;
}
