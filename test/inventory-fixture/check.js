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
    const { inventory } = steam.init()
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
