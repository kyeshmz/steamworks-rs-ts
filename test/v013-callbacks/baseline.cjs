const assert = require('node:assert/strict')
const steam = require(process.argv[2])
const client = steam.init(3104020)
const callback = steam.SteamCallback
const events = []
const wait = () => new Promise(resolve => setTimeout(resolve, 80))

async function emit(which) {
    client.apps.isAppInstalled(which)
    await wait()
}

async function main() {
    let persona = client.callback.register(callback.PersonaStateChange, event => events.push(['persona', event]))
    persona = null
    for (let i = 0; i < 3; i++) {
        global.gc()
        await wait()
    }
    const handles = [
        client.callback.register(callback.SteamServersConnected, event => events.push(['connected', event])),
        client.callback.register(callback.SteamServersDisconnected, event => events.push(['disconnected', event])),
        client.callback.register(callback.SteamServerConnectFailure, event => events.push(['failure', event])),
        client.callback.register(callback.LobbyChatUpdate, event => events.push(['chat', event])),
        client.callback.register(callback.LobbyDataUpdate, event => events.push(['lobby-data', event])),
        client.callback.register(callback.P2PSessionRequest, event => events.push(['p2p-request', event])),
        client.callback.register(callback.P2PSessionConnectFail, event => events.push(['p2p-fail', event])),
        client.callback.register(callback.GameLobbyJoinRequested, event => events.push(['lobby-join', event])),
        client.callback.register(callback.MicroTxnAuthorizationResponse, event => events.push(['transaction', event])),
    ]
    await emit(1)
    for (let i = 19; i <= 22; i++) await emit(i)
    await emit(23)
    await emit(24)
    for (let i = 25; i <= 30; i++) await emit(i)
    assert.deepEqual(events, [
        ['persona', { steam_id: 76561198012345678n, flags: { bits: 1 } }],
        ['connected', null],
        ['disconnected', { reason: 'Generic' }],
        ['failure', { reason: 'NoConnection', still_retrying: true }],
        ['chat', { lobby: (1n << 64n) - 2n, user_changed: 76561198012345678n, making_change: 76561198012345678n, member_state_change: 'Kicked' }],
        ['persona', { steam_id: 76561198012345678n, flags: { bits: 3 } }],
        ['persona', { steam_id: 76561198012345678n, flags: { bits: 0 } }],
        ['lobby-data', { lobby: (1n << 64n) - 2n, member: 76561198012345678n, success: true }],
        ['p2p-request', { remote: 76561198012345678n }],
        ['p2p-fail', { remote: 76561198012345678n, error: 4 }],
        ['lobby-join', { lobby_steam_id: (1n << 64n) - 2n, friend_steam_id: 76561198012345678n }],
        ['transaction', { app_id: 480, order_id: (1n << 64n) - 2n, authorized: true }],
    ])
    for (const handle of handles) handle.disconnect()
    console.log('Baseline 0.11 persona GC and legacy payloads passed')
}

main().then(() => process.exit(0), error => { console.error(error); process.exit(1) })
