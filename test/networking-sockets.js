const assert = require('node:assert/strict')
const path = require('node:path')

const binary = process.argv[2] || {
    'darwin-arm64': 'dist/osx/steamworksjs.darwin-arm64.node',
    'darwin-x64': 'dist/osx/steamworksjs.darwin-x64.node',
    'linux-x64': 'dist/linux64/steamworksjs.linux-x64-gnu.node',
    'win32-x64': 'dist/win64/steamworksjs.win32-x64-msvc.node',
}[`${process.platform}-${process.arch}`]
const sockets = require(path.resolve(binary)).networking_sockets

for (const port of [-1, 65536, 1.5, NaN]) {
    assert.throws(() => sockets.createListenSocketP2P(port), /virtualPort/)
}
for (const id of [0n, -1n, 1n << 64n, 7n]) {
    assert.throws(() => sockets.connectP2P(id, 0), /SteamID64/)
}
for (const method of ['acceptConnection', 'connectionInfo', 'closeConnection', 'closeListenSocket']) {
    for (const invalid of [0, 1.5, 2 ** 32]) {
        assert.throws(() => sockets[method](invalid), /handle/)
    }
}
for (const count of [0, 33, 1.5]) {
    assert.throws(() => sockets.receiveMessages(1, count), /maxMessages/)
}
assert.throws(() => sockets.sendMessage(1, Buffer.alloc(512 * 1024 + 1)), /512 KiB/)
console.log('Socket bounds verified without initializing Steam')

async function checkSelfConnection(appId) {
    const client = require('..').init(appId)
    const api = client.networking_sockets
    const self = client.localplayer.getSteamId().steamId64
    const payload = Buffer.from([0, 255, 65])
    let listener
    let outgoing
    let incoming
    let failure
    async function until(read) {
        const deadline = Date.now() + 10000
        while (Date.now() < deadline) {
            if (failure) throw failure
            const value = read()
            if (value) return value
            await new Promise(resolve => setTimeout(resolve, 20))
        }
        throw new Error('Timed out waiting for local Steam socket')
    }
    try {
        api.setConnectionStatusHandler(info => {
            try {
                if (info.listenSocket !== listener || info.state !== 1) return
                incoming = info.connection
                assert.equal(info.steamId64, self)
                api.acceptConnection(incoming)
            } catch (error) {
                failure = error
            }
        })
        listener = api.createListenSocketP2P(0)
        outgoing = api.connectP2P(self, 0)
        await until(() => incoming && api.connectionInfo(incoming)?.state === 3 &&
            api.connectionInfo(outgoing)?.state === 3)
        assert.equal(typeof api.sendMessage(outgoing, payload), 'bigint')
        assert.deepEqual(await until(() => api.receiveMessages(incoming, 1)[0]), payload)
        assert.deepEqual(api.receiveMessages(incoming), [])
        assert.equal(api.closeConnection(outgoing), true)
        outgoing = undefined
        await until(() => api.connectionInfo(incoming)?.state === 4)
        console.log('Local Steam P2P handshake, payload, and disconnect verified')
    } finally {
        api.setConnectionStatusHandler()
        if (outgoing) api.closeConnection(outgoing)
        if (incoming) api.closeConnection(incoming)
        if (listener) api.closeListenSocket(listener)
    }
}

if (process.env.STEAM_SOCKET_SELF_TEST_APP_ID) {
    checkSelfConnection(Number(process.env.STEAM_SOCKET_SELF_TEST_APP_ID)).then(
        () => process.exit(0),
        error => { console.error(error); process.exit(1) },
    )
}
