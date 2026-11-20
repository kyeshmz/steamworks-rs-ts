const assert = require('node:assert/strict')
const path = require('node:path')
const platform = process.arch === 'arm64' ? 'linuxarm64' : 'linux64'
const native = require(path.resolve(`dist/${platform}/steamworksjs.linux-${process.arch}-gnu.node`))
const sockets = native.networking_sockets
const net = native.networking
const messages = native.networking_messages

for (const id of [0n, -76561198012345678n, 1n << 64n]) {
    assert.throws(() => net.getP2PSessionState(id), /SteamID64/)
    assert.throws(() => net.tryAcceptP2PSession(id), /SteamID64/)
    assert.throws(() => net.tryCloseP2PSession(id), /SteamID64/)
    assert.throws(() => messages.getSessionConnectionInfo(id), /SteamID64/)
}
const firstPeer = 76561198012345678n
const secondPeer = 76561198012345679n
assert.equal(net.tryAcceptP2PSession(firstPeer), true)
assert.equal(net.tryAcceptP2PSession(secondPeer), false)
assert.equal(net.tryCloseP2PSession(firstPeer), false)
assert.equal(net.tryCloseP2PSession(secondPeer), true)
assert.equal(net.acceptP2PSession(firstPeer), undefined)
assert.equal(net.acceptP2PSession(secondPeer), undefined)
assert.equal(net.closeP2PSession(firstPeer), undefined)
assert.equal(net.closeP2PSession(secondPeer), undefined)
const packet = Buffer.from([0, 255, 65])
assert.equal(net.sendP2PPacket(secondPeer, 2, packet), true)
assert.equal(net.sendP2PPacketOnChannel(secondPeer, 2, packet, 2147483647), true)
assert.equal(net.isP2PPacketAvailable(), 3)
assert.equal(net.isP2PPacketAvailableOnChannel(2147483647), 3)
for (const result of [net.readP2PPacket(5), net.readP2PPacketFromChannel(5, 2147483647)]) {
    assert.deepEqual(result.data, packet)
    assert.equal(result.size, 3)
    assert.equal(result.steamId.steamId64, secondPeer)
}
for (const channel of [-1, 0.5, NaN, 2147483648]) {
    assert.throws(() => net.sendP2PPacketOnChannel(secondPeer, 2, packet, channel), /channel/)
    assert.throws(() => net.isP2PPacketAvailableOnChannel(channel), /channel/)
    assert.throws(() => net.readP2PPacketFromChannel(5, channel), /channel/)
}
assert.deepEqual(net.getP2PSessionState(76561198012345678n), {
    connectionActive: true, connecting: false, error: 4, usingRelay: true,
    bytesQueuedForSend: 120, packetsQueuedForSend: 2, remoteIp: '192.0.2.1', remotePort: 27015,
})
assert.deepEqual(sockets.sendMessages([]), [])
assert.throws(() => sockets.sendMessages(Array.from({ length: 33 }, () => ({ connection: 1, data: Buffer.alloc(0) }))), /32 messages/)
assert.throws(() => sockets.sendMessages([{ connection: 0, data: Buffer.alloc(0) }]), /handle/)
assert.throws(() => sockets.sendMessages([{ connection: 1, data: Buffer.alloc(0), lane: 65536 }]), /lane/)
assert.throws(() => sockets.sendMessages([{ connection: 1, data: Buffer.alloc(512 * 1024 + 1) }]), /512 KiB/)
assert.throws(() => sockets.setConnectionUserData(1, 1n << 63n), /64-bit/)
assert.throws(() => sockets.setConnectionName(1, 'a\0b'), /NUL/)
assert.throws(() => sockets.realtimeConnectionStatus(1, 33), /lanes/)
assert.throws(() => sockets.configureConnectionLanes(1, []), /lanes/)
assert.throws(() => sockets.configureConnectionLanes(1, [{ priority: 1.5, weight: 1 }]), /priority/)
assert.throws(() => sockets.configureConnectionLanes(1, [{ priority: 1, weight: 0 }]), /weight/)
assert.equal(sockets.connectionUserData(123), 9007199254740993n)
assert.equal(sockets.connectionUserData(456), null)
assert.equal(sockets.setConnectionUserData(123, -9007199254740993n), true)
assert.equal(sockets.connectionName(123), 'fixture')
assert.equal(sockets.connectionName(456), null)
sockets.setConnectionName(123, 'test')
sockets.flushMessages(123)
assert.equal(sockets.initAuthentication(), 100)
assert.deepEqual(sockets.authenticationStatus(), { availability: 100, debugMessage: 'ready' })
const status = sockets.realtimeConnectionStatus(123, 2)
assert.equal(status.state, 3)
assert.equal(status.ping, 21)
assert.equal(status.lanes[0].pendingUnreliable, 64)
assert.equal(status.lanes[0].queueTimeMicroseconds, 9007199254740993n)
assert.equal(status.lanes.length, 2)
sockets.configureConnectionLanes(123, [{ priority: -1, weight: 1 }, { priority: 42, weight: 65535 }])
assert.throws(() => sockets.sendMessages([
    { connection: 1, data: Buffer.from([1]) },
    { connection: 2, data: Buffer.alloc(17) },
]), /allocate a message/)
const result = sockets.sendMessages([
    { connection: 123, data: Buffer.from([0, 255, 65]), reliable: true, lane: 7 },
    { connection: 456, data: Buffer.alloc(0), reliable: false },
])
assert.equal(result[0].messageNumber, 9007199254740993n)
assert.equal(result[0].error, undefined)
assert.equal(result[1].messageNumber, undefined)
assert.equal(result[1].error, 8n)
console.log('Networking validation and batch SDK fixture passed')

async function callbackChecks() {
    native.init(3104020)
    assert.deepEqual(messages.getSessionConnectionInfo(76561198012345678n), {
        state: 3, steamId64: 76561198012345678n, listenSocket: 99,
        endReason: 42, ping: 27, qualityLocal: 0.5, qualityRemote: 0.75,
    })
    const setters = [
        messages.setSessionRequestHandler,
        messages.setSessionFailedHandler,
        sockets.setConnectionStatusHandler,
    ]
    const seen = [0, 0, 0]
    const old = [0, 0, 0]
    setters.forEach((set, i) => set(() => old[i]++))
    native.runCallbacks()
    await new Promise(resolve => setTimeout(resolve, 20))
    assert.deepEqual(old, [1, 1, 1])
    setters.forEach((set, i) => set(() => seen[i]++))
    native.runCallbacks()
    await new Promise(resolve => setTimeout(resolve, 20))
    assert.deepEqual(old, [1, 1, 1])
    assert.deepEqual(seen, [1, 1, 1])
    setters.forEach(set => set())
    native.runCallbacks()
    await new Promise(resolve => setTimeout(resolve, 20))
    assert.deepEqual(seen, [1, 1, 1])
    setters.forEach((set, i) => set(() => seen[i]++))
    native.runCallbacks()
    await new Promise(resolve => setTimeout(resolve, 20))
    assert.deepEqual(seen, [2, 2, 2])
    setters.forEach(set => set())
    console.log('Callback replacement, removal, and re-registration passed')
}

callbackChecks().catch(error => { console.error(error); process.exitCode = 1 })
