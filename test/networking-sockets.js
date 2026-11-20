const assert = require('node:assert/strict')
const path = require('node:path')

const binary = process.argv[2] || {
    'darwin-arm64': 'dist/osx/steamworksjs.darwin-arm64.node',
    'darwin-x64': 'dist/osx/steamworksjs.darwin-x64.node',
    'linux-x64': 'dist/linux64/steamworksjs.linux-x64-gnu.node',
    'linux-arm64': 'dist/linuxarm64/steamworksjs.linux-arm64-gnu.node',
    'win32-x64': 'dist/win64/steamworksjs.win32-x64-msvc.node',
}[`${process.platform}-${process.arch}`]
const sockets = require(path.resolve(binary)).networking_sockets

for (const port of [-1, 65536, 1.5, NaN]) {
    assert.throws(() => sockets.createListenSocketP2P(port), /virtualPort/)
}
for (const id of [0n, -1n, 1n << 64n, 7n]) {
    assert.throws(() => sockets.connectP2P(id, 0), /SteamID64/)
}
for (const address of ['localhost', '', '127.0.0.256', '[::1]:2000', 'fe80::1%en0']) {
    assert.throws(() => sockets.createListenSocketIp(2000, address), /IPv4 or IPv6/)
    assert.throws(() => sockets.connectByIpAddress(address, 2000), /IPv4 or IPv6/)
}
for (const address of ['0.0.0.0', '::', '::ffff:0.0.0.0']) {
    assert.throws(() => sockets.connectByIpAddress(address, 2000), /non-wildcard/)
}
for (const port of [0, 65536, 0.5]) {
    assert.throws(() => sockets.createListenSocketIp(port), /port/)
    assert.throws(() => sockets.connectByIpAddress('::1', port), /port/)
}
for (const method of ['acceptConnection', 'connectionInfo', 'closeConnection', 'closeListenSocket', 'destroyPollGroup', 'receiveMessagesOnPollGroup']) {
    for (const invalid of [0, 1.5, 2 ** 32]) {
        assert.throws(() => sockets[method](invalid), /handle/)
    }
}
for (const count of [0, 33, 1.5]) {
    assert.throws(() => sockets.receiveMessages(1, count), /maxMessages/)
    assert.throws(() => sockets.receiveMessagesOnPollGroup(1, count), /maxMessages/)
}
assert.throws(() => sockets.setConnectionPollGroup(0, 0), /handle/)
assert.throws(() => sockets.setConnectionPollGroup(1, -1), /group/)
assert.throws(() => sockets.sendMessage(1, Buffer.alloc(512 * 1024 + 1)), /512 KiB/)
console.log('Socket bounds verified without initializing Steam')

async function checkSelfConnection(appId, ipPort) {
    const client = require('..').init(appId)
    const api = client.networking_sockets
    const self = client.localplayer.getSteamId().steamId64
    const payload = Buffer.from([0, 255, 65])
    let listener
    let outgoing
    let incoming
    let group
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
                if (!ipPort) assert.equal(info.steamId64, self)
                api.acceptConnection(incoming)
            } catch (error) {
                failure = error
            }
        })
        listener = ipPort ? api.createListenSocketIp(ipPort) : api.createListenSocketP2P(0)
        outgoing = ipPort ? api.connectByIpAddress('127.0.0.1', ipPort) : api.connectP2P(self, 0)
        await until(() => incoming && api.connectionInfo(incoming)?.state === 3 &&
            api.connectionInfo(outgoing)?.state === 3)
        assert.equal(typeof api.sendMessage(outgoing, payload), 'bigint')
        assert.deepEqual(await until(() => api.receiveMessages(incoming, 1)[0]), payload)
        assert.deepEqual(api.receiveMessages(incoming), [])
        group = api.createPollGroup()
        assert.equal(api.setConnectionPollGroup(incoming, group), true)
        assert.equal(api.setConnectionPollGroup(outgoing, group), true)
        api.sendMessage(incoming, payload)
        assert.deepEqual(await until(() => api.receiveMessagesOnPollGroup(group, 1)[0]),
            { connection: outgoing, data: payload })
        api.sendMessage(outgoing, Buffer.alloc(0))
        assert.deepEqual(await until(() => api.receiveMessagesOnPollGroup(group)[0]),
            { connection: incoming, data: Buffer.alloc(0) })
        assert.deepEqual(api.receiveMessagesOnPollGroup(group), [])
        assert.equal(api.setConnectionPollGroup(incoming, 0), true)
        assert.equal(api.destroyPollGroup(group), true)
        group = undefined
        assert.equal(api.closeConnection(outgoing), true)
        outgoing = undefined
        await until(() => api.connectionInfo(incoming)?.state === 4)
        console.log(`Local Steam ${ipPort ? 'IP' : 'P2P'} handshake, poll group, payload, and disconnect verified`)
    } finally {
        api.setConnectionStatusHandler()
        if (group) api.destroyPollGroup(group)
        if (outgoing) api.closeConnection(outgoing)
        if (incoming) api.closeConnection(incoming)
        if (listener) api.closeListenSocket(listener)
    }
}

if (process.env.STEAM_SOCKET_SELF_TEST_APP_ID) {
    checkSelfConnection(Number(process.env.STEAM_SOCKET_SELF_TEST_APP_ID),
        Number(process.env.STEAM_SOCKET_LOOPBACK_PORT)).then(
        () => process.exit(0),
        error => { console.error(error); process.exit(1) },
    )
}
