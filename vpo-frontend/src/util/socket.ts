import type { Index } from "../ddgg/gen_vec";
import type { VertexIndex } from "../ddgg/graph";
import type { Connection } from "../node-engine/connection";
import type { NodeWrapper, UiData } from "../node-engine/node";

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

    createNode (graphIndex: Index, type: string, uiData?: UiData) {
        this.send({
            "action": "graph/newNode",
            "payload": {
                graphIndex,
                "type": type,
                "uiData": uiData,
            }
        });
    }

    removeNode (graphIndex: Index, vertexIndex: VertexIndex) {
        this.send({
            "action": "graph/removeNode",
            "payload": {
                graphIndex,
                vertexIndex
            }
        });
    }

    updateNodes (graphIndex: Index, nodes: NodeWrapper[]) {
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodes));

        this.send({
            "action": "graph/updateNodes",
            "payload": {
                graphIndex,
                "updatedNodes": nodesToUpdateJson,
            }
        });
    }

    updateNodesUi (graphIndex: Index, nodes: NodeWrapper[]) {
        const nodesToUpdateJson = JSON.parse(JSON.stringify(nodes));

        this.send({
            "action": "graph/updateNodesUi",
            "payload": {
                graphIndex,
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
            this.ipcRenderer.send("action", {
                title: "Please select a project",
                action: "io/openLoadDialog"
            });
        }        
    }
}