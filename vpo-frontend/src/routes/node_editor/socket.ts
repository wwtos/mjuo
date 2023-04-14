import type { VertexIndex } from "$lib/ddgg/graph";
import type { Socket, SocketDirection, SocketValue } from "$lib/node-engine/connection";

export interface SocketEvent {
    event: MouseEvent;
    socket: Socket;
    direction: SocketDirection;
    vertexIndex: VertexIndex;
}

export interface OverrideUpdateEvent {
    socket: Socket;
    direction: SocketDirection;
    newValue: SocketValue;
}
