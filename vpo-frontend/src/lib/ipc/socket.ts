import type { Index } from "$lib/ddgg/gen_vec";
import type { VertexIndex } from "$lib/ddgg/graph";
import type { Connection } from "$lib/node-engine/connection";
import type { UiData } from "$lib/node-engine/node";
import type { NodeGraph } from "$lib/node-engine/node_graph";
import type { Action } from "$lib/node-engine/state";
import type { Engine } from "../../routes/engine";

export abstract class IpcSocket {
    abstract send(json: object): void;

    abstract onMessage(f: Function): void;

    commit (bundle: Array<Action> | Action, forceAppend?: boolean) {
        if (!Array.isArray(bundle)) {
            bundle = [bundle];
        }

        this.send({
            "action": "graph/commit",
            "payload": {
                "actions": bundle,
                "forceAppend": forceAppend || false,
            }
        })
    }

    requestGraph (graphIndex: Index) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/get",
            "payload": {
                graphIndex
            }
        })));
    }

    updateNodeState (updatedStates: Array<[VertexIndex, any]>) {
        this.send(JSON.parse(JSON.stringify({
            "action": "graph/updateNodeState",
            "payload": {
                updatedStates
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

    create () {
        this.send({
            "action": "io/create"
        });
    }

    save () {
        this.send({
            "action": "io/save",
        });
    }

    load () {
        this.send({
            "action": "io/load",
        });
    }

    importRank (fileName: string, rankName: string) {
        this.send({
            "action": "io/importRank",
            "payload": {
                fileName, rankName
            }
        })
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
