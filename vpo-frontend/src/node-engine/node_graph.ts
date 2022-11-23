import type { NodeIndex } from "./node_index";
import { GenerationalNode, NodeRow, NodeWrapper, NODE_WIDTH, SocketValue, SOCKET_HEIGHT, SOCKET_OFFSET, TITLE_HEIGHT } from "./node";
import { InputSideConnection, OutputSideConnection, Connection, SocketType, SocketDirection } from "./connection";
import type { IPCSocket } from "../util/socket";
import { makeTaggedUnion } from "safety-match";
import { BehaviorSubject, generate, Observable } from "rxjs";
import { distinctUntilChanged, map } from "rxjs/operators";
import { deepEqual, shallowEqual } from "fast-equals";
import { match, matchOrElse } from "../util/discriminated-union";
import { Property } from "./property";

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = makeTaggedUnion({
    "Some": (node: GenerationalNode) => node, // GenerationalNode
    "None": (generation: number) => generation, // generation last held (u32)
});

export class NodeGraph {
    private nodes: (NodeWrapper | undefined)[];
    keyedNodeStore: BehaviorSubject<([string, NodeWrapper])[]>;
    keyedConnectionStore: BehaviorSubject<([string, Connection])[]>;
    nodeStore: BehaviorSubject<(NodeWrapper | undefined)[]>;
    changedNodes: NodeIndex[];
    ipcSocket: IPCSocket;
    graphIndex: number;
    selectedNodes: [];

    constructor (ipcSocket: IPCSocket, graphIndex: number) {
        this.ipcSocket = ipcSocket;

        this.nodes = [];

        this.nodeStore = new BehaviorSubject(this.nodes);
        this.keyedNodeStore = new BehaviorSubject(this.getKeyedNodes());
        this.keyedConnectionStore = new BehaviorSubject(this.getKeyedConnections());

        this.changedNodes = [];

        this.graphIndex = graphIndex;
    }

    getNode (index: NodeIndex): (NodeWrapper | undefined) {
        if (index.index >= this.nodes.length) {
            return undefined;
        }

        let node = this.nodes[index.index];

        if (node && node.index.generation === index.generation) {
            return node;
        }

        return undefined;
    }

    getNodes(): BehaviorSubject<(NodeWrapper | undefined)[]> {
        return this.nodeStore;
    }

    update() {
        this.keyedNodeStore.next(this.getKeyedNodes());
        this.keyedConnectionStore.next(this.getKeyedConnections());
        this.nodeStore.next(this.nodes);
    }

    applyJson(json: any) {
        if (this.graphIndex !== json.graphIndex) {
            throw new Error(`json being applied to wrong graph! Current graph is ${this.graphIndex}, got ${json.graphIndex}`);
        }

        this.graphIndex = json.graphIndex;

        for (let i = 0; i < json.nodes.length; i++) {
            let node: NodeWrapper | null = json.nodes[i];

            if (node === null) {
                this.nodes[i] = undefined;
                continue;
            }

            const index: NodeIndex = node.index;

            // does this node already exist?
            if (this.nodes[i] != undefined) {
                // are they not the same generation?
                if (index.generation !== this.nodes[i]?.index.generation) {
                    // in that case erase the old one
                    this.nodes[i] = undefined;
                }
            }

            this.nodes[i] = node;
        }

        this.update();
    }

    // TODO: this is very naïve and inefficient
    private getKeyedConnections (): ([string, Connection])[] {
        let keyedConnections: ([string, Connection])[] = [];

        for (let node of this.nodes) {
            if (!node) continue;

            for (let connection of node.connected_inputs) {
                let newConnection: Connection = {
                    "from_socket_type": connection.from_socket_type,
                    "from_node": connection.from_node,
                    "to_socket_type": connection.to_socket_type,
                    "to_node": node.index
                };

                keyedConnections.push([
                    this.graphIndex + "-" + Connection.getKey(newConnection),
                    newConnection
                ]);
            }
        }

        return keyedConnections;
    }

