const assert = require('assert').strict
const { init } = require('..')

const [appId, definition, property, expected] = process.argv.slice(2)
if (!appId || !definition || !property || expected === undefined) {
    throw new Error('Usage: node test/inventory-definitions.js <appId> <definitionId> <property> <expectedValue>')
}

async function run() {
    const { inventory } = init(Number(appId))
    assert.throws(() => inventory.getItemDefinitionProperty(Number(definition), 'name\0suffix'), /NUL/)
    assert.equal(inventory.loadItemDefinitions(), true)
    const deadline = Date.now() + 30_000
    while (!inventory.getItemDefinitionIds()?.includes(Number(definition))) {
        assert.ok(Date.now() < deadline, `Definition ${definition} did not load`)
        await new Promise(resolve => setTimeout(resolve, 100))
    }
    assert.ok(inventory.getItemDefinitionProperty(Number(definition)).split(',').includes(property))
    assert.equal(inventory.getItemDefinitionProperty(Number(definition), property), expected)
}

run().then(() => process.exit(0), error => {
    console.error(error)
    process.exit(1)
})
