import {createEnumDefinition} from "../util/enum";

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
