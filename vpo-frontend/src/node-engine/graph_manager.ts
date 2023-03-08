import type { Index } from "../ddgg/gen_vec";
import { Graph, VertexIndex } from "../ddgg/graph";
import type { IPCSocket } from "../util/socket";
import type { NodeWrapper } from "./node";
import { NodeConnection, NodeGraph } from "./node_graph";

export type ConnectedThrough = VertexIndex;

export class GraphManager {
    nodeGraphs: Graph<NodeGraph, ConnectedThrough>;
    ipcSocket: IPCSocket;
    ipcListenerRegistered: boolean = false;
    graphWaitingFor: Index | null = null;
    graphWaitingForEvent: Function | null = null;

    constructor() {
        this.nodeGraphs = {edges: {vec: []}, verticies: {vec: []}};
    }

    setIpcSocket(ipcSocket: IPCSocket) {
        this.ipcSocket = ipcSocket;
    }

    applyJson(json: {
            graphIndex: Index,
            nodes: Graph<NodeWrapper, NodeConnection>
    }) {
        if (!this.nodeGraphs[json.graphIndex.index]) {
            this.nodeGraphs[json.graphIndex.index] = new NodeGraph(this.ipcSocket, json.graphIndex);
        }

        this.nodeGraphs[json.graphIndex.index].applyJson(json);
    }

    onMessage ([message]) {
        if (message.action === "graph/updateGraph" &&
            message.payload.graphIndex === this.graphWaitingFor &&
            this.graphWaitingForEvent) {
            this.graphWaitingFor = null;
            this.graphWaitingForEvent(message);
        }
    }

    async getGraph(graphIndex: Index)/*: NodeGraph*/ {
        if (Graph.getVertex(this.nodeGraphs, graphIndex)) {
            return Graph.getVertex(this.nodeGraphs, graphIndex);
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

            let graph: {
                payload: {
                    nodes: Graph<NodeWrapper, NodeConnection>
                }
            } = (await graphPromise) as any;

            this.nodeGraphs.verticies[graphIndex.index] = {
                "variant": "Some",
                "data": new NodeGraph(this.ipcSocket, graphIndex)
            };

            this.nodeGraphs[graphIndex.index].applyJson(graph.payload);
        }

        return this.nodeGraphs[graphIndex.index];
    }

    getRootGraph() {
        if (!this.nodeGraphs[0]) {
            this.nodeGraphs[0] = new NodeGraph(this.ipcSocket, {index: 0, generation: 0});
        }

        return this.nodeGraphs[0];
    }
}
