import {createEnumDefinition, EnumInstance} from "../util/enum";

export const MidiSocketType = createEnumDefinition({
    "Default": null
});

export const StreamSocketType = createEnumDefinition({
    "Audio": null,
    "Gate": null,
    "Detune": null,
    "Dynamic": ["u64"]
});

export const ValueType = createEnumDefinition({
    "Gain": null
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
    "Value": [ValueType],
    "MethodCall": "array"
});

export enum SocketDirection {
    Input = 0,
    Output = 1
};

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
        [SocketType.ids.Value, ([value/* :ValueType*/]) => {
            return value.match([
                [ValueType.ids.Gain, () => "Gain value"]
            ]);
        }],
        [SocketType.ids.MethodCall, () => "Method call"]
    ]);

    return response;
}
