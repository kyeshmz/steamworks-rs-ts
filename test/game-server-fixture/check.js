const assert = require('node:assert/strict')
const steam = require('../../target/game-server-fixture')
const server = steam.gameServer
const listing = { serverName: 'Room', mapName: 'map', maxPlayers: 8 }
const options = {
    bindIp: '127.0.0.1', gamePort: 12345, queryPort: 12346, mode: 'Authentication',
    version: '1.0', product: '480', description: 'Fixture', listing,
}
const peer = (1n << 63n) + 1n
const owner = (1n << 64n) - 1n

assert.equal(server.shutdown(), false)
assert.throws(() => server.status(), /not initialized/)
for (const override of [
    { bindIp: '::1' }, { gamePort: 2 ** 32 + 12345 }, { queryPort: 65536 },
    { queryPort: 12345 }, { gamePort: 1.5 }, { mode: 'invalid' }, { description: 'x\0y' },
    { product: '  ' }, { listing: { ...listing, mapName: 'é'.repeat(16) } },
    { listing: { ...listing, maxPlayers: NaN } },
    { listing: { ...listing, botPlayers: 9 } },
]) assert.throws(() => server.init({ ...options, ...override }))
assert.throws(() => server.init({ ...options, version: 'fail' }), /fixture init refused/)

function tick() {
    return new Promise(resolve => setImmediate(resolve))
}

async function discardListener(register) {
    const reference = new WeakRef(register())
    for (let i = 0; i < 10; i++) {
        await tick()
        global.gc()
    }
    await tick()
    assert.equal(reference.deref(), undefined, 'listener object was collected')
}

