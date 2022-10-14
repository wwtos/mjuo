import { GenerationalNode, Node, NodeIndex, NodeWrapper, UiData } from "./node";
import { InputSideConnection, OutputSideConnection, Connection } from "./connection";
import { Readable, writable, Writable } from 'svelte/store';
import type { IPCSocket } from "../util/socket";
import { makeTaggedUnion } from "safety-match";
import { BehaviorSubject } from "rxjs";

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

        console.log("parsed nodes", this.nodes);

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
}

