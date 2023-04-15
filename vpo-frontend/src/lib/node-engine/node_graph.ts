import { NodeRow, type NodeWrapper, NODE_WIDTH, SOCKET_HEIGHT, SOCKET_OFFSET, TITLE_HEIGHT } from "./node";
import { BehaviorSubject } from "rxjs";
import { deepEqual } from "fast-equals";
import { matchOrElse } from "../util/discriminated-union";
import type { Property } from "./property";
import { Index } from "../ddgg/gen_vec";
import { Graph, type Vertex, type VertexIndex } from "../ddgg/graph";
import type { IpcSocket } from "$lib/ipc/socket";
import type { Connection, InputSideConnection, OutputSideConnection, Socket, SocketDirection, SocketValue } from "./connection";

// import {Node, VertexIndex, GenerationalNode} from "./node";

export interface NodeConnection {
    fromSocket: Socket,
    toSocket: Socket,
}

export class NodeGraph {
    private nodes: Graph<NodeWrapper, NodeConnection>;
    nodeStore: BehaviorSubject<Array<[Vertex<NodeWrapper>, VertexIndex]>>;
    keyedNodeStore: BehaviorSubject<([string, NodeWrapper, VertexIndex])[]>;
    keyedConnectionStore: BehaviorSubject<([string, Connection])[]>;
    changedNodes: VertexIndex[];
    ipcSocket: IpcSocket;
    graphIndex: Index;
    selectedNodes: [];

    constructor (ipcSocket: IpcSocket, graphIndex: Index) {
        this.ipcSocket = ipcSocket;

        this.nodes = {verticies: {vec: []}, edges: {vec: []}};

        this.nodeStore = new BehaviorSubject(Graph.verticies(this.nodes));
        this.keyedNodeStore = new BehaviorSubject(this.getKeyedNodes());
        this.keyedConnectionStore = new BehaviorSubject(this.getKeyedConnections());

        this.changedNodes = [];

        this.graphIndex = graphIndex;

        this.selectedNodes = [];
    }

    getNode (index: VertexIndex): (NodeWrapper | undefined) {
        return Graph.getVertexData(this.nodes, index);
    }

    getNodeVertex (index: VertexIndex): (Vertex<NodeWrapper> | undefined) {
        return Graph.getVertex(this.nodes, index);
    }

    update() {
        this.keyedNodeStore.next(this.getKeyedNodes());
        this.keyedConnectionStore.next(this.getKeyedConnections());
        this.nodeStore.next(Graph.verticies(this.nodes));
    }

    applyJson(json: {
        nodes: Graph<NodeWrapper, NodeConnection>
    }) {
        this.nodes = json.nodes;

        this.update();
    }

    private getKeyedConnections (): ([string, Connection])[] {
        let keyedConnections: ([string, Connection])[] = [];

        for (let [edge, index] of Graph.edges(this.nodes)) {
            let newConnection: Connection = {
                "fromNode": edge.from,
                "toNode": edge.to,
                "data": edge.data
            };

            keyedConnections.push([
                "(" + Index.toKey(this.graphIndex) + ") " + JSON.stringify(newConnection),
                newConnection
            ]);
        }

        return keyedConnections;
    }

    private getKeyedNodes (): ([string, NodeWrapper, VertexIndex])[] {
        let keyedNodes: ([string, NodeWrapper, VertexIndex])[] = [];

        for (let [node, index] of Graph.verticies(this.nodes)) {
            keyedNodes.push(["(" + Index.toKey(this.graphIndex) + ") " + Index.toKey(index), node.data, index]);
        }

        return keyedNodes;
    }

    markNodeAsUpdated(index: VertexIndex) {
        console.log(`node ${index} was updated`);
        
        // don't mark it for updating if it's already been marked
        if (this.changedNodes.find(vertexIndex => vertexIndex.index === index.index && vertexIndex.generation === index.generation)) return;

        this.changedNodes.push(index);
    }

    writeChangedNodesToServer() {
        // only write changes if any nodes were marked for updating
        if (this.changedNodes.length > 0) {
            this.ipcSocket.updateNodes(this, this.changedNodes);

            this.changedNodes.length = 0;
        }
    }

    writeChangedNodesToServerUi() {
        // only write changes if any nodes were marked for updating
        if (this.changedNodes.length > 0) {
            this.ipcSocket.updateNodesUi(this, this.changedNodes);

            this.changedNodes.length = 0;
        }
    }

