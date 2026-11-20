const assert = require('node:assert/strict')
const { EventEmitter } = require('node:events')
const Module = require('node:module')

const timers = new Map()
let nextTimer = 0
global.setInterval = (callback, delay) => {
    const id = ++nextTimer
    timers.set(id, { callback, delay })
    return id
}
global.clearInterval = (id) => timers.delete(id)

class Contents extends EventEmitter {
    constructor() {
        super()
        this.destroyed = false
        this.painting = false
        this.invalidations = 0
    }

    isDestroyed() { return this.destroyed }
    isPainting() { return this.painting }
    invalidate() {
        assert.equal(this.destroyed, false)
        this.invalidations++
    }
    destroy() {
        this.destroyed = true
        this.emit('destroyed')
    }
}

class Window extends EventEmitter {
    constructor() {
        super()
        this.destroyed = false
        this.webContents = new Contents()
    }

    isDestroyed() { return this.destroyed }
    destroy() {
        this.destroyed = true
        this.emit('closed')
    }
}

const switches = []
const app = new EventEmitter()
app.commandLine = { appendSwitch: (value) => switches.push(value) }
const windows = [new Window()]
const electron = { app, BrowserWindow: { getAllWindows: () => windows.filter((window) => !window.destroyed) } }
const originalLoad = Module._load
Module._load = function (request, ...args) {
    if (request === 'electron') return electron
    if (request.startsWith('./dist/') && request.endsWith('.node')) {
        return { callback: { SteamCallback: {} }, game_server: {} }
    }
    return originalLoad.call(this, request, ...args)
}
const { electronEnableSteamOverlay } = require('..')

const tick = () => {
    for (const { callback } of [...timers.values()]) callback()
}

electronEnableSteamOverlay(true)
assert.equal(timers.size, 0)
assert.equal(app.listenerCount('browser-window-created'), 0)
assert.deepEqual(switches, ['in-process-gpu', 'disable-direct-composition'])

electronEnableSteamOverlay()
electronEnableSteamOverlay()
tick()
const duplicateInvalidations = windows[0].webContents.invalidations
const listeners = app.listenerCount('browser-window-created')
windows[0].destroy()
tick()
const timersAfterDestroy = timers.size
console.log(`repeat setup: invalidations=${duplicateInvalidations}, listeners=${listeners}, timers after window destruction=${timersAfterDestroy}`)
assert.equal(duplicateInvalidations, 1)
assert.equal(listeners, 1)
assert.equal(timersAfterDestroy, 0)
assert.equal(Object.prototype.hasOwnProperty.call(windows[0], 'steamworksRepaintInterval'), false)
assert.equal(windows[0].listenerCount('closed'), 0)
assert.equal(windows[0].webContents.listenerCount('destroyed'), 0)

const later = new Window()
windows.push(later)
app.emit('browser-window-created', {}, later)
app.emit('browser-window-created', {}, later)
electronEnableSteamOverlay(true)
assert.equal(timers.size, 1)
assert.equal([...timers.values()][0].delay, 1000 / 60)
later.webContents.painting = true
tick()
assert.equal(later.webContents.invalidations, 0)
later.webContents.painting = false
tick()
assert.equal(later.webContents.invalidations, 1)
later.webContents.destroy()
assert.equal(timers.size, 0)
assert.equal(later.listenerCount('closed'), 0)
assert.equal(later.webContents.listenerCount('destroyed'), 0)
tick()
assert.equal(later.webContents.invalidations, 1)

const last = new Window()
windows.push(last)
app.emit('browser-window-created', {}, last)
tick()
assert.equal(last.webContents.invalidations, 1)
last.destroy()
assert.equal(timers.size, 0)

const silentlyDestroyed = new Window()
windows.push(silentlyDestroyed)
app.emit('browser-window-created', {}, silentlyDestroyed)
silentlyDestroyed.webContents.destroyed = true
tick()
assert.equal(timers.size, 0)
assert.equal(silentlyDestroyed.webContents.invalidations, 0)
Module._load = originalLoad
console.log('Electron overlay lifecycle verified')
