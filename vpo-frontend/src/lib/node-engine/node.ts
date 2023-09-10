import type { Property, PropertyType } from "./property";
import {
    type DiscriminatedUnion,
    match,
    matchOrElse,
} from "../util/discriminated-union";
import type { Index } from "../ddgg/gen_vec";
import type { Socket, SocketDirection, SocketValue } from "./connection";

export const TITLE_HEIGHT = 30;
export const SOCKET_HEIGHT = 36;
export const SOCKET_OFFSET = 26;
export const NODE_WIDTH = 270;

export type NodeRow = DiscriminatedUnion<
    "variant",
    {
        Input: { data: [Socket, SocketValue] };
        Output: { data: Socket };
        Property: { data: [string, PropertyType, Property] };
        InnerGraph: { data: undefined };
    }
>;

type SocketAndDirection = { socket: Socket; direction: SocketDirection };

export const NodeRow = {
    toSocketAndDirection: (
        nodeRow: NodeRow
    ): SocketAndDirection | undefined => {
        return matchOrElse(
            nodeRow,
            {
                Input: ({ data: [socket, _] }): SocketAndDirection => ({
                    socket: socket,
                    direction: { variant: "Input" },
                }),
                Output: ({ data: socket }) => ({
                    socket: socket,
                    direction: { variant: "Output" },
                }),
            },
            () => undefined
        );
    },
    fromTypeAndDirection: (
        socket: Socket,
        direction: SocketDirection,
        defaultValue: SocketValue
    ): NodeRow => {
        if (direction.variant === "Input") {
            return {
                variant: "Input",
                data: [socket, defaultValue],
            };
        } else {
            return {
                variant: "Output",
                data: socket,
            };
        }
    },
    getDefault(nodeRow: NodeRow): SocketValue {
        return matchOrElse(
            nodeRow,
            {
                Input: ({ data: [_, defaultValue] }) => defaultValue,
                Output: ({ data: _ }) => ({ variant: "None" }),
            },
            () => ({ variant: "None" })
        );
    },
    getHeight(nodeRow: NodeRow): number {
        return SOCKET_HEIGHT;
    },
};

export interface UiElementInstance {
    resourceId: string;
    properties: { [key: string]: string };
    x: number;
    y: number;
    selected: boolean;
}

export interface UiData {
    x: number;
    y: number;
    selected?: boolean;
    title?: string;
    panelInstances?: {
        [key: string]: UiElementInstance[];
    };
}

export interface Node {
    inputSockets: Socket[];
    outputSockets: Socket[];
    usableProperties: {
        [prop: string]: PropertyType;
    };
}

export interface NodeInstance {
    nodeType: string;
    nodeRows: NodeRow[];
    defaultOverrides: NodeRow[];
    properties: { [key: string]: Property };
    uiData: UiData;
    childGraphIndex: Index | null;
    state: {
        countedDuringMapset: boolean;
        value: any;
        other: any;
    };
}
export interface GenerationalNode {
    node: NodeInstance;
    generation: number;
}
