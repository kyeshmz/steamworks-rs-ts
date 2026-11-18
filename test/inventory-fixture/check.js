const assert = require('node:assert/strict')
const { resolve } = require('node:path')
const { Worker, isMainThread, parentPort, workerData } = require('node:worker_threads')

const root = resolve(__dirname, '../../target/inventory-fixture')
const steam = require(root)

if (!isMainThread) {
    parentPort.postMessage('ready')
    Atomics.wait(workerData, 0, 0)
    steam.init(480)
    parentPort.postMessage('initialized')
} else {
    run().then(() => process.exit(0), e => { console.error(e); process.exit(1) })
}

async function run() {
    assert.throws(() => steam.init(480), /fixture first init failure/)
    const gate = new Int32Array(new SharedArrayBuffer(4))
    const workers = Array.from({ length: 3 }, () => new Worker(__filename, { workerData: gate }))
    const wait = (worker, expected) => new Promise((resolve, reject) => {
        worker.once('error', reject)
        worker.once('message', message => { assert.equal(message, expected); resolve() })
    })
    await Promise.all(workers.map(w => wait(w, 'ready')))
    const initialized = Promise.all(workers.map(w => wait(w, 'initialized')))
    Atomics.store(gate, 0, 1)
    Atomics.notify(gate, 0)
    await initialized
    const { inventory, auth, callback, cloud, voice, runtime } = steam.init()
    assert.equal(cloud.beginFileWriteBatch(), true)
    assert.equal(cloud.beginFileWriteBatch(), false)
    for (const mask of [9, 0xffffffff]) {
        assert.equal(cloud.setSyncPlatforms('cross.sav', mask), true)
        assert.equal(cloud.getSyncPlatforms('cross.sav'), mask)
    }
    assert.equal(cloud.endFileWriteBatch(), true)
    assert.deepEqual(cloud.getQuota(), { totalBytes: (1n << 64n) - 1n, availableBytes: (1n << 64n) - 2n })
    assert.equal(voice.getAvailableVoice(), 0)
    assert.deepEqual(voice.getVoice(), Buffer.alloc(0))
    assert.deepEqual(voice.decompressVoice(Buffer.from([123]), 24000), Buffer.alloc(24000, 42))
    const peer = 76561197960265729n
    for (const invalid of [0n, -1n, 1n << 64n, 1]) {
        assert.throws(() => auth.beginAuthSession(invalid, Buffer.from('bad')))
        assert.throws(() => auth.endAuthSession(invalid))
    }
    assert.throws(() => auth.beginAuthSession(peer, 'bad'))
    assert.throws(() => auth.beginAuthSession(peer, Buffer.alloc(0)), /nonempty ticket/)
    assert.throws(() => auth.beginAuthSession(peer, Buffer.from('bad')), /invalid ticket/)
    const events = []
    let listener
    await new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error('missing auth validation')), 3000)
        listener = callback.register(steam.SteamCallback.ValidateAuthTicketResponse, event => {
            events.push(event)
            if (events.length === 2) { clearTimeout(timer); resolve() }
        })
        assert.equal(auth.beginAuthSession(peer, Buffer.from([1, 0, 255])), undefined)
    })
    assert.deepEqual(events, [0, 10].map(response => ({ steam_id: peer, owner_steam_id: peer + 1n, response })))
    assert.ok(runtime.pumpStats(true).runs > 0)
    auth.endAuthSession(peer)
    listener.disconnect()
    const itemId = (1n << 64n) - 2n
    const item = { itemId, definition: 8, quantity: 2, flags: 1 }
    const props = { itemId, definition: 8, quantity: 2, properties: { name: 'Token', quantity: '65538' } }
    const all = inventory.getAllItems()
    steam.init(480)
    assert.throws(() => steam.init(481), /already initialized for App ID 480/)
    assert.deepEqual(await all, { timestamp: 1234, items: [item] })
    assert.deepEqual(await inventory.getItemsByID([itemId]), { timestamp: 1234, items: [item] })
    assert.deepEqual(await inventory.getItemProperties([itemId]), [props])
    assert.deepEqual(await inventory.getAllItemProperties(), [props])
    await assert.rejects(inventory.getItemsByID([-1n]), /unsigned 64-bit/)
    await assert.rejects(inventory.getItemsByID([1n << 64n]), /unsigned 64-bit/)
    await assert.rejects(inventory.getItemsById([1n]), /^Error: ServiceUnavailable$/)
    await assert.rejects(inventory.getItemsByID([2n]), /read inventory result items/)
    await assert.rejects(inventory.getItemsByID([3n]), /not submitted/)
    await assert.rejects(inventory.getItemProperties([4n]), /read inventory item property/)
    await assert.rejects(inventory.getItemsByID([5n]), /timed out/)
}
