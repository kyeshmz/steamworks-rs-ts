const { platform, arch } = process

let nativeBinding = undefined

if (platform === 'win32' && arch === 'x64') {
    nativeBinding = require('./dist/win64/steamworksjs.win32-x64-msvc.node')
} else if (platform === 'linux' && arch === 'x64') {
    nativeBinding = require('./dist/linux64/steamworksjs.linux-x64-gnu.node')
} else if (platform === 'linux' && arch === 'arm64') {
    nativeBinding = require('./dist/linuxarm64/steamworksjs.linux-arm64-gnu.node')
} else if (platform === 'darwin' && arch === 'x64') {
    nativeBinding = require('./dist/osx/steamworksjs.darwin-x64.node')
} else if (platform === 'darwin' && arch === 'arm64') {
    nativeBinding = require('./dist/osx/steamworksjs.darwin-arm64.node')
} else {
    throw new Error(`Unsupported OS: ${platform}, architecture: ${arch}`)
}

let runCallbacksInterval = undefined

module.exports.isInitialized = () => nativeBinding.isInitialized()

module.exports.init = (appId) => {
    const { init: internalInit, runCallbacks, restartAppIfNecessary, game_server, ...api } = nativeBinding

    internalInit(appId)

    clearInterval(runCallbacksInterval)
    runCallbacksInterval = setInterval(runCallbacks, 1000 / 30)

    return api
}

module.exports.restartAppIfNecessary = (appId) => nativeBinding.restartAppIfNecessary(appId);

let overlayInvalidationEnabled = false

module.exports.electronEnableSteamOverlay = (disableEachFrameInvalidation) => {
    const electron = require('electron')
    if (!electron) {
        throw new Error('Electron module not found')
    }

    electron.app.commandLine.appendSwitch('in-process-gpu')
    electron.app.commandLine.appendSwitch('disable-direct-composition')

    if (!disableEachFrameInvalidation && !overlayInvalidationEnabled) {
        overlayInvalidationEnabled = true
        const attachedWindows = new WeakSet()
        const attachFrameInvalidator = (browserWindow) => {
            if (browserWindow.isDestroyed() || attachedWindows.has(browserWindow)) return
            const webContents = browserWindow.webContents
            if (webContents.isDestroyed()) return
            attachedWindows.add(browserWindow)
            const cleanup = () => {
                clearInterval(interval)
                attachedWindows.delete(browserWindow)
                browserWindow.removeListener('closed', cleanup)
                webContents.removeListener('destroyed', cleanup)
            }
            const interval = setInterval(() => {
                if (browserWindow.isDestroyed() || webContents.isDestroyed()) {
                    cleanup()
                } else if (!webContents.isPainting()) {
                    webContents.invalidate()
                }
            }, 1000 / 60)
            browserWindow.once('closed', cleanup)
            webContents.once('destroyed', cleanup)
        }

        electron.BrowserWindow.getAllWindows().forEach(attachFrameInvalidator)
        electron.app.on('browser-window-created', (_, bw) => attachFrameInvalidator(bw))
    }
}

const SteamCallback = nativeBinding.callback.SteamCallback
module.exports.SteamCallback = SteamCallback
const gameServer = nativeBinding.game_server
module.exports.gameServer = gameServer
