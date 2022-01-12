const { app, BrowserWindow } = require('electron');
const path = require('path');

//require('./main/client.js');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) { // eslint-disable-line global-require
    app.quit();
}

const createWindow = () => {
    const mainWindow = new BrowserWindow({
        width: 800,
        height: 600
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

require('electron-reload')(__dirname, {
    electron: path.join(__dirname, '../node_modules', '.bin', 'electron'),
    awaitWriteFinish: true,
});
