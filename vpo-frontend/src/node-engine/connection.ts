import {createEnumDefinition, EnumInstance} from "../util/enum";
import { NodeIndex } from "./node";
import { i18n } from '../i18n';

export const MidiSocketType = createEnumDefinition({
    "Default": null,
    "Dynamic": "u64",
});

MidiSocketType.deserialize = function (json) {
    switch (json.type) {
        case "Default":
            return MidiSocketType.Default;
        case "Dynamic":
            return MidiSocketType.Dynamic(json.content);
    }
};

export const StreamSocketType = createEnumDefinition({
    "Audio": null,
    "Gate": null,
    "Gain": null,
    "Detune": null,
    "Dynamic": "u64",
});

StreamSocketType.deserialize = function (json) {
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
};

export const ValueSocketType = createEnumDefinition({
    "Gain": null,
    "Frequency": null,
    "Gate": null,
    "Dynamic": "u64",
});

ValueSocketType.deserialize = function (json) {
    switch (json.type) {
        case "Gain":
            return ValueSocketType.Gain;
        case "Frequency":
            return ValueSocketType.Frequency;
        case "Gate":
            return ValueSocketType.Gate;
        case "Dynamic":
            return ValueSocketType.Dynamic(json.content);
    }
};

export const Primitive = createEnumDefinition({
    "Float": "f32",
    "Int": "i32",
    "Boolean": "boolean",
    "String": "string"
});

Primitive.deserialize = function(json) {
    return Primitive[json.type](json.content);
};

export const SocketType = createEnumDefinition({
    "Stream": [StreamSocketType],
    "Midi": [MidiSocketType],
    "Value": [ValueSocketType],
    "MethodCall": "array"
});

export function jsonToSocketType (json: object) {
    switch (json["type"]) {
        case "Stream":
            return SocketType.Stream(StreamSocketType[json["content"]["type"]]);
        break;
        case "Midi":
            return SocketType.Midi(MidiSocketType[json["content"]["type"]]);
        break;
        case "Value":
            return SocketType.Value(ValueSocketType[json["content"]["type"]]);
        break;
        case "MethodCall":
            return SocketType.MethodCall(json["content"]);
        break;
    }
}

export function socketTypeToKey(socketType: EnumInstance, recursiveKey?: string) {
    if (!recursiveKey) recursiveKey = "";

    recursiveKey += "," + socketType.getType();

    if (socketType.content && socketType.content[0] instanceof EnumInstance) {
        return socketTypeToKey(socketType.content[0], recursiveKey);
    } else {
        return recursiveKey;
    }
};

export function socketToKey(socket: EnumInstance/*SocketType*/, direction: SocketDirection) {
    return socketTypeToKey(socket) + ":" + direction;
}

export enum SocketDirection {
    Input = 0,
    Output = 1
};

export class Connection {
    fromSocketType: EnumInstance; /*SocketType*/
    fromNode: NodeIndex;
    toSocketType: EnumInstance; /*SocketType*/
    toNode: NodeIndex;

    constructor(fromSocketType: EnumInstance, fromNode: NodeIndex, toSocketType: EnumInstance, toNode: NodeIndex) {
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
    fromSocketType: EnumInstance; /*SocketType*/
    fromNode: NodeIndex;
    toSocketType: EnumInstance; /*SocketType*/

    constructor (fromSocketType: EnumInstance, fromNode: NodeIndex, toSocketType: EnumInstance) {
        this.fromSocketType = fromSocketType;
        this.fromNode = fromNode;
        this.toSocketType = toSocketType;
    }
}

export class OutputSideConnection {
    fromSocketType: EnumInstance; /*SocketType*/
    toNode: NodeIndex;
    toSocketType: EnumInstance; /*SocketType*/

    constructor (fromSocketType: EnumInstance, toNode: NodeIndex, toSocketType: EnumInstance) {
        this.fromSocketType = fromSocketType;
        this.toNode = toNode;
        this.toSocketType = toSocketType;
    }
}

export function socketTypeToString(socketType: /*SocketType*/EnumInstance): string {
    let response = socketType.match<string>([
        [SocketType.ids.Stream, ([stream/*: StreamSocketType*/]) => {
            return stream.match([
                [StreamSocketType.ids.Audio, () => i18n.t("socketType.stream.audio")],
                [StreamSocketType.ids.Gate, () => i18n.t("socketType.stream.gate")],
                [StreamSocketType.ids.Gain, () => i18n.t("socketType.stream.gain")],
                [StreamSocketType.ids.Detune, () => i18n.t("socketType.stream.detune")],
                [StreamSocketType.ids.Dynamic, (uid) => i18n.t("socketType.stream.dynamic", { uid })],
            ]);
        }],
        [SocketType.ids.Midi, ([midi/* :MidiSocketType*/]) => {
            return midi.match([
                [MidiSocketType.ids.Default, () => i18n.t("socketType.midi.default")],
                [StreamSocketType.ids.Dynamic, (uid) => i18n.t("socketType.midi.dynamic", { uid })],
            ]);
        }],
        [SocketType.ids.Value, ([value/* :ValueSocketType*/]) => {
            return value.match([
                [ValueSocketType.ids.Gain, () => i18n.t("socketType.value.gain")],
                [ValueSocketType.ids.Frequency, () => i18n.t("socketType.value.frequency")],
                [ValueSocketType.ids.Gate, () => i18n.t("socketType.value.gate")],
                [StreamSocketType.ids.Dynamic, (uid) => i18n.t("socketType.value.dynamic", { uid })],
            ]);
        }],
        [SocketType.ids.MethodCall, () => "Method call"]
    ]);

    return response;
}