    updateNode(vertexIndex: VertexIndex) {
        // TODO: naïve
        this.nodeStore.next(Graph.verticies(this.nodes));
    }

    getNodeInputConnection(vertexIndex: VertexIndex, socket: Socket): InputSideConnection | undefined {
        const node = this.getNodeVertex(vertexIndex);
        
        if (node && node.connectionsFrom) {
            let connection = node.connectionsFrom
                .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
                .filter(edge => edge && deepEqual(edge.data.toSocket, socket))
                .map(edge => (edge && 
                    {
                        fromSocket: edge.data.fromSocket,
                        fromNode: edge.from,
                        toSocket: edge.data.toSocket
                    }
                ));

            return connection[0];
        }
    }

    getNodeOutputConnections(vertexIndex: VertexIndex, socket: Socket): OutputSideConnection[] {
        let node = this.getNodeVertex(vertexIndex);

        if (!node) return [];

        let connections = node.connectionsFrom
            .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
            .filter(edge => edge && deepEqual(edge.data.toSocket, socket))
            .map(edge => (edge && 
                {
                    fromSocket: edge.data.fromSocket,
                    toNode: edge.to,
                    toSocket: edge.data.toSocket
                }
            ));

        return connections as OutputSideConnection[];
    }

    getNodeSocketDefault(vertexIndex: VertexIndex, socket: Socket, direction: SocketDirection): SocketValue {
        const node = this.getNode(vertexIndex);

        if (node) {
            const defaultOverride = node.defaultOverrides.find(defaultOverride => {
                const typeAndDirection = NodeRow.toSocketAndDirection(defaultOverride);

                if (typeAndDirection) {
                    const {
                        socket: overrideSocketType,
                        direction: overrideDirection
                    } = typeAndDirection;

                    return deepEqual(socket, overrideSocketType) &&
                        direction.variant === overrideDirection.variant;
                }
            });

            if (defaultOverride && defaultOverride.data) return NodeRow.getDefault(defaultOverride);

            const defaultNodeRow = node.nodeRows.find(nodeRow => {
                const typeAndDirection = NodeRow.toSocketAndDirection(nodeRow);

                if (typeAndDirection) {
                    const {
                        socket: nodeRowSocketType,
                        direction: nodeRowDirection
                    } = typeAndDirection;

                    return deepEqual(socket, nodeRowSocketType) &&
                        direction.variant === nodeRowDirection.variant;
                }
            });

            if (defaultNodeRow && defaultNodeRow.data) return NodeRow.getDefault(defaultNodeRow);
        }

        return { variant: "None" };
    }

    getNodePropertyValue(vertexIndex: VertexIndex, propName: string): Property | undefined {
        const node = this.getNode(vertexIndex);

        if (node) {
            if (node.properties[propName]) return node.properties[propName];

                    const row = node.nodeRows.find(nodeRow => {
                        return matchOrElse(nodeRow, 
                            {
                                Property({ data: [rowName] }) {
                                    return rowName === propName;
                                }
                            },
                            () => false
                        );
                    });

            if (!row) return undefined;

            return matchOrElse(row, {
                    Property: ({ data: [_name, _type, defaultVal ]}) => defaultVal
                },
                () => { throw new Error("unreachable"); }
            );
        }
    }

    getNodeSocketXy(index: VertexIndex, socket: Socket, direction: SocketDirection): { x: number, y: number } {
        const node = this.getNode(index);

        if (!node) return { x: 0, y: 0 };

        let y = TITLE_HEIGHT;

        const rowIndex = node.nodeRows.findIndex(nodeRow => {
            const typeAndDirection = NodeRow.toSocketAndDirection(nodeRow);
            const height = NodeRow.getHeight(nodeRow);

            y += height;

            if (typeAndDirection) {
                const {
                    socket: rowSocketType,
                    direction: rowDirection
                 } = typeAndDirection;

                return deepEqual(socket, rowSocketType) && rowDirection.variant === direction.variant;
            }

            return false;
        });

        if (rowIndex === -1) return { x: 0, y: 0 };

        const relativeX = direction.variant === "Output" ? NODE_WIDTH : 0;
        const relativeY = (y - SOCKET_HEIGHT) + SOCKET_OFFSET;

        return { x: node.uiData.x + relativeX, y: node.uiData.y + relativeY };
    }
}
