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
}