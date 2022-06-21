import type { Connection } from "../node-engine/connection";
import type { NodeWrapper, UiData } from "../node-engine/node";

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

    createNode (type: string, uiData?: UiData) {
        this.send({
            "action": "graph/newNode",
            "payload": {
                "type": type,
                "ui_data": uiData
            }
        });
    }

    updateNodes (nodes: NodeWrapper[]) {
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodes));

        this.send({
            "action": "graph/updateNodes",
            "payload": nodesToUpdateJson
        });
    }

    disconnectNode (connection: Connection) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/disconnectNode",
            "payload": connection
        })));
    }
}