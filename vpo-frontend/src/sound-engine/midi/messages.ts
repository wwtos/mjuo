import type { DiscriminatedUnion } from "../../lib/util/discriminated-union";

export interface Timecode {
    hours: number,
    minutes: number,
    seconds: number,
}

export type SystemCommonMessageData = DiscriminatedUnion<"variant", {
    SystemExclusive: { content: { id: Uint8Array, message: Uint8Array }},
    QuarterFrame: { content: { rate: number, time: Timecode }},
}>;

export type SystemRealtimeMessageData = DiscriminatedUnion<"variant", {
    TimingClock: {},
    Start: {},
    Continue: {},
    Stop: {},
    ActiveSensing: {},
    Reset: {},
}>;

export type MidiData = DiscriminatedUnion<"variant", {
    NoteOff: { content: { channel: number, note: number, velocity: number }},
    NoteOn: { content: { channel: number, note: number, velocity: number }},
    Aftertouch: { content: { channel: number, note: number, pressure: number }},
    ControlChange: { content: { channel: number, controller: number, value: number }},
    ProgramChange: { content: { channel: number, patch: number }},
    ChannelAftertouch: { content: { channel: number, pitchBend: number }},
    PitchBend: { content: { channel: number, bend: number }},
    SystemCommonMessage: { content: { data: SystemCommonMessageData }},
    SystemRealtimeMessage: { content: { data: SystemRealtimeMessageData }},
    MidiNone: {}
}>;
