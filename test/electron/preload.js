const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('steam', {
    playerName: () => ipcRenderer.invoke('steam:player-name'),
    openExampleOverlay: () => ipcRenderer.invoke('steam:example-overlay'),
})
