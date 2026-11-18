# Connect peers with Steam Networking Sockets

Use `client.networking_sockets` when you need to authorize a connection before exchanging messages. Both peers must initialize Steam for the same app. Keep the JavaScript event loop running so `init()` can pump Steam callbacks.

Install a status handler before you open a listener. Check the peer's SteamID against your game's admission policy. The example admits one invited user. It rejects other incoming requests without sending a message.

```js
function listenForGuest(client, invitedSteamId64) {
    const sockets = client.networking_sockets
    let listener
    sockets.setConnectionStatusHandler(info => {
        if (info.listenSocket !== listener) return
        if (info.state === 1) {
            if (info.steamId64 === invitedSteamId64) {
                sockets.acceptConnection(info.connection)
            } else {
                sockets.closeConnection(info.connection)
            }
        } else if (info.state === 3) {
            sockets.sendMessage(info.connection, Buffer.from('welcome'))
        } else if (info.state === 4 || info.state === 5) {
            sockets.closeConnection(info.connection)
        }
    })
    listener = sockets.createListenSocketP2P(0)
    return listener
}
```

On the guest, call `connectP2P(hostSteamId64, 0)`. Pass the host's SteamID64 as a `bigint` and the same virtual port as the listener (0 to 65535). The returned numeric handle identifies the connection. It does not mean the host has accepted it.

Use `setConnectionStatusHandler` on the guest too, or poll `connectionInfo(handle)`. `connectionInfo` returns `null` for an invalid handle. A status notification describes a transition that has already happened. The peer can disconnect before you act on it, so handle errors from `acceptConnection` and `sendMessage`.

| `state` | Meaning |
| --- | --- |
| 0 | No connection |
| 1 | Connecting. A nonzero `listenSocket` identifies an incoming request. |
| 2 | Finding a route |
| 3 | Connected |
| 4 | Closed by peer. Read `endReason`, then close the handle. |
| 5 | Local problem. Read `endReason`, then close the handle. |

`sendMessage(handle, buffer, reliable?)` queues up to 512 KiB and returns the SDK message number as a `bigint`. Delivery is reliable by default. A successful send does not acknowledge delivery.

Poll `receiveMessages(handle, maxMessages?)` for an array of `Buffer` payloads on that connection. The default batch size is 10 and the allowed range is 1 to 32. An empty array means no messages are queued. An empty payload is a zero-length `Buffer`, not an empty array.

Call `closeConnection(handle, linger?)` when finished. The default is not to linger. Pass `true` to let Steam try to flush queued reliable messages. Call `closeListenSocket(listener)` when you stop hosting. This also closes connections tied to that listener. Do not reuse a closed handle. Omit the argument to `setConnectionStatusHandler()` to remove the handler. Installing another handler replaces the previous one.

To warm up Steam Datagram Relay, call `initRelayNetworkAccess()`. `relayStatus()` returns `Current` when the relay network is ready. Other possible values are `NeverTried`, `Waiting`, `Attempting`, `Unknown`, `CannotTry`, `Failed`, `Previously`, and `Retrying`. Relay readiness does not indicate that any particular peer is connected.

## Listen on an IP address

Use `createListenSocketIp(port, address?)` for direct UDP rather than Steam P2P routing. Omitting the address binds to `127.0.0.1`, so only clients on the same machine can reach it. For IPv6 loopback, pass `::1`. For remote clients, choose a local interface explicitly. A wildcard listener can expose the port on multiple interfaces.

Call `connectByIpAddress(address, port)` with the listener's literal IPv4 or IPv6 address and a port from 1 to 65535. Pass the address separately from the port, without brackets, a DNS name, or a zone ID. A wildcard is not a destination.

Use the same status handler, explicit acceptance, and close functions as for P2P. A direct IP peer might not supply a SteamID. Do not treat an absent SteamID or a network address as proof of identity.

## Read messages from several connections

Create a native poll group and assign each connection you want to read. A connection belongs to at most one group. Assignment moves it from any previous group.

```js
const group = sockets.createPollGroup()
try {
    for (const connection of connections) {
        if (!sockets.setConnectionPollGroup(connection, group)) {
            throw new Error('Cannot assign socket to poll group')
        }
    }
    for (const { connection, data } of sockets.receiveMessagesOnPollGroup(group, 32)) {
        handleMessage(connection, data)
    }
} finally {
    sockets.destroyPollGroup(group)
}
```

Each read consumes queued messages and returns an empty array if none are available. The batch limit and default match `receiveMessages`. Each entry contains the receiving connection's handle and a copied `Buffer`. Reading a message through its connection or group consumes it for both.

Pass `0` as the group to `setConnectionPollGroup` to detach one connection. `destroyPollGroup(group)` detaches all remaining members without closing their connections. Both calls return `false` for an invalid handle.
