import { app, BrowserWindow } from 'electron';
import path from 'path';

import client from './main/client';

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
// if (require('electron-squirrel-startup')) { // eslint-disable-line global-require
//     app.quit();
// }

const createWindow = () => {
    const mainWindow = new BrowserWindow({
        width: 1200,
        height: 800
    });
    mainWindow.loadFile(path.join(__dirname, '../public/index.html'));
    mainWindow.webContents.openDevTools();
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

client.on("message", message => {
    console.log(JSON.stringify(message, null, 4));
});
client.open();

require('electron-reload')(__dirname, {
    electron: path.join(__dirname, '../node_modules', '.bin', 'electron'),
    awaitWriteFinish: true,
});

export {}