    private getKeyedNodes (): ([string, NodeWrapper])[] {
        let keyedNodes: ([string, NodeWrapper])[] = [];

        for (let i = 0; i < this.nodes.length; i++) {
            const node = this.nodes[i];

            if (node === undefined) continue;

            const generation = node.index.generation;
            const nodeWrapper = node;

            keyedNodes.push([this.graphIndex + "-" + i + "," + generation, nodeWrapper]);
        }

        return keyedNodes;
    }

    markNodeAsUpdated(index: NodeIndex) {
        console.log(`node ${index} was updated`);
        
        // don't mark it for updating if it's already been marked
        if (this.changedNodes.find(nodeIndex => nodeIndex.index === index.index && nodeIndex.generation === index.generation)) return;

        this.changedNodes.push(index);
    }

    writeChangedNodesToServer() {
        // only write changes if any nodes were marked for updating
        if (this.changedNodes.length > 0) {
            const nodesToUpdateJson = 
                JSON.parse(JSON.stringify(this.changedNodes.map(
                    (nodeIndex) => {
                        const node = this.getNode(nodeIndex);
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
                    (nodeIndex) => {
                        const node = this.getNode(nodeIndex);
                        return node;
                    }
                )));

            this.ipcSocket.updateNodesUi(this.graphIndex, nodesToUpdateJson);

            this.changedNodes.length = 0;
        }
    }

    subscribeToNode(nodeIndex: NodeIndex): Observable<NodeWrapper | undefined> {
        return this.nodeStore.pipe(
            map(nodes => {
                if (nodes && nodes[nodeIndex.index] && nodes[nodeIndex.index]?.index.generation === nodeIndex.generation) {
                    return nodes[nodeIndex.index];
                } else {
                    return undefined;
                }
            }),
            distinctUntilChanged(deepEqual)
        )
    }

    updateNode(nodeIndex: NodeIndex) {
        // TODO: naïve
        this.nodeStore.next(this.nodes);
    }

    getNodeInputConnection(nodeIndex: NodeIndex, socketType: SocketType): Observable<InputSideConnection | undefined> {
        return this.subscribeToNode(nodeIndex).pipe(
            map(node => {
                if (node && node.connected_inputs) {
                    return node.connected_inputs.find(input => SocketType.areEqual(input.to_socket_type, socketType));
                }
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeOutputConnections(nodeIndex: NodeIndex, socketType: SocketType): Observable<OutputSideConnection[]> {
        return this.subscribeToNode(nodeIndex).pipe(
            map(node => {
                if (node && node.connected_outputs) {
                    return node.connected_outputs.filter(output => output.from_socket_type === socketType);
                } else {
                    return [];
                }
            }),
            distinctUntilChanged(shallowEqual)
        );
    }

    getNodeSocketDefault(nodeIndex: NodeIndex, socketType: SocketType, direction: SocketDirection): Observable<SocketValue> {
        return this.subscribeToNode(nodeIndex).pipe(
            map(node => {
                if (node) {
                    const defaultOverride = node.default_overrides.find(defaultOverride => {
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

                    const defaultNodeRow = node.node_rows.find(nodeRow => {
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

    getNodePropertyValue(nodeIndex: NodeIndex, propName: string): Observable<Property | undefined> {
        return this.subscribeToNode(nodeIndex).pipe(
            map(node => {
                if (node) {
                    if (node.properties[propName]) return node.properties[propName];

                    const row = node.node_rows.find(nodeRow => {
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

    getNodeSocketXY(index: NodeIndex, socketType: SocketType, direction: SocketDirection): { x: number, y: number } {
        const node = this.nodes[index.index];

        if (!node) return { x: 0, y: 0 };

        const rowIndex = node.node_rows.findIndex(nodeRow => {
            const typeAndDirection = NodeRow.getTypeAndDirection(nodeRow);

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
        const relativeY = TITLE_HEIGHT + rowIndex * SOCKET_HEIGHT + SOCKET_OFFSET;

        return { x: node.ui_data.x + relativeX, y: node.ui_data.y + relativeY };
    }
}

