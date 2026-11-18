const assert = require('node:assert/strict')
const steam = require('../../target/game-server-fixture')
const server = steam.gameServer
const listing = { serverName: 'Room', mapName: 'map', maxPlayers: 8 }
const options = {
    bindIp: '127.0.0.1', gamePort: 12345, queryPort: 12346, mode: 'Authentication',
    version: '1.0', product: '480', description: 'Fixture', listing,
}

assert.equal(server.shutdown(), false)
assert.throws(() => server.status(), /not initialized/)
for (const override of [
    { bindIp: '::1' }, { gamePort: 2 ** 32 + 12345 }, { queryPort: 65535 },
    { queryPort: 12345 }, { gamePort: 1.5 }, { mode: 'invalid' }, { description: 'x\0y' },
    { product: '  ' }, { listing: { ...listing, mapName: 'é'.repeat(16) } },
    { listing: { ...listing, maxPlayers: NaN } },
]) assert.throws(() => server.init({ ...options, ...override }))
assert.throws(() => server.init({ ...options, version: 'fail' }), /fixture init refused/)

for (const submit of [() => server.logOnAnonymous(), () => server.logOn('fixture-only')]) {
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
    assert.equal(server.shutdown(), true)
    assert.equal(server.shutdown(), false)
    assert.throws(() => server.runCallbacks(), /not initialized/)
    assert.throws(() => steam.init(480))
}
console.log('Dedicated server fake-SDK lifecycle passed')
