import type { Index } from "../ddgg/gen_vec";
import { Graph, type VertexIndex } from "../ddgg/graph";
import type { IPCSocket } from "../util/socket";
import type { NodeWrapper } from "./node";
import { type NodeConnection, NodeGraph } from "./node_graph";

export type ConnectedThrough = VertexIndex;

export class GraphManager {
    nodeGraphs: Graph<NodeGraph, ConnectedThrough>;
    ipcSocket: IPCSocket;
    ipcListenerRegistered: boolean = false;
    graphWaitingFor: Index | null = null;
    graphWaitingForEvent: Function | null = null;

    constructor(ipcSocket: IPCSocket) {
        this.nodeGraphs = {edges: {vec: []}, verticies: {vec: []}};
        this.ipcSocket = ipcSocket;
    }

    applyJson(json: {
            graphIndex: Index,
            nodes: Graph<NodeWrapper, NodeConnection>
    }) {
        if (!Graph.getVertex(this.nodeGraphs, json.graphIndex)) {
            this.nodeGraphs.verticies.vec[json.graphIndex.index] = {variant: "Occupied", data: [{
                connectionsFrom: [],
                connectionsTo: [],
                data: new NodeGraph(this.ipcSocket, json.graphIndex)
            }, json.graphIndex.generation]};
        }

        Graph.getVertexData(this.nodeGraphs, json.graphIndex)?.applyJson(json);
    }

    onMessage ([message]: [any]) {
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

            this.nodeGraphs.verticies.vec[graphIndex.index] = {
                "variant": "Occupied",
                "data": [{
                    data: new NodeGraph(this.ipcSocket, graphIndex),
                    connectionsFrom: [],
                    connectionsTo: []
                }, graphIndex.generation]
            };

            Graph.getVertexData(this.nodeGraphs, graphIndex)?.applyJson(graph.payload);
        }

        return Graph.getVertexData(this.nodeGraphs, graphIndex);
    }

    getRootGraph() {
        if (!Graph.getVertexData(this.nodeGraphs, {index: 0, generation: 0})) {
            this.nodeGraphs.verticies.vec[0] = {
                "variant": "Occupied",
                "data": [{
                    data:  new NodeGraph(this.ipcSocket, {index: 0, generation: 0}),
                    connectionsFrom: [],
                    connectionsTo: []
                }, 0]
            };
        }

        return Graph.getVertexData(this.nodeGraphs, {index: 0, generation: 0});
    }
}
