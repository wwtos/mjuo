import { BehaviorSubject } from "rxjs";
import { NodeGraph } from "../node-engine/node_graph";
import { SocketRegistry } from "../node-engine/socket_registry";

export const socketRegistry: BehaviorSubject<SocketRegistry> = new BehaviorSubject(new SocketRegistry());
export const graph: BehaviorSubject<NodeGraph | undefined> = new BehaviorSubject<NodeGraph | undefined>(undefined);
