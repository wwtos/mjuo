import type { Connection } from "../node-engine/connection";
import type { NodeWrapper, UiData } from "../node-engine/node";
import type { NodeIndex } from "../node-engine/node_index";

export class IPCSocket {
    ipcRenderer: any;

    constructor(ipcRenderer: any) {
        this.ipcRenderer = ipcRenderer;

        this.onMessage(([message]) => {
            if (message?.action === "io/getSaveLocation") {
                this.ipcRenderer.send("action", {
                    title: "Select a folder to put your project files in",
                    action: "io/openSaveDialog"
                });
            } else if (message?.action === "io/loaded") {
                location.reload();
            }
        })
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
            this.ipcRenderer.send("action", {
                title: "Please select a project",
                action: "io/openLoadDialog"
            });
        }        
    }
}