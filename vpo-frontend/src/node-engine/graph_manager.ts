import type { IPCSocket } from "../util/socket";
import { NodeGraph } from "./node_graph";

export class GraphManager {
    graphs: {[key: string]: NodeGraph} = {};
    ipcSocket: IPCSocket;
    ipcListenerRegistered: boolean = false;
    graphWaitingFor: number | null = null;
    graphWaitingForEvent: Function | null = null;

    setIpcSocket(ipcSocket: IPCSocket) {
        this.ipcSocket = ipcSocket;
    }

    applyJson(json: any) {
        if (!this.graphs[json.payload.graphIndex]) {
            this.graphs[json.payload.graphIndex] = new NodeGraph(this.ipcSocket, json.payload.graphIndex);
        }

        this.graphs[json.payload.graphIndex].applyJson(json.payload);
    }

    onMessage ([message]) {
        if (message.action === "graph/updateGraph" &&
            message.payload.graphIndex === this.graphWaitingFor &&
            this.graphWaitingForEvent) {
            this.graphWaitingFor = null;
            this.graphWaitingForEvent(message);
        }
    }

    async getGraph(graphIndex: number)/*: NodeGraph*/ {
        if (this.graphs[graphIndex]) {
            return this.graphs[graphIndex];
        } else {
            if (!this.ipcListenerRegistered) {
                this.ipcSocket.onMessage(this.onMessage.bind(this));
                this.ipcListenerRegistered = true;
            }

            this.graphWaitingFor = graphIndex;

            let graphPromise = new Promise((resolve, _) => {
                this.graphWaitingForEvent = resolve;
            });

            this.ipcSocket.requestGraph(graphIndex);

            let graph: any = await graphPromise;

            this.graphs[graphIndex] = new NodeGraph(this.ipcSocket, graphIndex);
            this.graphs[graphIndex].applyJson(graph.payload);
        }

        return this.graphs[graphIndex];
    }

    getRootGraph() {
        if (!this.graphs[0]) {
            this.graphs[0] = new NodeGraph(this.ipcSocket, 0);
        }

        return this.graphs[0];
    }
}
