const assert = require('node:assert/strict')
const steam = require(process.argv[2])

async function main() {
    const { workshop, cloud, matchmaking } = steam.init(3104020)
    assert.deepEqual(workshop.getSubscribedItems(), [18446744073709551614n, 42n])
    assert.deepEqual(workshop.getSubscribedItems(true), [18446744073709551614n, 42n])
    process.env.CONTENT_OVERSIZED_SUBSCRIPTIONS = '1'
    assert.throws(() => workshop.getSubscribedItems(), /100000/)
    delete process.env.CONTENT_OVERSIZED_SUBSCRIPTIONS
    await assert.rejects(workshop.startPlaytimeTracking([]), /1 to 100/)
    await assert.rejects(workshop.startPlaytimeTracking(Array(101).fill(42n)), /1 to 100/)
    await assert.rejects(workshop.stopPlaytimeTracking(Array(101).fill(42n)), /1 to 100/)
    await assert.rejects(workshop.startPlaytimeTracking([-1n]), /positive/)
    await assert.rejects(workshop.startPlaytimeTracking([1n << 64n]), /64-bit/)
    await assert.rejects(workshop.subscribe(-1n), /positive/)
    await assert.rejects(workshop.unsubscribe(0n), /positive/)
    for (const method of ['state', 'installInfo', 'downloadInfo']) {
        assert.throws(() => workshop[method](1n << 64n), /64-bit/)
    }
    assert.throws(() => workshop.download(-1n, false), /positive/)
    await workshop.startPlaytimeTracking([18446744073709551614n, 42n])
    await workshop.startPlaytimeTracking(Array(100).fill(42n))
    await assert.rejects(workshop.stopPlaytimeTracking([42n]))
    await workshop.stopPlaytimeTrackingForAllItems()
    await assert.rejects(workshop.deleteItem(42n))
    workshop.suspendDownloads(true)
    process.env.CONTENT_FAIL_ON_UPDATE_START = '1'
    await assert.rejects(workshop.updateItem(42n, { contentPath: '/definitely-missing-workshop-fixture-path' }), /Invalid workshop path/)
    delete process.env.CONTENT_FAIL_ON_UPDATE_START
    await assert.rejects(workshop.updateItem(42n, { language: 'reject' }), /Steam rejected/)
    await new Promise((resolve, reject) => {
        workshop.updateItemWithCallback(42n, { language: 'reject' }, undefined,
            () => reject(new Error('rejected setter reported success')),
            error => {
                try { assert.match(String(error), /Steam rejected/); resolve() }
                catch (error) { reject(error) }
            })
    })
    assert.equal((await workshop.createItem()).itemId, 42n)
    const update = await workshop.updateItem(42n, { language: 'german' })
    assert.equal(update.itemId, 42n)
    await new Promise((resolve, reject) => {
        workshop.updateItemWithCallback(42n, { language: 'german' }, undefined,
            result => {
                try { assert.equal(result.itemId, 42n); resolve() }
                catch (error) { reject(error) }
            }, reject)
    })
    await assert.rejects(workshop.updateItem(42n, { language: 'bad\0language' }), /NUL/)
    assert.equal(cloud.setSyncPlatforms('save', 0xffffffff), true)
    assert.equal(cloud.getSyncPlatforms('save'), 0xffffffff)
    assert.equal(cloud.setSyncPlatforms('save', 9), true)
    assert.equal(cloud.getSyncPlatforms('save'), 9)
    assert.throws(() => cloud.setSyncPlatforms('save', 128), /mask/)
    assert.equal(cloud.forgetFile('save'), true)
    assert.equal(await cloud.shareFile('save'), 18446744073709551614n)
    await assert.rejects(cloud.shareFile(''), /filename/)
    const lobby = await matchmaking.createLobby(matchmaking.LobbyType.Public, 4)
    lobby.setGameServer('192.168.1.1', 27015, 18446744073709551614n)
    assert.deepEqual(lobby.getGameServer(), { ip: '192.168.1.1', port: 27015, steamId: 18446744073709551614n })
    lobby.id = 100n
    assert.equal(lobby.id, 100n)
    assert.equal(lobby.getGameServer().ip, '192.168.1.1')
    assert.throws(() => { lobby.id = -1n }, /64-bit/)
    assert.equal(lobby.id, 100n)
    assert.throws(() => lobby.setGameServer('::1', 1), /IPv4/)
    assert.throws(() => lobby.setGameServer('1.2.3.4', 65536), /Port/)
    assert.throws(() => lobby.setGameServer('1.2.3.4', 1, -1n), /64-bit/)
    await assert.rejects(matchmaking.createLobby(matchmaking.LobbyType.Public, 251), /250/)
    const found = await matchmaking.getLobbies({
        stringFilters: [{ key: 'map', value: 'arena' }],
        numberFilters: [{ key: 'rank', value: 3, comparison: matchmaking.LobbyComparison.GreaterThan }],
        nearValueFilters: [{ key: 'rank', value: 4 }],
        count: 10,
    })
    assert.equal(found[0].id, 99n)
    const args = [1, workshop.UGCQueryType.RankedByVote, workshop.UGCType.Items, 480, 480]
    const checkRow = item => {
        assert.equal(item.publishedFileId, 18446744073709551614n)
        assert.equal(item.title, 'Fixture title')
        assert.equal(item.statistics.numSubscriptions, 7n)
    }
    const detailed = await workshop.getAllItems(...args, { onlyIds: true, searchText: 'puzzle' })
    checkRow(detailed.items[0])
    checkRow(await workshop.getItem(42n, { onlyIds: true }))
    checkRow((await workshop.getItems([42n], { onlyIds: true })).items[0])
    const userArgs = [1, 1, workshop.UserListType.Published, workshop.UGCType.Items, workshop.UserListOrder.CreationOrderAsc, {}]
    checkRow((await workshop.getUserItems(...userArgs, { onlyIds: true, cloudFileNameFilter: 'save.dat' })).items[0])
    assert.deepEqual(await workshop.getAllItemIds(...args), [18446744073709551614n])
    assert.deepEqual(await workshop.getUserItemIds(...userArgs, { cloudFileNameFilter: 'save.dat' }), [18446744073709551614n])
    assert.equal(await workshop.getAllItemTotal(...args), 9)
    await assert.rejects(workshop.getAllItemTotal(1, args[1], args[2], 481, 480))
    await assert.rejects(workshop.getAllItems(1, args[1], args[2], 481, 480))
    await assert.rejects(workshop.getAllItemIds(...args, { onlyTotal: true }), /onlyTotal/)
    await assert.rejects(workshop.getUserItemIds(...userArgs, { onlyTotal: true }), /onlyTotal/)
    await assert.rejects(workshop.getAllItemTotal(...args, { onlyIds: true }), /onlyIds/)
    console.log('content fixture passed')
}
async function timeoutCase() {
    const { cloud, workshop } = steam.init(3104020)
    if (process.env.FIXTURE_CASE === 'content-timeout-create') {
        await assert.rejects(workshop.createItem(), /Timed out waiting for create item callback/)
    } else if (process.env.FIXTURE_CASE === 'content-timeout-query') {
        await assert.rejects(workshop.getAllItems(1, workshop.UGCQueryType.RankedByVote, workshop.UGCType.Items, 480, 480, { searchText: 'puzzle' }), /Timed out waiting for workshop query callback/)
    } else {
        await assert.rejects(cloud.shareFile('save'), /Timed out waiting for cloud share callback/)
    }
    console.log(`${process.env.FIXTURE_CASE} timed out`)
}

const run = process.env.FIXTURE_CASE?.startsWith('content-timeout') ? timeoutCase : main
run().then(() => process.exit(0), error => { console.error(error); process.exit(1) })
