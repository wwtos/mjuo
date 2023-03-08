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
    from_socket_type: SocketType,
    to_socket_type: SocketType,
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
        return Graph.getVertex(this.nodes, index)?.data;
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

        for (let [vertex, index] of Graph.verticies(json.nodes)) {
            this.nodes.verticies.vec[index.index].data[0] = vertex.data;
        }

        this.update();
    }

    // TODO: this is very naïve and inefficient
    private getKeyedConnections (): ([string, Connection])[] {
        let keyedConnections: ([string, Connection])[] = [];

        for (let [edge, index] of Graph.edges(this.nodes)) {
            let newConnection: Connection = {
                "from_node": edge.from,
                "to_node": edge.to,
                "data": edge.data
            };

            keyedConnections.push([
                this.graphIndex + "-" + Connection.getKey(newConnection),
                newConnection
            ]);
        }

        return keyedConnections;
    }

    private getKeyedNodes (): ([string, NodeWrapper, VertexIndex])[] {
        let keyedNodes: ([string, NodeWrapper, VertexIndex])[] = [];

        for (let [node, index] of Graph.verticies(this.nodes)) {
            keyedNodes.push([this.graphIndex + "-" + Index.toKey(index), node.data, index]);
        }

        console.log("here", this.nodes, Graph.verticies(this.nodes), keyedNodes);

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
            const nodesToUpdateJson = 
                JSON.parse(JSON.stringify(this.changedNodes.map(
                    (vertexIndex) => {
                        const node = this.getNode(vertexIndex);
                        return node;
                    }
                )));

            this.ipcSocket.updateNodes(this.graphIndex, nodesToUpdateJson);

            this.changedNodes.length = 0;
        }
    }

    writeChangedNodesToServerUi() {
        // only write changes if any nodes were marked for updating
        if (this.changedNodes.length > 0) {
            const nodesToUpdateJson = 
                JSON.parse(JSON.stringify(this.changedNodes.map(
                    (vertexIndex) => {
                        const node = this.getNode(vertexIndex);
                        return node;
                    }
                )));

            this.ipcSocket.updateNodesUi(this.graphIndex, nodesToUpdateJson);

            this.changedNodes.length = 0;
        }
    }

    subscribeToNode(vertexIndex: VertexIndex): Observable<NodeWrapper> {
        return this.nodeStore.pipe(
            map(nodes => {
                return nodes.find(([_, index]) => deepEqual(index, vertexIndex));
            }),
            filter(isDefined),
            map(([vertex, _]) => vertex.data),
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
                if (node && node.connectedInputs) {
                    return node.connectedInputs.find(input => SocketType.areEqual(input.to_socket_type, socketType));
                }
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeOutputConnections(vertexIndex: VertexIndex, socketType: SocketType): Observable<OutputSideConnection[]> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(node => {
                if (node && node.connectedOutputs) {
                    return node.connectedOutputs.filter(output => output.from_socket_type === socketType);
                } else {
                    return [];
                }
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeSocketDefault(vertexIndex: VertexIndex, socketType: SocketType, direction: SocketDirection): Observable<SocketValue> {
        return this.subscribeToNode(vertexIndex).pipe(
            map(node => {
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
            map(node => {
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

    getNodeSocketXY(index: VertexIndex, socketType: SocketType, direction: SocketDirection): { x: number, y: number } {
        const node = this.nodes[index.index];

        if (!node) return { x: 0, y: 0 };

        let y = TITLE_HEIGHT;

        const rowIndex = node.node_rows.findIndex(nodeRow => {
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

function filterNullish(): import("rxjs").OperatorFunction<[Vertex<NodeWrapper>, Index][] | undefined, unknown> {
    throw new Error("Function not implemented.");
}

