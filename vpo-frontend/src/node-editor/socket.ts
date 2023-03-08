import type { VertexIndex } from "../ddgg/graph";
import type { SocketDirection, SocketType } from "../node-engine/connection";

export interface SocketEvent {
    event: MouseEvent;
    type: SocketType;
    direction: SocketDirection;
    vertexIndex?: VertexIndex;
}
