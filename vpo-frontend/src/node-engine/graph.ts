import {createEnumDefinition, EnumInstance} from "../util/enum";
import { GenerationalNode, Node, NodeIndex, NodeWrapper, UIData } from "./node";
import { SocketType, StreamSocketType, MidiSocketType, ValueSocketType, Parameter, InputSideConnection, OutputSideConnection, Connection, socketTypeToKey } from "./connection";
import { PropertyType } from "./property";
import { readable, Readable } from 'svelte/store';

// import {Node, NodeIndex, GenerationalNode} from "./node";

export const PossibleNode = createEnumDefinition({
    "Some": "object", // GenerationalNode
    "None": "number", // generation last held (u32)
});

const socketTypeLookup = {
    Stream: StreamSocketType,
    Midi: MidiSocketType,
    Value: ValueSocketType
};

function jsonToSocketType(socketTypeJson: any) {
    if (socketTypeJson.type === "MethodCall") {
        return socketTypeJson.content.map(parameter => {
            return Parameter[parameter.type](parameter.content);
        });
    } if (socketTypeJson.content.content) {
        return SocketType[socketTypeJson.type](socketTypeLookup[socketTypeJson.type][socketTypeJson.content.type](socketTypeJson.content.content));
    } else {
        return SocketType[socketTypeJson.type](socketTypeLookup[socketTypeJson.type][socketTypeJson.content.type]);
    }
}

export class Graph {
    nodes: (NodeWrapper | undefined)[];
    nodeStore: Readable<([string, NodeWrapper])[]>;
    nodeStoreSet: Function;
    connectionStore: Readable<([string, Connection])[]>;
    connectionStoreSet: Function;

    constructor () {
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
        
        this.nodeStore = readable(this.getKeyedNodes(), (set) => {
            this.nodeStoreSet = set;
        });

        this.connectionStore = readable(this.getKeyedConnections(), (set) => {
            this.connectionStoreSet = set;
        });
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

    update () {
        this.nodeStoreSet(this.getKeyedNodes());
        this.connectionStoreSet(this.getKeyedConnections());
    }

    applyJson (json: any) {
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
                    [], [], {}, new UIData({})
                );
            }

            // apply new properties
            for (var data in node.properties) {
                this.nodes[i].properties[data] = node.properties[data];
            }

            // apply new ui data
            for (var data in node.ui_data) {
                this.nodes[i].uiData[data] = node.ui_data[data];
            }

            // apply new input and output connections
            this.nodes[i].connectedInputs = node.connected_inputs.map(inputConnection => {
                return new InputSideConnection(
                    jsonToSocketType(inputConnection.from_socket_type),
                    new NodeIndex(inputConnection.from_node.index, inputConnection.from_node.generation),
                    jsonToSocketType(inputConnection.to_socket_type),
                );
            });

            this.nodes[i].connectedOutputs = node.connected_outputs.map(outputConnection => {
                return new OutputSideConnection(
                    jsonToSocketType(outputConnection.from_socket_type),
                    new NodeIndex(outputConnection.to_node.index, outputConnection.to_node.generation),
                    jsonToSocketType(outputConnection.to_socket_type),
                );
            });

            // apply node stuff
            this.nodes[i].node.inputSockets = node.node.input_sockets.map(inputSocketType => {
                return jsonToSocketType(inputSocketType);
            });

            this.nodes[i].node.outputSockets = node.node.output_sockets.map(outputSocketType => {
                return jsonToSocketType(outputSocketType);
            });

            node.node.usableProperties = {};
            for (var prop in node.node.properties) {
                this.nodes[i].node.usableProperties[prop] = PropertyType[node.node.properties[prop].type];
            }
        }

        console.log("parsed nodes", this.nodes);

        this.update();
    }

    subscribeToKeyedNodes (): Readable<([string, NodeWrapper])[]> {
        return this.nodeStore;
    }

    subscribeToKeyedConnections (): Readable<([string, Connection][])> {
        return this.connectionStore;
    }

    getKeyedConnections (): ([string, Connection])[] {
        let keyedConnections = [];

        for (let node of this.nodes) {
            for (let connection of node.connectedInputs) {
                let newConnection = new Connection(connection.fromSocketType, connection.fromNode, connection.toSocketType, node.index);

                keyedConnections.push([
                    newConnection.getKey(),
                    newConnection
                ]);
            }
        }

        return keyedConnections;
    }

    getKeyedNodes (): ([string, NodeWrapper])[] {
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

    // add_node (node: Node): NodeIndex {
    //     let index;
    //     let new_generation;

    //     if (this.nodes.length === 0) {
    //         this.nodes.push(create_new_node(node));

    //         index = this.nodes.length - 1;
    //         new_generation = 0;
    //     } else {
    //         // find an empty slot (if any)
    //     }
    // }
}

