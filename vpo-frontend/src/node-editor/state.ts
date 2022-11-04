import { BehaviorSubject } from "rxjs";
import { GraphManager } from "../node-engine/graph_manager";
import { NodeGraph } from "../node-engine/node_graph";
import { SocketRegistry } from "../node-engine/socket_registry";
import { IPCSocket } from "../util/socket";

export const socketRegistry: BehaviorSubject<SocketRegistry> = new BehaviorSubject(new SocketRegistry());
export const ipcSocket: BehaviorSubject<IPCSocket | undefined> = new BehaviorSubject<IPCSocket | undefined>(undefined);
export const graphManager = new GraphManager();
export const activeEditor = new BehaviorSubject<"nodes" | "ui" | "files">("nodes");
