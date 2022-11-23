import type { SocketDirection, SocketType } from "../node-engine/connection";
import type { NodeIndex } from "../node-engine/node_index";

export interface SocketEvent {
    event: MouseEvent;
    type: SocketType;
    direction: SocketDirection;
    nodeIndex?: NodeIndex;
}
