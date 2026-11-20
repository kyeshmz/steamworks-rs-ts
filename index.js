const { platform, arch } = process

let nativeBinding = undefined

if (platform === 'win32' && arch === 'x64') {
    nativeBinding = require('./dist/win64/steamworksjs.win32-x64-msvc.node')
} else if (platform === 'linux' && arch === 'x64') {
    nativeBinding = require('./dist/linux64/steamworksjs.linux-x64-gnu.node')
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

module.exports.electronEnableSteamOverlay = (disableEachFrameInvalidation) => {
    const electron = require('electron')
    if (!electron) {
        throw new Error('Electron module not found')
    }

    electron.app.commandLine.appendSwitch('in-process-gpu')
    electron.app.commandLine.appendSwitch('disable-direct-composition')

    if (!disableEachFrameInvalidation) {
        const attachFrameInvalidator = (browserWindow) => {
            browserWindow.steamworksRepaintInterval = setInterval(() => {
                if (browserWindow.isDestroyed()) {
                    clearInterval(browserWindow.steamworksRepaintInterval)
                } else if (!browserWindow.webContents.isPainting()) {
                    browserWindow.webContents.invalidate()
                }
            }, 1000 / 60)
        }

        electron.BrowserWindow.getAllWindows().forEach(attachFrameInvalidator)
        electron.app.on('browser-window-created', (_, bw) => attachFrameInvalidator(bw))
    }
}

const SteamCallback = nativeBinding.callback.SteamCallback
module.exports.SteamCallback = SteamCallback
const gameServer = nativeBinding.game_server
module.exports.gameServer = gameServer
