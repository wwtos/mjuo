import { IpcRendererEvent } from "electron";

const { contextBridge, ipcRenderer } = require("electron");    

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld(
    "ipcRenderer", {
        ...ipcRenderer,
            on: function(on: string, func: (event: IpcRendererEvent, ...args: any[]) => void) {
            ipcRenderer.on(on, func);
        }
    }
);
