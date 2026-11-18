const assert = require('node:assert/strict')
const { randomBytes } = require('node:crypto')
const steam = require('..')

const [role, app, remote] = process.argv.slice(2)
if (!['host', 'guest', 'identity'].includes(role) || !/^[1-9]\d*$/.test(app) ||
    Number(app) > 0xffffffff || (role !== 'identity' && !/^\d{17}$/.test(remote))) {
    throw Error('Usage: node test/two-account-auth.js identity <appId> | host|guest <appId> <otherSteamId64>')
}

async function run() {
    const client = steam.init(Number(app))
    assert.ok(client.localplayer.isLoggedOn(), 'Steam must be online')
    const self = client.localplayer.getSteamId().steamId64
    if (role === 'identity') return console.log(self.toString())
    const peer = BigInt(remote)
    assert.notEqual(self, peer, 'This test requires two distinct Steam accounts')
    const net = client.networking_sockets
    let connection, listener, handler, ticket, submitted = false, authorized = false
    let phase = 'connect', failure, done = false, cancelled = false
    const nonce = randomBytes(32)
    const send = (tag, data = Buffer.alloc(0)) =>
        net.sendMessage(connection, Buffer.concat([Buffer.from([tag]), data]), true)
    const guarded = fn => event => {
        try { fn(event) } catch (err) { failure = err }
    }
    try {
        net.setConnectionStatusHandler(guarded(info => {
            if (role === 'host' && info.listenSocket === listener && info.state === 1) {
                if (connection || info.steamId64 !== peer) return net.closeConnection(info.connection)
                connection = info.connection
                net.acceptConnection(connection)
            }
            if (connection !== info.connection) return
            if (info.state === 3) phase = 'ticket'
            if (info.state === 4 || info.state === 5) {
                if (role === 'guest' && cancelled && info.state === 4) done = true
                else if (!done) throw Error(`Disconnected during ${phase}: ${info.endReason}`)
            }
        }))
        if (role === 'host') {
            handler = client.callback.register(steam.SteamCallback.ValidateAuthTicketResponse, guarded(event => {
                if (!submitted || event.steam_id !== peer) return
                if (event.response === 0) {
                    if (phase !== 'validation') return
                    authorized = true
                    phase = 'payload'
                    send(2)
                } else {
                    authorized = false
                    client.auth.endAuthSession(peer)
                    submitted = false
                    assert.equal(phase, 'revocation', 'Steam rejected the peer before payload exchange')
                    assert.equal(event.response, 6, 'Expected Steam AuthTicketCanceled')
                    assert.ok(net.closeConnection(connection))
                    connection = undefined
                    done = true
                    console.log('Host: Steam validated the peer, echoed its payload, reported ticket cancellation, and the host closed access.')
                }
            }))
            listener = net.createListenSocketP2P(0)
            console.log('Host ready for the specified peer (120 seconds).')
        } else {
            connection = net.connectP2P(peer, 0)
        }
        const deadline = Date.now() + 120_000
        while (!done) {
            if (failure) throw failure
            assert.ok(Date.now() < deadline, `Timed out during ${phase}`)
            if (role === 'guest' && phase === 'ticket') {
                phase = 'validation'
                ticket = await client.auth.getSessionTicketWithSteamId(peer, 15)
                send(1, ticket.getBytes())
            }
            if (connection && phase !== 'connect') {
                for (const frame of net.receiveMessages(connection)) {
                    const data = frame.subarray(1)
                    if (role === 'host' && phase === 'ticket' && frame[0] === 1) {
                        phase = 'validation'
                        client.auth.beginAuthSession(peer, data)
                        submitted = true
                    } else if (role === 'host' && authorized && phase === 'payload' && frame[0] === 3) {
                        assert.equal(data.length, 32)
                        send(4, data)
                        phase = 'revocation'
                    } else if (role === 'guest' && phase === 'validation' && frame[0] === 2) {
                        phase = 'echo'
                        send(3, nonce)
                    } else if (role === 'guest' && phase === 'echo' && frame[0] === 4) {
                        assert.deepEqual(data, nonce)
                        phase = 'revocation'
                        ticket.cancel()
                        cancelled = true
                    } else throw Error(`Unexpected frame during ${phase}`)
                }
            }
            await new Promise(resolve => setTimeout(resolve, 20))
        }
        if (failure) throw failure
        if (role === 'guest') console.log('Guest: verified echo, cancelled its ticket, and observed host disconnect.')
    } finally {
        if (ticket && !cancelled) ticket.cancel()
        if (submitted) client.auth.endAuthSession(peer)
        handler?.disconnect()
        net.setConnectionStatusHandler()
        if (connection) net.closeConnection(connection)
        if (listener) net.closeListenSocket(listener)
    }
}

run().then(() => process.exit(0), err => { console.error(err.message); process.exit(1) })
