const assert = require('node:assert/strict')
const steam = require('..')
const appId = Number(process.argv[2])
if (!Number.isInteger(appId) || appId <= 0 || appId > 0xffffffff) {
    throw new Error('Usage: node test/inventory-reads.js <Steam App ID>')
}

async function run() {
    const { inventory } = steam.init(appId)
    const reading = inventory.getAllItems()
    assert.equal(steam.init().utils.getAppId(), appId)
    assert.equal(steam.init(appId).utils.getAppId(), appId)
    assert.throws(() => steam.init(appId === 1 ? 2 : 1), /cannot switch/)
    const { timestamp, items } = await reading
    assert.ok(Number.isInteger(timestamp) && timestamp > 0)
    assert.ok(Array.isArray(items))
    if (!items.length) {
        console.log('Empty inventory; individual instances and properties not verified')
        return
    }
    const { itemId, definition } = items[0]
    assert.equal(typeof itemId, 'bigint')
    const { items: selected } = await inventory.getItemsByID([itemId])
    assert.ok(selected.some(i => i.itemId === itemId && i.definition === definition))
    const rows = await inventory.getItemProperties([itemId])
    const row = rows.find(i => i.itemId === itemId)
    assert.equal(row.definition, definition)
    assert.ok(Object.values(row.properties).every(value => typeof value === 'string'))
    console.log(`Verified inventory reads for App ${appId}`)
}

run().then(() => process.exit(0), e => { console.error(e); process.exit(1) })
