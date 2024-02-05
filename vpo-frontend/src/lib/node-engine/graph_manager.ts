import { Index } from "$lib/ddgg/gen_vec";
import type { IpcSocket } from "$lib/ipc/socket";
import { Graph, type VertexIndex } from "../ddgg/graph";
import type { NodeInstance } from "./node";
import { type NodeConnection, NodeGraph } from "./node_graph";

export type ConnectedThrough = VertexIndex;

export interface GlobalNodeIndex {
    graphIndex: VertexIndex;
    nodeIndex: VertexIndex;
}

export class GraphManager {
    nodeGraphs: Graph<NodeGraph, ConnectedThrough>;
    ipcSocket: IpcSocket;
    ipcListenerRegistered: boolean = false;
    graphWaitingFor: string | null = null;
    graphWaitingForEvent: Function | null = null;

    constructor(ipcSocket: IpcSocket) {
        this.nodeGraphs = { edges: [], verticies: [] };
        this.ipcSocket = ipcSocket;
    }

    applyJson(json: {
        graphIndex: string;
        nodes: Graph<NodeInstance, NodeConnection>;
    }) {
        if (!Graph.getVertex(this.nodeGraphs, json.graphIndex)) {
            this.nodeGraphs.verticies[Index.fromString(json.graphIndex).index] =
            {
                variant: "Occupied",
                data: [
                    {
                        connectionsFrom: [],
                        connectionsTo: [],
                        data: new NodeGraph(
                            this.ipcSocket,
                            json.graphIndex
                        ),
                    },
                    Index.fromString(json.graphIndex).generation,
                ],
            };
        }

        Graph.getVertexData(this.nodeGraphs, json.graphIndex)?.applyJson(json);
    }

    onMessage(message: any) {
        if (
            message.action === "graph/updateGraph" &&
            message.payload.graphIndex === this.graphWaitingFor &&
            this.graphWaitingForEvent
        ) {
            this.graphWaitingFor = null;
            this.graphWaitingForEvent(message);
        }
    }

    async getGraph(graphIndex: string): Promise<NodeGraph> {
        if (Graph.getVertex(this.nodeGraphs, graphIndex)) {
            return Graph.getVertexData(
                this.nodeGraphs,
                graphIndex
            ) as NodeGraph;
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
                    nodes: Graph<NodeInstance, NodeConnection>;
                };
            } = (await graphPromise) as any;

            this.nodeGraphs.verticies[Index.fromString(graphIndex).index] = {
                variant: "Occupied",
                data: [
                    {
                        data: new NodeGraph(this.ipcSocket, graphIndex),
                        connectionsFrom: [],
                        connectionsTo: [],
                    },
                    Index.fromString(graphIndex).generation,
                ],
            };

            Graph.getVertexData(this.nodeGraphs, graphIndex)?.applyJson(
                graph.payload
            );
        }

        return Graph.getVertexData(this.nodeGraphs, graphIndex) as NodeGraph;
    }

    getRootGraph() {
        if (!Graph.getVertexData(this.nodeGraphs, "0.0")) {
            this.nodeGraphs.verticies[0] = {
                variant: "Occupied",
                data: [
                    {
                        data: new NodeGraph(this.ipcSocket, "0.0"),
                        connectionsFrom: [],
                        connectionsTo: [],
                    },
                    0,
                ],
            };
        }

        return Graph.getVertexData(this.nodeGraphs, "0.0") as NodeGraph;
    }
}
