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
    { bindIp: '::1' }, { gamePort: 2 ** 32 + 12345 }, { queryPort: 65535 },
    { queryPort: 12345 }, { gamePort: 1.5 }, { mode: 'invalid' }, { description: 'x\0y' },
    { product: '  ' }, { listing: { ...listing, mapName: 'é'.repeat(16) } },
    { listing: { ...listing, maxPlayers: NaN } },
]) assert.throws(() => server.init({ ...options, ...override }))
assert.throws(() => server.init({ ...options, version: 'fail' }), /fixture init refused/)

function tick() {
    return new Promise(resolve => setImmediate(resolve))
}

async function scenario(submit) {
    server.init(options)
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
    server.setAdvertising(true)
    server.runCallbacks()
    assert.deepEqual(server.status(), { state: 'Disconnected' })
    server.runCallbacks()
    assert.deepEqual(server.status(), { state: 'Failed' })

    const events = []
    const listener = server.onValidateAuthTicketResponse(event => events.push(event))
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
    server.beginAuthSession(peer, Buffer.from('ticket'))
    server.runCallbacks()
    await tick()
    assert.deepEqual(events, [
        { steam_id: peer, owner_steam_id: owner, response: 0 },
        { steam_id: peer, owner_steam_id: owner, response: 6 },
    ])
    assert.deepEqual(replacement, [{ steam_id: peer, owner_steam_id: owner, response: 0 }])
    current.disconnect()

    server.endAuthSession(peer)

    assert.equal(server.shutdown(), true)
    assert.equal(server.shutdown(), false)
    assert.throws(() => server.runCallbacks(), /not initialized/)
    assert.throws(() => server.beginAuthSession(peer, Buffer.from('ticket')), /not initialized/)
    assert.throws(() => server.endAuthSession(peer), /not initialized/)
    assert.throws(() => server.onValidateAuthTicketResponse(() => {}), /not initialized/)
    assert.throws(() => steam.init(480))
}

;(async () => {
    for (const submit of [() => server.logOnAnonymous(), () => server.logOn('fixture-only')]) {
        await scenario(submit)
    }
    console.log('Dedicated server fake-SDK lifecycle and auth events passed')
})().catch(error => {
    console.error(error)
    process.exit(1)
})
