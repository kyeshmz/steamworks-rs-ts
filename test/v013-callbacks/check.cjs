const assert = require('node:assert/strict')
const steam = require(process.argv[2])
const client = steam.init(3104020)
const callbacks = steam.SteamCallback
const events = []
const handles = []

const wait = () => new Promise(resolve => setTimeout(resolve, 80))
async function emit(which) {
    client.apps.isAppInstalled(which)
    await wait()
}

async function main() {
    let old = client.callback.register(callbacks.PersonaStateChange, e => events.push(['old', e]))
    const current = client.callback.registerTyped(callbacks.PersonaStateChange, e => events.push(['new', e]))
    old.disconnect()
    old = null
    global.gc()
    await emit(1)
    assert.equal(events.length, 1)
    assert.deepEqual(events[0], ['new', { steam_id: 76561198012345678n, flags: { bits: 1 } }])
    current.disconnect()
    await emit(1)
    assert.equal(events.length, 1)

    let gcHandle = client.callback.registerTyped(callbacks.PersonaStateChange, e => events.push(['gc', e]))
    gcHandle = null
    for (let i = 0; i < 3; i++) {
        global.gc()
        await wait()
    }
    await emit(1)
    assert.equal(events.length, 2)
    assert.deepEqual(events[1], ['gc', { steam_id: 76561198012345678n, flags: { bits: 1 } }])

    const replacement = client.callback.register(callbacks.PersonaStateChange, e => events.push(['replacement', e]))
    await emit(1)
    assert.deepEqual(events[2], ['replacement', { steam_id: 76561198012345678n, flags: { bits: 1 } }])
    replacement.disconnect()
    await emit(1)
    assert.equal(events.length, 3)

    const flags = []
    const flagsHandle = client.callback.register(callbacks.PersonaStateChange, event => flags.push(event))
    await emit(23)
    await emit(24)
    assert.deepEqual(flags, [
        { steam_id: 76561198012345678n, flags: { bits: 3 } },
        { steam_id: 76561198012345678n, flags: { bits: 0 } },
    ])
    flagsHandle.disconnect()

    const cross = []
    const typedOld = client.callback.registerTyped(callbacks.PersonaStateChange, event => cross.push(['old', event]))
    const legacyNew = client.callback.register(callbacks.PersonaStateChange, event => cross.push(['new', event]))
    typedOld.disconnect()
    await emit(1)
    assert.deepEqual(cross, [['new', { steam_id: 76561198012345678n, flags: { bits: 1 } }]])
    legacyNew.disconnect()
    await emit(1)
    assert.equal(cross.length, 1)

    handles.push(client.callback.register(callbacks.LobbyCreated, e => events.push(['created', e])))
    handles.push(client.callback.register(callbacks.LobbyEnter, e => events.push(['enter', e])))
    handles.push(client.callback.register(callbacks.NewUrlLaunchParameters, e => events.push(['url', e])))
    handles.push(client.callback.register(callbacks.UserStatsReceived, e => events.push(['stats', e])))
    handles.push(client.callback.register(callbacks.UserStatsStored, e => events.push(['stored', e])))
    handles.push(client.callback.register(callbacks.UserAchievementStored, e => events.push(['achievement', e])))
    handles.push(client.callback.register(callbacks.UserAchievementIconFetched, e => events.push(['icon', e])))
    handles.push(client.callback.register(callbacks.DownloadItemResult, e => events.push(['download', e])))
    handles.push(client.callback.register(callbacks.RemotePlayConnected, e => events.push(['connected', e])))
    handles.push(client.callback.register(callbacks.RemotePlayDisconnected, e => events.push(['disconnected', e])))
    handles.push(client.callback.register(callbacks.ValidateAuthTicketResponse, e => events.push(['validation', e])))
    handles.push(client.callback.register(callbacks.RelayNetworkStatus, e => events.push(['relay', e])))
    handles.push(client.callback.register(callbacks.GlobalStatsReceived, e => events.push(['global', e])))
    handles.push(client.callback.register(callbacks.DeleteItemResult, e => events.push(['delete', e])))
    handles.push(client.callback.registerTyped(callbacks.SteamServersConnected, e => events.push(['server-connected', e])))
    handles.push(client.callback.register(callbacks.SteamServersDisconnected, e => events.push(['server-disconnected', e])))
    handles.push(client.callback.register(callbacks.SteamServerConnectFailure, e => events.push(['server-failed', e])))
    handles.push(client.callback.registerTyped(callbacks.LobbyChatUpdate, e => events.push(['chat', e])))
    handles.push(client.callback.register(callbacks.LobbyDataUpdate, e => events.push(['lobby-data', e])))
    handles.push(client.callback.register(callbacks.LobbyChatMsg, e => events.push(['message', e])))
    handles.push(client.callback.register(callbacks.P2PSessionRequest, e => events.push(['p2p-request', e])))
    handles.push(client.callback.register(callbacks.P2PSessionConnectFail, e => events.push(['p2p-fail', e])))
    handles.push(client.callback.register(callbacks.GameLobbyJoinRequested, e => events.push(['lobby-join', e])))
    handles.push(client.callback.register(callbacks.MicroTxnAuthorizationResponse, e => events.push(['transaction', e])))
    await emit(2)
    await emit(3)
    await emit(4)
    await emit(5)
    await emit(6)
    for (let i = 7; i <= 13; i++) await emit(i)
    const deletion = client.workshop.deleteItem((1n << 64n) - 2n)
    await wait()
    await emit(18)
    await deletion
    assert.equal(events.filter(([kind]) => kind === 'delete').length, 0)
    for (let i = 14; i <= 17; i++) await emit(i)
    for (let i = 19; i <= 22; i++) await emit(i)
    for (let i = 25; i <= 30; i++) await emit(i)
    assert.deepEqual(events.slice(3), [
        ['created', { result: 1, lobby: (1n << 64n) - 2n }],
        ['enter', { lobby: (1n << 64n) - 2n, chat_permissions: 0, blocked: true, chat_room_enter_response: 'Full' }],
        ['url', {}],
        ['stats', { steam_id: (1n << 64n) - 2n, game_id: (1n << 64n) - 2n, error: null }],
        ['stats', { steam_id: (1n << 64n) - 2n, game_id: (1n << 64n) - 2n, error: 'a generic failure from the steamworks API' }],
        ['stored', { game_id: (1n << 64n) - 2n, error: null }],
        ['achievement', { game_id: (1n << 64n) - 2n, achievement_name: 'ACH_TEST', current_progress: 3, max_progress: 10 }],
        ['icon', { game_id: (1n << 64n) - 2n, achievement_name: 'ACH_TEST', achieved: true, icon_handle: 17 }],
        ['download', { app_id: 480, published_file_id: (1n << 64n) - 2n, error: 'a generic failure from the steamworks API' }],
        ['connected', { session: 4294967295 }],
        ['disconnected', { session: 4294967295 }],
        ['validation', { steam_id: (1n << 64n) - 2n, owner_steam_id: (1n << 64n) - 1n, response: 6 }],
        ['relay', { availability: -102, ping_measurement_in_progress: true, network_config: 2, any_relay: 100, debug_message: 'ready' }],
        ['global', { game_id: (1n << 64n) - 2n, error: 'a generic failure from the steamworks API' }],
        ['delete', { published_file_id: (1n << 64n) - 2n, error: 'a generic failure from the steamworks API' }],
        ['relay', { availability: -101, ping_measurement_in_progress: false, network_config: -10, any_relay: 2147483647, debug_message: `x\ufffd${'x'.repeat(254)}` }],
        ['server-connected', null],
        ['server-disconnected', { reason: 'Generic' }],
        ['server-failed', { reason: 'NoConnection', still_retrying: true }],
        ['chat', { lobby: (1n << 64n) - 2n, user_changed: 76561198012345678n, making_change: 76561198012345678n, member_state_change: 'Kicked' }],
        ['lobby-data', { lobby: (1n << 64n) - 2n, member: 76561198012345678n, success: true }],
        ['message', { lobby: (1n << 64n) - 2n, user: 76561198012345678n, chat_entry_type: 'ChatMsg', chat_id: 42 }],
        ['p2p-request', { remote: 76561198012345678n }],
        ['p2p-fail', { remote: 76561198012345678n, error: 4 }],
        ['lobby-join', { lobby_steam_id: (1n << 64n) - 2n, friend_steam_id: 76561198012345678n }],
        ['transaction', { app_id: 480, order_id: (1n << 64n) - 2n, authorized: true }],
    ])
    const shapeCases = [
        [callbacks.SteamServersConnected, 19, null],
        [callbacks.SteamServersDisconnected, 20, { reason: 'Generic' }],
        [callbacks.SteamServerConnectFailure, 21, { reason: 'NoConnection', still_retrying: true }],
        [callbacks.LobbyChatUpdate, 22, { lobby: (1n << 64n) - 2n, user_changed: 76561198012345678n, making_change: 76561198012345678n, member_state_change: 'Kicked' }],
    ]
    for (const [event, trigger, expected] of shapeCases) {
        for (const method of ['register', 'registerTyped']) {
            const actual = []
            const handle = client.callback[method](event, value => actual.push(value))
            await emit(trigger)
            assert.deepEqual(actual, [expected])
            handle.disconnect()
        }
    }
    for (const handle of handles) handle.disconnect()
    console.log('Legacy and runtime-typed callbacks, replacement, disconnect and v0.13 payloads passed')
}

main().then(() => process.exit(0), error => { console.error(error); process.exit(1) })