async function scenario(submit, shared, previous) {
    server.init({ ...options, queryPort: shared ? 65535 : 12346 })
    assert.throws(() => server.init(options), /already initialized/)
    assert.throws(() => steam.init(480))
    assert.deepEqual(server.status(), { state: 'Ready' })
    assert.throws(() => server.logOn('bad\0token'), /NUL-free/)
    submit()
    assert.throws(() => server.logOnAnonymous(), /already requested/)
    assert.deepEqual(server.status(), { state: 'Connecting' })
    server.runCallbacks()
    assert.deepEqual(server.status(), { state: 'Connected', steamId: (1n << 64n) - 2n })
    server.updateListing({ ...listing, mapName: 'next' })
    server.updateListing({ ...listing, mapName: 'next', botPlayers: 2, passwordProtected: true })
    server.setGameData('mode=arena')
    server.setModDir('mod')
    server.setGameTags('ranked')
    server.setKeyValue('mode', 'arena')
    server.clearAllKeyValues()
    assert.throws(() => server.setGameTags('x'.repeat(128)))
    assert.throws(() => server.setGameData('x\0y'))
    assert.throws(() => server.handleIncomingPacket(Buffer.alloc(16385), '1.2.3.4', 27015))
    if (shared) {
        assert.equal(server.handleIncomingPacket(Buffer.from([255, 255, 255, 255, 81]), '1.2.3.4', 27015), true)
        assert.deepEqual(server.getNextOutgoingPacket(), { address: '1.2.3.4', port: 27015, data: Buffer.from('reply') })
        assert.equal(server.getNextOutgoingPacket(), null)
    } else {
        assert.throws(() => server.getNextOutgoingPacket(), /Shared socket/)
    }
    server.setAdvertising(true)
    server.runCallbacks()
    assert.deepEqual(server.status(), { state: 'Disconnected' })
    server.runCallbacks()
    assert.deepEqual(server.status(), { state: 'Failed' })

    assert.deepEqual(server.diagnostics(), { appId: 480, serverRealTime: 1700000000 })
    assert.throws(() => server.p2pSessionState(1n << 64n), /nonzero unsigned/)
    const p2pRequests = []
    await discardListener(() => server.onP2pSessionRequest(event => p2pRequests.push(event)))
    assert.deepEqual(server.p2pSessionState(peer), {
        connectionActive: true, connecting: false, error: 4, usingRelay: true,
        bytesQueuedForSend: 12, packetsQueuedForSend: 2, remoteIp: '1.2.3.4', remotePort: 27015,
    })
    server.runCallbacks()
    await tick()
    assert.deepEqual(p2pRequests, [{ steamId: peer }])
    assert.equal(server.acceptP2pSession(peer), true)
    assert.equal(server.closeP2pSession(peer), true)
    const requests = []
    const oldRequestListener = server.onMessageSessionRequest(() => { throw Error('replaced request listener invoked') })
    await discardListener(() => server.onMessageSessionRequest(event => requests.push(event)))
    oldRequestListener.disconnect()
    assert.equal(server.acceptMessageSession(peer), true)
    assert.throws(() => server.sendMessageToUser(peer, Buffer.alloc(512 * 1024 + 1), 7, true), /512 KiB/)
    assert.throws(() => server.receiveMessages(7, 33), /maxMessages/)
    assert.throws(() => server.receiveMessages(-1, 1), /channel/)
    server.sendMessageToUser(peer, Buffer.from('ping'), 7, true)
    server.runCallbacks()
    await tick()
    assert.deepEqual(requests, [{ steamId: peer }])
    assert.deepEqual(server.receiveMessages(7, 2), [{ data: Buffer.from('server-message'), steamId: owner }])
    assert.deepEqual(server.receiveMessages(7, 2), [])
    assert.throws(() => server.receiveMessages(8, 2), /invalid message count/)
    assert.throws(() => server.receiveMessages(9, 2), /invalid message payload/)
    assert.equal(server.closeMessageSession(peer), true)

    assert.throws(() => server.openHostedListenSocket(-1), /virtualPort/)
    server.openHostedListenSocket(0)
    assert.throws(() => server.openHostedListenSocket(0), /already open/)
    server.runCallbacks()
    const first = server.takeHostedEvents(10)
    assert.equal(first.dropped, 0)
    assert.equal(first.events.length, 1)
    const connection = first.events[0].connection
    assert.equal(typeof connection, 'bigint')
    assert.deepEqual(first.events[0], { connection, state: 1, steamId: peer, endReason: 0 })
    if (previous) assert.throws(() => server.acceptHostedConnection(previous.connection), /Unknown hosted connection/)
    assert.throws(() => server.acceptHostedConnection(123456789n), /Unknown hosted connection/)
    server.acceptHostedConnection(connection)
    server.runCallbacks()
    assert.deepEqual(server.takeHostedEvents(10), { events: [{ connection, state: 3, steamId: peer, endReason: 0 }], dropped: 0 })
    assert.throws(() => server.sendHostedMessage(connection, Buffer.alloc(512 * 1024 + 1), true), /512 KiB/)
    assert.equal(server.sendHostedMessage(connection, Buffer.from('play'), true), 9007199254740993n)
    assert.deepEqual(server.receiveHostedMessages(connection, 2), [{ data: Buffer.from('server-message'), steamId: peer }])
    assert.deepEqual(server.receiveHostedMessages(connection, 2), [])
    if (!shared) {
        assert.equal(server.closeHostedConnection(connection), true)
        assert.throws(() => server.acceptHostedConnection(connection), /Unknown hosted connection/)
        assert.throws(() => server.sendHostedMessage(connection, Buffer.from('x'), true), /Unknown hosted connection/)
        assert.equal(server.closeHostedListenSocket(), true)
        assert.equal(server.closeHostedListenSocket(), false)
        server.openHostedListenSocket(0)
        server.runCallbacks()
        const reopened = server.takeHostedEvents(10).events[0].connection
        assert.notEqual(reopened, connection)
        assert.equal(server.closeMessageSession(peer), true)
        server.runCallbacks()
        assert.deepEqual(server.takeHostedEvents(10), { events: [{ connection: reopened, state: 4, steamId: peer, endReason: 0 }], dropped: 0 })
        assert.throws(() => server.acceptHostedConnection(reopened), /Unknown hosted connection/)
        assert.equal(server.closeHostedListenSocket(), true)
    }

    const fireAndForgetEvents = []
    await discardListener(() => server.onValidateAuthTicketResponse(event => fireAndForgetEvents.push(event)))
    server.beginAuthSession(peer, Buffer.from('ticket'))
    server.runCallbacks()
    await tick()
    assert.deepEqual(fireAndForgetEvents, [{ steam_id: peer, owner_steam_id: owner, response: 0 }])
    const events = []
    const listener = server.onValidateAuthTicketResponse(event => events.push(event))
    if (previous) previous.listener.disconnect()
    assert.throws(() => server.beginAuthSession(peer, Buffer.alloc(0)), /nonempty ticket/)
    assert.throws(() => server.beginAuthSession(0n, Buffer.from('ticket')), /nonzero unsigned 64-bit/)
    assert.throws(() => server.endAuthSession(-1n), /nonzero unsigned 64-bit/)
    assert.throws(() => server.beginAuthSession(peer, Buffer.from('reject')), /invalid ticket/)
    server.beginAuthSession(peer, Buffer.from('ticket'))
    server.runCallbacks()
    await tick()
    assert.deepEqual(events, [{ steam_id: peer, owner_steam_id: owner, response: 0 }])
    events.length = 0
    server.beginAuthSession(peer, Buffer.from('revoke'))
    server.runCallbacks()
    await tick()
    assert.deepEqual(events, [
        { steam_id: peer, owner_steam_id: owner, response: 0 },
        { steam_id: peer, owner_steam_id: owner, response: 6 },
    ])
    listener.disconnect()

    const replacement = []
    const current = server.onValidateAuthTicketResponse(event => replacement.push(event))
    listener.disconnect()
    server.beginAuthSession(peer, Buffer.from('ticket'))
    server.runCallbacks()
    await tick()
    assert.deepEqual(events, [
        { steam_id: peer, owner_steam_id: owner, response: 0 },
        { steam_id: peer, owner_steam_id: owner, response: 6 },
    ])
    assert.deepEqual(replacement, [{ steam_id: peer, owner_steam_id: owner, response: 0 }])
    current.disconnect()

    const serverEvents = []
    const outdated = server.onServerEvent(() => { throw Error('replaced listener invoked') })
    await discardListener(() => server.onServerEvent(event => serverEvents.push(event)))
    outdated.disconnect()
    assert.throws(() => server.requestUserGroupStatus(0n, owner), /nonzero unsigned/)
    assert.equal(server.requestUserGroupStatus(peer, owner), true)
    server.runCallbacks()
    await tick()
    assert.deepEqual(serverEvents, [
        { kind: 'approve', user: peer, owner },
        { kind: 'deny', user: peer, reason: 5, text: 'denied' },
        { kind: 'kick', user: peer, reason: 3 },
        { kind: 'group', user: peer, group: owner, member: true, officer: false },
    ])

    server.endAuthSession(peer)

    assert.equal(server.shutdown(), true)
    assert.equal(server.shutdown(), false)
    assert.throws(() => server.runCallbacks(), /not initialized/)
    assert.throws(() => server.diagnostics(), /not initialized/)
    assert.throws(() => server.takeHostedEvents(1), /not initialized/)
    assert.throws(() => server.p2pSessionState(peer), /not initialized/)
    assert.throws(() => server.beginAuthSession(peer, Buffer.from('ticket')), /not initialized/)
    assert.throws(() => server.endAuthSession(peer), /not initialized/)
    assert.throws(() => server.onValidateAuthTicketResponse(() => {}), /not initialized/)
    assert.throws(() => steam.init(480))
    return { listener, connection }
}

;(async () => {
    const previous = await scenario(() => server.logOnAnonymous(), false)
    await scenario(() => server.logOn('fixture-only'), true, previous)
    console.log('Dedicated server fake-SDK lifecycle and auth events passed')
})().catch(error => {
    console.error(error)
    process.exit(1)
})
