import { NodeRow, NodeWrapper, NODE_WIDTH, SocketValue, SOCKET_HEIGHT, SOCKET_OFFSET, TITLE_HEIGHT } from "./node";
import { InputSideConnection, OutputSideConnection, Connection, SocketType, SocketDirection } from "./connection";
import type { IPCSocket } from "../util/socket";
import { BehaviorSubject, Observable } from "rxjs";
import { distinctUntilChanged, filter, map } from "rxjs/operators";
import { deepEqual, shallowEqual } from "fast-equals";
import { matchOrElse } from "../util/discriminated-union";
import type { Property } from "./property";
import { Index } from "../ddgg/gen_vec";
import { Graph, Vertex, VertexIndex } from "../ddgg/graph";
import { isDefined } from "../util/rxjs_extensions";

// import {Node, VertexIndex, GenerationalNode} from "./node";

export interface NodeConnection {
    fromSocketType: SocketType,
    toSocketType: SocketType,
}

export class NodeGraph {
    private nodes: Graph<NodeWrapper, NodeConnection>;
    nodeStore: BehaviorSubject<Array<[Vertex<NodeWrapper>, VertexIndex]>>;
    keyedNodeStore: BehaviorSubject<([string, NodeWrapper, VertexIndex])[]>;
    keyedConnectionStore: BehaviorSubject<([string, Connection])[]>;
    changedNodes: VertexIndex[];
    ipcSocket: IPCSocket;
    graphIndex: Index;
    selectedNodes: [];

