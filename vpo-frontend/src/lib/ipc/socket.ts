import type { Index } from "$lib/ddgg/gen_vec";
import type { VertexIndex } from "$lib/ddgg/graph";
import type { Connection } from "$lib/node-engine/connection";
import type { UiData } from "$lib/node-engine/node";
import type { NodeGraph } from "$lib/node-engine/node_graph";
import type { Engine } from "../../routes/engine";

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
    messages: object[];

    constructor (address: string | URL) {
        super();

        this.socket = new WebSocket(address);
        this.socket.onopen = this.flushMessages.bind(this);
        
        this.messages = [];
    }

    send (json: object) {
        this.messages.push(json);

        this.flushMessages();
    }

    flushMessages () {
        if (this.socket.readyState === WebSocket.OPEN) {
            while (this.messages.length > 0) {
                const message = this.messages.splice(0, 1)[0];
                console.log("sending", message);

                this.socket.send(JSON.stringify(message));
            }
        }
    }

    onMessage(f: Function) {
        this.socket.addEventListener("message", message => {
            f(JSON.parse(message.data));
        });
    }
}

export class WasmIpcSocket extends IpcSocket {
    messages: object[];
    eventListeners: Function[];
    engine: Engine | undefined;

    constructor() {
        super();

        this.messages = [];
        this.eventListeners = [];
    }

    send (json: object) {
        this.messages.push({
            type: "message",
            payload: json
        });

        this.flushMessages();
    }

    sendRaw (json: object) {
        this.messages.push(json);

        this.flushMessages();
    }

    onMessage(f: Function) {
        this.eventListeners.push(f);
    }

    setEngine (engine: Engine) {
        this.engine = engine;

        engine.worklet.port.onmessage = (message) => {
            const data = JSON.parse(message.data);

            for (let message of data) {
                for (let listener of this.eventListeners) {
                    listener(message);
                }
            }
        };

        this.flushMessages();
    }

    flushMessages () {
        if (this.engine && this.engine.context.state === "running") {
            while (this.messages.length > 0) {
                const message = this.messages.splice(0, 1)[0];

                if ("type" in message && message["type"] !== "midi") {
                    console.log("sending", message);
                }   

                this.engine.send(message);
            }
        }
    }
}
