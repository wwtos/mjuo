import type { DiscriminatedUnion } from "$lib/util/discriminated-union";
import type { Range } from "../util/rust_std";
import type { IoRoutes } from "./io_routing";

export type MySampleFormat = DiscriminatedUnion<"variant", {
    I8: {},
    I16: {},
    I32: {},
    I64: {},
    U8: {},
    U16: {},
    U32: {},
    U64: {},
    F32: {},
    F64: {},
}>;

export interface SoundConfig {
    sampleRate: number;
}

export interface StreamConfigOptions {
    channels: Range,
    sample_rate: Range,
    buffer_size: Range,
    sample_formats: Array<MySampleFormat>,
}

export interface AudioDeviceStatus {
    name: String,
    sourceOptions: StreamConfigOptions | null,
    sinkOptions: StreamConfigOptions | null,
}

export interface MidiDeviceStatus {
    name: String,
}

export interface GlobalState {
    activeProject: string | null;
    soundConfig: SoundConfig;
    devices: {
        midi: {
            [key: string]: MidiDeviceStatus
        },
        streams: {
            [key: string]: AudioDeviceStatus
        }
    };
    ioRoutes: IoRoutes
}

export interface Resources {
    ui: { [key: string]: any };
}
