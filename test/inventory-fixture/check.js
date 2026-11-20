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
    assert.equal(steam.isInitialized(), false)
    assert.throws(() => steam.init(480), /fixture first init failure/)
    assert.equal(steam.isInitialized(), false)
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
    assert.equal(steam.isInitialized(), true)
    const { inventory, auth, callback, cloud, voice, runtime, friends, utils, input: steamInput } = steam.init()
    const controllers = steamInput.getControllers()
    assert.equal(controllers.length, 1)
    assert.equal(controllers[0].getHandle(), (1n << 64n) - 8n)
    assert.deepEqual(controllers[0].getDigitalActionData(11n), { active: true, pressed: false })
    assert.deepEqual(controllers[0].getAnalogActionData(22n), { active: true, mode: 6, x: -0.5, y: 0.25 })
    assert.equal(steamInput.getStringForActionOrigin(494), 'SDK 1.64 origin')
    assert.throws(() => steamInput.getStringForActionOrigin(495), /action origin/)
    const input = prompt => utils.showGamepadTextInput(utils.GamepadTextInputMode.Normal, utils.GamepadTextInputLineMode.SingleLine, prompt, 20)
    const floating = width => utils.showFloatingGamepadTextInput(utils.FloatingGamepadTextInputMode.SingleLine, 2, 3, width, 100)
    const timedOut = setTimeout(() => { throw Error('Steam keyboard dismissal did not settle') }, 5000)
    assert.equal(await input('unavailable'), null)
    assert.equal(await input('cancel'), null)
    assert.equal(await input('submit'), 'Caf!')
    const dialogs = [input('held'), input('held')]
    assert.equal(await Promise.race(dialogs), null)
    steamInput.getStringForActionOrigin(493)
    assert.deepEqual((await Promise.all(dialogs)).sort(), ['Caf!', null])
    assert.equal(await floating(0), false)
    assert.equal(await floating(80), true)
    clearTimeout(timedOut)
    const fixturePeer = 76561197960265729n
    assert.equal(friends.getNickname(fixturePeer), 'Player Two')
    assert.equal(friends.getNickname(fixturePeer + 1n), null)
    assert.equal(friends.hasFriend(fixturePeer, friends.FriendFilter.Blocked), false)
    await new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(Error('missing SDK warning')), 3000)
        utils.setWarningHandler(warning => {
            clearTimeout(timer)
            try { assert.deepEqual(warning, { severity: 1, message: 'fixture warning' }); resolve() }
            catch (e) { reject(e) }
        })
        assert.equal(friends.hasFriend(fixturePeer), true)
    })
    utils.setWarningHandler()
    assert.deepEqual(friends.getFriendsGroups(), [{ id: -2, name: 'Party' }, { id: 8, name: 'Empty' }])
    assert.deepEqual(friends.getFriendsGroupMembersList(-2), [76561197960265729n])
    assert.deepEqual(friends.getFriendsGroupMembersList(8), [])
    assert.throws(() => friends.getFriendsGroupMembersList(-1), /Unknown friends group/)
    assert.throws(() => friends.getFriendsGroupMembersList(65534), /16-bit/)
    assert.throws(() => steam.gameServer.init({
        bindIp: '127.0.0.1', gamePort: 27015, queryPort: 27016,
        mode: steam.gameServer.Mode.Authentication, version: '1', product: 'test', description: 'test',
        listing: { serverName: 'test', mapName: 'test', maxPlayers: 2 },
    }), /separate processes/)
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
    await assert.rejects(auth.requestEncryptedAppTicket(Buffer.alloc(129)), /128 bytes/)
    const tickets = await Promise.all([17, 29].map(n => {
        const data = Buffer.from([n])
        const result = auth.requestEncryptedAppTicket(data)
        data.fill(0)
        return result
    }))
    assert.deepEqual(tickets, [Buffer.alloc(1025, 17), Buffer.alloc(1025, 29)])
    await assert.rejects(auth.requestEncryptedAppTicket(Buffer.from([0])), /Steam rejected/)
    for (const invalid of [0n, -1n, 1n << 64n, 1]) {
        assert.throws(() => auth.beginAuthSession(invalid, Buffer.from('bad')))
        assert.throws(() => auth.endAuthSession(invalid))
    }
    assert.throws(() => auth.beginAuthSession(peer, 'bad'))
    assert.throws(() => auth.beginAuthSession(peer, Buffer.alloc(0)), /nonempty ticket/)
    assert.throws(() => auth.beginAuthSession(peer, Buffer.from('bad')), /invalid ticket/)
    const events = []
    const deliveries = [
        ['GameOverlayActivated', [{ active: true, userInitiated: true, appId: 480 }, { active: false, userInitiated: false, appId: 480 }]],
        ['GameLobbyJoinRequested', [{ lobby_steam_id: peer + 4n, friend_steam_id: peer }]],
        ['MicroTxnAuthorizationResponse', [{ app_id: 480, order_id: (1n << 64n) - 10n, authorized: true }]],
    ].map(([name, expected]) => new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error(`missing ${name}`)), 3000)
        const seen = []
        const handle = callback.register(steam.SteamCallback[name], event => {
            seen.push(event)
            if (seen.length === expected.length) {
                clearTimeout(timer)
                handle.disconnect()
                try { assert.deepEqual(seen, expected); resolve() } catch (error) { reject(error) }
            }
        })
    }))
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
    await Promise.all(deliveries)
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
    const exchanged = { timestamp: 1234, items: [item] }
    assert.deepEqual(
        await inventory.exchangeItems([{ definition: 8, quantity: 1 }], [{ itemId, quantity: 2 }]),
        exchanged,
    )
    assert.deepEqual(
        await inventory.exchangeItemsSerialized([{ definition: 8, quantity: 1 }], [{ itemId, quantity: 1 }]),
        { ...exchanged, buffer: Buffer.from('receipt-2-10') },
    )
    assert.deepEqual(await inventory.consumeItemSerialized(itemId, 2), { ...exchanged, buffer: Buffer.from('receipt-1-11') })
    await assert.rejects(inventory.consumeItemSerialized(itemId - 1n, 1), /Could not serialize/)
    await assert.rejects(
        inventory.exchangeItemsSerialized([{ definition: 8, quantity: 1 }, { definition: 8, quantity: 1 }], []),
        /Could not serialize/,
    )
    await assert.rejects(inventory.consumeItemSerialized(1n, 1), /not submitted/)
    await assert.rejects(
        inventory.exchangeItemsSerialized([{ definition: 8, quantity: 1 }], [{ itemId: 1n, quantity: 1 }]),
        /not submitted/,
    )
    await assert.rejects(inventory.consumeItemSerialized(-1n, 1), /unsigned 64-bit/)
    await assert.rejects(inventory.consumeItemSerialized(1n << 64n, 1), /unsigned 64-bit/)
    await assert.rejects(inventory.consumeItemSerialized(itemId + 1n, 1), /sentinel/)
    await assert.rejects(inventory.consumeItemSerialized(itemId, 0), /greater than zero/)
    for (const invalid of [-1, 1.5, 4294967297, NaN]) {
        await assert.rejects(inventory.consumeItemSerialized(itemId, invalid), /greater than zero/)
    }
    await assert.rejects(inventory.exchangeItems([], []), /at least one/)
    await assert.rejects(inventory.exchangeItemsSerialized([{ definition: 8, quantity: 0 }], []), /greater than zero/)
    await assert.rejects(inventory.exchangeItemsSerialized([], [{ itemId: itemId + 1n, quantity: 1 }]), /sentinel/)
    await assert.rejects(inventory.exchangeItemsSerialized([], [{ itemId: -1n, quantity: 1 }]), /unsigned 64-bit/)
}
