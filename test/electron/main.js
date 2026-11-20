const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('node:path')
const steamworks = require('@kyeshmz/steamworks.ts')

function createWindow() {
    const mainWindow = new BrowserWindow({
        width: 800,
        height: 600,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            nodeIntegration: false,
            contextIsolation: true,
            sandbox: true,
        }
    })

    if (process.env.SteamTenfoot) {
        mainWindow.setFullScreen(true)
    } else {
        mainWindow.maximize()
    }

    mainWindow.webContents.openDevTools()
    mainWindow.loadFile('index.html')
}

app.whenReady().then(() => {
    const client = steamworks.init(480)
    ipcMain.handle('steam:player-name', () => client.localplayer.getName())
    ipcMain.handle('steam:example-overlay', () => client.overlay.activateToWebPage('https://www.example.com/'))
    createWindow()

    app.on('activate', function () {
        if (BrowserWindow.getAllWindows().length === 0) createWindow()
    })
})

app.on('window-all-closed', function () {
    if (process.platform !== 'darwin') app.quit()
})

steamworks.electronEnableSteamOverlay()
