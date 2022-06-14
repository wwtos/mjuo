import {createEnumDefinition, EnumInstance} from "../util/enum";
import { GenerationalNode, Node, NodeIndex, NodeRow, NodeWrapper, UiData } from "./node";
import { InputSideConnection, OutputSideConnection, Connection, jsonToSocketType } from "./connection";
import { Property, PropertyType } from "./property";
import { readable, Readable, writable, Writable, get } from 'svelte/store';
import { IPCSocket } from "../util/socket";

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "number", // generation last held (u32)
});

export class Graph {
    private nodes: (NodeWrapper | undefined)[];
    keyedNodeStore: Writable<([string, NodeWrapper])[]>;
    keyedConnectionStore: Writable<([string, Connection])[]>;
    nodeStore: Writable<NodeWrapper[]>;
    changedNodes: NodeIndex[];
    ipcSocket: IPCSocket;
    selectedNodes: [];

    constructor (ipcSocket: IPCSocket) {
        this.ipcSocket = ipcSocket;

        this.nodes = [/* NodeWrapper {
                    Node {

                    },
                    NodeIndex {
                        index: usize,
                        generation: u32
                    }
                },
                generation: u32
            } */];

        this.nodeStore = writable(this.nodes);
        this.keyedNodeStore = writable(this.getKeyedNodes());
        this.keyedConnectionStore = writable(this.getKeyedConnections());
        this.changedNodes = [];
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

    getNodes(): Readable<NodeWrapper[]> {
        return this.nodeStore;
    }

    update() {
        this.keyedNodeStore.set(this.getKeyedNodes());
        this.keyedConnectionStore.set(this.getKeyedConnections());
        this.nodeStore.set(this.nodes);
    }

    applyJson(json: any) {
        for (let i = 0; i < json.nodes.length; i++) {
            let node = json.nodes[i];
            var index = new NodeIndex(node.index.index, node.index.generation);

            // does this node already exist?
            if (this.nodes[i] != undefined) {
                // are they not the same generation?
                if (index.generation !== this.nodes[i].index.generation) {
                    // in that case erase the old one
                    this.nodes[i] = undefined;
                }
            }

            // if it doesn't exist, create a new one
            if (this.nodes[i] == undefined) {
                // to be populated later on
                this.nodes[i] = new NodeWrapper(
                    new Node([], [], {}),
                    index,
                    [], [], [], {}, new UiData({})
                );
            }

            // apply new properties
            for (let data in node.properties) {
                this.nodes[i].properties[data] = Property.deserialize(node.properties[data]);
            }

            // apply new ui data
            this.nodes[i].uiData.next({
                ...this.nodes[i].uiData.getValue(),
                ...node.ui_data
            });

            // apply new input and output connections
            this.nodes[i].connectedInputs.next(node.connected_inputs.map(inputConnection => {
                return new InputSideConnection(
                    jsonToSocketType(inputConnection.from_socket_type),
                    new NodeIndex(inputConnection.from_node.index, inputConnection.from_node.generation),
                    jsonToSocketType(inputConnection.to_socket_type),
                );
            }));

            this.nodes[i].connectedOutputs.next(node.connected_outputs.map(outputConnection => {
                return new OutputSideConnection(
                    jsonToSocketType(outputConnection.from_socket_type),
                    new NodeIndex(outputConnection.to_node.index, outputConnection.to_node.generation),
                    jsonToSocketType(outputConnection.to_socket_type),
                );
            }));

            // apply node stuff
            this.nodes[i].nodeRows.next(node.node_rows.map(NodeRow.deserialize));
        }

        console.log("parsed nodes", this.nodes);

        this.update();
    }

    subscribeToKeyedNodes (): Writable<([string, NodeWrapper])[]> {
        return this.keyedNodeStore;
    }

    subscribeToKeyedConnections (): Writable<([string, Connection][])> {
        return this.keyedConnectionStore;
    }

    // TODO: this is very naïve and inefficient
    private getKeyedConnections (): ([string, Connection])[] {
        let keyedConnections = [];

        for (let node of this.nodes) {
            for (let connection of node.connectedInputs.getValue()) {
                let newConnection = new Connection(connection.fromSocketType, connection.fromNode, connection.toSocketType, node.index);

                keyedConnections.push([
                    newConnection.getKey(),
                    newConnection
                ]);
            }
        }

        return keyedConnections;
    }

    private getKeyedNodes (): ([string, NodeWrapper])[] {
        let keyedNodes = [];

        for (let i = 0; i < this.nodes.length; i++) {
            if (this.nodes[i] != undefined) {
                const generation = this.nodes[i].index.generation;
                const nodeWrapper = this.nodes[i];

                keyedNodes.push([i + "," + generation, nodeWrapper]);
            }
        }

        return keyedNodes;
    }

    markNodeAsUpdated(index: NodeIndex) {
        this.changedNodes.push(index);
    }

    writeChangedNodesToServer() {
        // only write changes if any nodes were marked for updating
        if (this.changedNodes.length > 0) {
            const nodesToUpdateJson = 
                JSON.parse(JSON.stringify(this.changedNodes.map(
                    (nodeIndex) => {
                        const node = this.getNode(nodeIndex);
                        console.log("node to be serialized:", node.toJSON());
                        return node;
                    }
                )));

            this.ipcSocket.send({
                "action": "graph/updateNodes",
                "payload": nodesToUpdateJson
            });

            this.changedNodes.length = 0;
        }
    }
}

