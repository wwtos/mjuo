import {createEnumDefinition, EnumInstance} from "../util/enum";
import { NodeIndex } from "./node";

export const MidiSocketType = createEnumDefinition({
    "Default": null,
});

export const StreamSocketType = createEnumDefinition({
    "Audio": null,
    "Gate": null,
    "Detune": null,
    "Dynamic": ["u64"]
});

export const ValueSocketType = createEnumDefinition({
    "Gain": null,
    "Frequency": null,
    "Gate": null
});

export const Parameter = createEnumDefinition({
    "Float": ["f32"],
    "Int": ["i32"],
    "Boolean": ["boolean"],
    "String": ["string"]
});

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
    var response = socketType.match([
        [SocketType.ids.Stream, ([stream/*: StreamSocketType*/]) => {
            return stream.match([
                [StreamSocketType.ids.Audio, () => "Audio"],
                [StreamSocketType.ids.Gate, () => "Gate"],
                [StreamSocketType.ids.Detune, () => "Detune"],
                [StreamSocketType.ids.Dynamic, (_) => "Dynamic"],
            ]);
        }],
        [SocketType.ids.Midi, ([midi/* :MidiSocketType*/]) => {
            return midi.match([
                [MidiSocketType.ids.Default, () => "Midi"]
            ]);
        }],
        [SocketType.ids.Value, ([value/* :ValueSocketType*/]) => {
            return value.match([
                [ValueSocketType.ids.Gain, () => "Gain value"],
                [ValueSocketType.ids.Frequency, () => "Frequency value"],
                [ValueSocketType.ids.Gate, () => "Gate value"],
            ]);
        }],
        [SocketType.ids.MethodCall, () => "Method call"]
    ]);

    return response;
}