    constructor (ipcSocket: IPCSocket, graphIndex: Index) {
        this.ipcSocket = ipcSocket;

        this.nodes = {verticies: {vec: []}, edges: {vec: []}};

        this.nodeStore = new BehaviorSubject(Graph.verticies(this.nodes));
        this.keyedNodeStore = new BehaviorSubject(this.getKeyedNodes());
        this.keyedConnectionStore = new BehaviorSubject(this.getKeyedConnections());

        this.changedNodes = [];

        this.graphIndex = graphIndex;
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
        graphIndex: Index,
        nodes: Graph<NodeWrapper, NodeConnection>
    }) {
        this.graphIndex = json.graphIndex;

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
                "(" + Index.toKey(this.graphIndex) + ") " + Connection.getKey(newConnection),
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

    subscribeToNode(vertexIndex: VertexIndex): Observable<Vertex<NodeWrapper>> {
        return this.nodeStore.pipe(
            map(nodes => {
                return nodes.find(([_, index]) => deepEqual(index, vertexIndex));
            }),
            filter(isDefined),
            map(([vertex, _]) => vertex),
            distinctUntilChanged(deepEqual)
        );
    }

    updateNode(vertexIndex: VertexIndex) {
        // TODO: naïve
        this.nodeStore.next(Graph.verticies(this.nodes));
    }

    getNodeInputConnection(vertexIndex: VertexIndex, socketType: SocketType): Observable<InputSideConnection | undefined> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(node => {
                if (node && node.connectionsFrom) {
                    let connection = node.connectionsFrom
                        .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
                        .filter(edge => edge && SocketType.areEqual(edge.data.fromSocketType, socketType))
                        .map(edge => (edge && 
                            {
                                fromSocketType: edge.data.fromSocketType,
                                fromNode: edge.from,
                                toSocketType: edge.data.toSocketType
                            }
                        ));

                    return connection[0];
                }
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeInputConnectionImmediate(vertexIndex: VertexIndex, socketType: SocketType): InputSideConnection | undefined {
        let node = this.getNodeVertex(vertexIndex);

        if (!node) return undefined;

        let connections = node.connectionsFrom
            .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
            .filter(edge => edge && SocketType.areEqual(edge.data.fromSocketType, socketType))
            .map(edge => (edge && 
                {
                    fromSocketType: edge.data.fromSocketType,
                    fromNode: edge.from,
                    toSocketType: edge.data.toSocketType
                }
            ));

        return connections[0];
    }

    getNodeOutputConnections(vertexIndex: VertexIndex, socketType: SocketType): Observable<OutputSideConnection[]> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(node => {
                let connections = node.connectionsFrom
                    .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
                    .filter(edge => edge && SocketType.areEqual(edge.data.toSocketType, socketType))
                    .map(edge => (edge && 
                        {
                            fromSocketType: edge.data.fromSocketType,
                            toNode: edge.to,
                            toSocketType: edge.data.toSocketType
                        }
                    ));

                return connections as OutputSideConnection[];
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeOutputConnectionsImmediate(vertexIndex: VertexIndex, socketType: SocketType): OutputSideConnection[] {
        let node = this.getNodeVertex(vertexIndex);

        if (!node) return [];

        let connections = node.connectionsFrom
            .map(([_, input_index]) => Graph.getEdge(this.nodes, input_index))
            .filter(edge => edge && SocketType.areEqual(edge.data.toSocketType, socketType))
            .map(edge => (edge && 
                {
                    fromSocketType: edge.data.fromSocketType,
                    toNode: edge.to,
                    toSocketType: edge.data.toSocketType
                }
            ));

        return connections as OutputSideConnection[];
    }

    getNodeSocketDefault(vertexIndex: VertexIndex, socketType: SocketType, direction: SocketDirection): Observable<SocketValue> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(({data: node}) => {
                if (node) {
                    const defaultOverride = node.defaultOverrides.find(defaultOverride => {
                        const typeAndDirection = NodeRow.getTypeAndDirection(defaultOverride);

                        if (typeAndDirection) {
                            const {
                                socketType: overrideSocketType,
                                direction: overrideDirection
                            } = typeAndDirection;
    
                            return SocketType.areEqual(socketType, overrideSocketType) &&
                                direction === overrideDirection;
                        }
                    });

                    if (defaultOverride && defaultOverride.data) return NodeRow.getDefault(defaultOverride);

                    const defaultNodeRow = node.nodeRows.find(nodeRow => {
                        const typeAndDirection = NodeRow.getTypeAndDirection(nodeRow);

                        if (typeAndDirection) {
                            const {
                                socketType: nodeRowSocketType,
                                direction: nodeRowDirection
                            } = typeAndDirection;
    
                            return SocketType.areEqual(socketType, nodeRowSocketType) &&
                                direction === nodeRowDirection;
                        }
                    });

                    if (defaultNodeRow && defaultNodeRow.data) return NodeRow.getDefault(defaultNodeRow);
                    
                    return { variant: "None" };
                } else {
                    return { variant: "None" };
                }
            })
        )
    }

    getNodePropertyValue(vertexIndex: VertexIndex, propName: string): Observable<Property | undefined> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(({data: node}) => {
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
            })
        );
    }

    getNodeSocketXy(index: VertexIndex, socketType: SocketType, direction: SocketDirection): { x: number, y: number } {
        const node = this.getNode(index);

        if (!node) return { x: 0, y: 0 };

        let y = TITLE_HEIGHT;

        const rowIndex = node.nodeRows.findIndex(nodeRow => {
            const typeAndDirection = NodeRow.getTypeAndDirection(nodeRow);
            const height = NodeRow.getHeight(nodeRow);

            y += height;

            if (typeAndDirection) {
                const {
                    socketType: rowSocketType,
                    direction: rowDirection
                 } = typeAndDirection;

                return SocketType.areEqual(socketType, rowSocketType) && rowDirection === direction;
            }

            return false;
        });

        if (rowIndex === -1) return { x: 0, y: 0 };

        const relativeX = direction === SocketDirection.Output ? NODE_WIDTH : 0;
        const relativeY = (y - SOCKET_HEIGHT) + SOCKET_OFFSET;

        return { x: node.uiData.x + relativeX, y: node.uiData.y + relativeY };
    }
}
