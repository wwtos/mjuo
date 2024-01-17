import type { DiscriminatedUnion } from "$lib/util/discriminated-union";

export type DeviceType = DiscriminatedUnion<"variant", {
    Midi: {},
    Stream: {},
}>;

export type DeviceDirection = DiscriminatedUnion<"variant", {
    Source: {},
    Sink: {},
}>;

export type RouteRule = {
    deviceId: string,
    deviceType: DeviceType,
    deviceDirection: DeviceDirection,
    deviceChannel: number,
    node: string,
    nodeChannel: number,
};

export type DeviceInfo = {
    name: string,
    deviceType: DeviceType,
    deviceDirection: DeviceDirection,
    channels: number,
    bufferSize: number,
};

export type IoRoutes = {
    rules: Array<RouteRule>,
    devices: Array<DeviceInfo>,
};