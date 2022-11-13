import { BehaviorSubject } from "rxjs";
import { GraphManager } from "../node-engine/graph_manager";
import { SocketRegistry } from "../node-engine/socket_registry";
import { IPCSocket } from "../util/socket";
import { type GlobalState } from "../node-engine/global_state";

export const socketRegistry: BehaviorSubject<SocketRegistry> = new BehaviorSubject(new SocketRegistry());
export const ipcSocket: BehaviorSubject<IPCSocket | undefined> = new BehaviorSubject<IPCSocket | undefined>(undefined);
export const graphManager = new GraphManager();
export const activeEditor = new BehaviorSubject<"nodes" | "ui" | "files">("files");
export const globalState = new BehaviorSubject<GlobalState>({
    active_project: null,
    sound_config: {sample_rate: 0},
    resources: []
});
