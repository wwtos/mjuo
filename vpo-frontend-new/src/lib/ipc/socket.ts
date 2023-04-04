import type { Index } from "$lib/ddgg/gen_vec";
import type { VertexIndex } from "$lib/ddgg/graph";
import type { Connection } from "$lib/node-engine/connection";
import type { UiData } from "$lib/node-engine/node";
import type { NodeGraph } from "$lib/node-engine/node_graph";

export abstract class IpcSocket {
    abstract send(json: object): void;

    abstract onMessage(f: Function): void;

    createNode (graphIndex: Index, type: string, uiData?: UiData) {
        this.send({
            "action": "graph/newNode",
            "payload": {
                graphIndex,
                "nodeType": type,
                "uiData": uiData,
            }
        });
    }

    removeNode (graphIndex: Index, nodeIndex: VertexIndex) {
        this.send({
            "action": "graph/removeNode",
            "payload": {
                graphIndex,
                nodeIndex
            }
        });
    }

    updateNodes (graph: NodeGraph, nodes: Array<VertexIndex>) {
        const nodesToUpdate = nodes.map(index => [graph.getNode(index), index]);
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodesToUpdate));

        this.send({
            "action": "graph/updateNodes",
            "payload": {
                graphIndex: graph.graphIndex,
                "updatedNodes": nodesToUpdateJson,
            }
        });
    }

    updateNodesUi (graph: NodeGraph, nodes: Array<VertexIndex>) {
        const nodesToUpdate = nodes.map(index => [graph.getNode(index), index]);
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodesToUpdate));

        this.send({
            "action": "graph/updateNodesUi",
            "payload": {
                graphIndex: graph.graphIndex,
                "updatedNodes": nodesToUpdateJson,
            }
        });
    }

    connectNode (graphIndex: Index, connection: Connection) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/connectNode",
            "payload": {
                graphIndex,
                connection
            }
        })));
    }

    disconnectNode (graphIndex: Index, connection: Connection) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/disconnectNode",
            "payload": {
                graphIndex,
                connection
            }
        })));
    }

    requestGraph (graphIndex: Index) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/get",
            "payload": {
                graphIndex
            }
        })));
    }

    undo () {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/undo",
        })));
    }

    redo () {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/redo",
        })));
    }

    save (path?: string) {
        if (path) {
            this.send({
                "action": "io/save",
                "payload": {
                    "path": path
                }
            });
        } else {
            this.send({
                "action": "io/save",
            });
        }
    }

    load (path?: string) {
        if (path) {
            this.send({
                "action": "io/load",
                "payload": {
                    "path": path
                }
            });
        } else {
            this.send({
                "action": "io/load",
            });
        }        
    }
}

export class WebIpcSocket extends IpcSocket {
    socket: WebSocket;

    constructor (address: string | URL) {
        super();

        this.socket = new WebSocket(address);
    }

    send (json: object) {
        this.socket.send(JSON.stringify(json));
    }

    onMessage(f: Function) {
        this.socket.addEventListener("message", message => {
            f(JSON.parse(message.data));
        });
    }
}