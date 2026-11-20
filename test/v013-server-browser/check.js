const assert = require('node:assert/strict')
const { Worker } = require('node:worker_threads')
const sdk = require(process.argv[2])

const platform = process.arch === 'arm64' ? 'linuxarm64' : 'linux64'
const nativePath = `${process.argv[2]}/dist/${platform}/steamworksjs.linux-${process.arch}-gnu.node`
const native = require(nativePath)
const browser = native.matchmaking_servers
assert.throws(() => new browser.ServerList(), /constructor/)
for (const id of [0, -1, 1.5, Infinity, 2 ** 32]) {
    assert.throws(() => browser.requestList(id, 'internet'), /App ID/)
}
assert.throws(() => browser.requestList(3104020, 'unknown'), /kind/)
assert.throws(() => browser.requestList(3104020, 'lan', [{key: 'x', value: 'y'}]), /LAN/)
assert.throws(() => browser.requestList(3104020, 'internet', [{key: '', value: 'x'}]), /Filter keys/)
assert.throws(() => browser.requestList(3104020, 'internet', [{key: 'x', value: '\0'}]), /NUL/)
assert.throws(() => browser.requestList(3104020, 'internet', [{key: 'é'.repeat(128), value: 'x'}]), /255/)
assert.throws(() => browser.requestList(3104020, 'internet', [{key: 'x', value: 'a'.repeat(256)}]), /255/)
assert.throws(() => browser.requestList(3104020, 'internet', Array(33).fill({key: 'x', value: 'y'})), /32/)
assert.throws(() => browser.requestList(3104020, 'internet', [{key: 'x'.repeat(255), value: 'a'.repeat(255)}]), /not initialized/)
assert.throws(() => browser.requestList(3104020, 'internet'), /not initialized/)

const client = sdk.init(3104020)
assert.equal(client.matchmaking_servers, browser)
const filters = [{key: 'map', value: 'island'}, {key: 'secure', value: '1'}]
for (const kind of ['internet', 'lan', 'friends', 'favorites', 'history', 'spectator']) {
    const list = browser.requestList(3104020, kind, kind === 'lan' ? [] : filters)
    assert.equal(list.isRefreshing(), true)
    assert.equal(list.count(), 2)
    assert.deepEqual(list.getServer(0), {
        appId: 3104020, steamId: 76561198012345678n, address: '192.0.2.1',
        queryPort: 27015, connectionPort: 27016, pingMs: -1, players: 4,
        maxPlayers: 12, botPlayers: 1, version: 1, lastTimePlayed: 0,
        successfulResponse: true, doNotRefresh: false, password: false,
        secure: true, name: 'Fixture', map: 'island', gameDir: 'game',
        description: 'Fixture game', tags: 'test'
    })
    assert.equal(list.getServer(1), null)
    assert.equal(list.getServer(2), null)
    for (const index of [-1, 0.5, Infinity, 100000]) assert.throws(() => list.getServer(index), /index/)
    assert.equal(list.isRefreshing(), false)
    assert.equal(list.release(), true)
    assert.equal(list.release(), false)
    assert.equal(list.cancel(), false)
    assert.throws(() => list.count(), /released/)
}
async function finish() {
    const pending = browser.requestList(3104020, 'internet')
    const another = browser.requestList(3104020, 'friends')
    assert.equal(another.getServer(0).version, 2)
    const saved = pending.getServer(0)
    assert.equal(pending.isRefreshing(), true)
    assert.equal(pending.cancel(), true)
    assert.equal(saved.steamId, 76561198012345678n)
    assert.equal(saved.name, 'Fixture')
    assert.throws(() => pending.isRefreshing(), /released/)
    assert.equal(another.release(), true)
    ;(() => { browser.requestList(3104020, 'internet') })()
    for (let i = 0; i < 20; i++) {
        global.gc()
        await new Promise(resolve => setImmediate(resolve))
    }
    const afterGc = browser.requestList(3104020, 'internet')
    assert.equal(afterGc.getServer(0).version, 1)
    afterGc.release()
    const worker = new Worker(`
        const { parentPort, workerData } = require('node:worker_threads')
        const native = require(workerData)
        parentPort.postMessage('ready')
        for (let i = 0; i < 150; i++) native.runCallbacks()
        parentPort.postMessage('done')
    `, { eval: true, workerData: nativePath })
    await new Promise((resolve, reject) => {
        worker.once('error', reject)
        worker.once('message', resolve)
    })
    for (let i = 0; i < 50; i++) {
        const list = browser.requestList(3104020, 'internet')
        assert.equal(list.cancel(), true)
    }
    await new Promise((resolve, reject) => {
        worker.once('error', reject)
        worker.once('message', resolve)
    })
    console.log('Server browser fixture passed')
    process.exit(0)
}
finish().catch(error => { console.error(error); process.exit(1) })
