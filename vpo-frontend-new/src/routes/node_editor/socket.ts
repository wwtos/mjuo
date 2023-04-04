import type { VertexIndex } from "$lib/ddgg/graph";
import type { SocketDirection, SocketType } from "$lib/node-engine/connection";

export interface SocketEvent {
    event: MouseEvent;
    type: SocketType;
    direction: SocketDirection;
    vertexIndex: VertexIndex;
}
