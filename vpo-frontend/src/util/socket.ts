import type { Connection } from "../node-engine/connection";
import type { NodeIndex, NodeWrapper, UiData } from "../node-engine/node";

export class IPCSocket {
    ipcRenderer: any;

    constructor(ipcRenderer: any) {
        this.ipcRenderer = ipcRenderer;
    }

    send(json: object) {
        console.log("sending", json);
        this.ipcRenderer.send("send", json);
    }

    onMessage(f: Function) {
        this.ipcRenderer.on("receive", function(_: object, message: object) {
            f(message);
        });
    }

    createNode (graphIndex: number, type: string, uiData?: UiData) {
        this.send({
            "action": "graph/newNode",
            "payload": {
                graphIndex,
                "type": type,
                "ui_data": uiData,
            }
        });
    }

    removeNode (graphIndex: number, nodeIndex: NodeIndex) {
        this.send({
            "action": "graph/removeNode",
            "payload": {
                graphIndex,
                nodeIndex
            }
        });
    }

    updateNodes (graphIndex: number, nodes: NodeWrapper[]) {
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodes));

        this.send({
            "action": "graph/updateNodes",
            "payload": {
                graphIndex,
                "updatedNodes": nodesToUpdateJson,
            }
        });
    }

    updateNodesUi (graphIndex: number, nodes: NodeWrapper[]) {
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodes));

        this.send({
            "action": "graph/updateNodesUi",
            "payload": {
                graphIndex,
                "updatedNodes": nodesToUpdateJson,
            }
        });
    }

    connectNode (graphIndex: number, connection: Connection) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/connectNode",
            "payload": {
                graphIndex,
                connection
            }
        })));
    }

    disconnectNode (graphIndex: number, connection: Connection) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/disconnectNode",
            "payload": {
                graphIndex,
                connection
            }
        })));
    }

    requestGraph (graphIndex: number) {
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
}