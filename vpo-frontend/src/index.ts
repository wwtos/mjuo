import { app, BrowserWindow, IpcMainEvent, WebContents, dialog } from 'electron';
import path from 'path';

import { open, RawMessage } from "./main/client";

import { ipcMain } from "electron";
import { match, matchOrElse } from './util/discriminated-union';

let activeWindow: BrowserWindow;

interface Reply {
    value: object,
    channel: string
}


const client = open();

let ipcSender: WebContents;
let replies: Reply[] = [];

function sendToRenderer(channel: string, value: object) {
    if (ipcSender) {
        ipcSender.send(channel, value);
    } else {
        replies.push({
            value: value,
            channel
        });
    }
}

ipcMain.on("send", (event, data) => {
    ipcSender = event.sender;

    if (replies.length > 0) {
        for (var oldReply of replies) {
            ipcSender.send(oldReply.channel, oldReply.value);
        }

        replies.length = 0;
    }

    client.sendJson(data);
});

ipcMain.on("action", (event, data) => {
    if (data?.action === "io/openSaveDialog") {
        ipcSender = event.sender;

        dialog.showOpenDialog(BrowserWindow.getFocusedWindow() as BrowserWindow, {
            properties: [ "openDirectory" ]
        }).then(({filePaths}) => {
            client.sendJson({
                "action": "io/save",
                "payload": {"path": filePaths[0]}
            });
        });
    } else if (data?.action === "io/openLoadDialog") {
        ipcSender = event.sender;

        dialog.showOpenDialog(BrowserWindow.getFocusedWindow() as BrowserWindow, {
            properties: [ "openDirectory" ]
        }).then(({filePaths}) => {
            client.sendJson({
                "action": "io/load",
                "payload": {"path": filePaths[0]}
            });
        });
    }
    
    // ipcSender.send("action", );
});

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
// if (require('electron-squirrel-startup')) { // eslint-disable-line global-require
//     app.quit();
// }

const createWindow = () => {
    const mainWindow = new BrowserWindow({
        width: 1200,
        height: 800,
        webPreferences: {
            preload: path.join(__dirname, "./main/preload.js")
        }
    });
    mainWindow.loadFile(path.join(__dirname, '../public/index.html'));
    mainWindow.webContents.openDevTools();

    activeWindow = mainWindow;
};

app.on('ready', createWindow);

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
        createWindow();
    }
});

client.on("message", (event: RawMessage) => {
    matchOrElse(
        event,
        {
            Json: ({ data }) => {
                sendToRenderer("receive", data);
            }
        },
        () => { throw "unimplemented" }
    );
});

require('electron-reload')(path.join(__dirname, "../public"), {
    electron: path.join(__dirname, '../node_modules', '.bin', 'electron'),
    awaitWriteFinish: true,
});

export {}