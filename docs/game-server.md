# Dedicated game server

Run `gameServer` in its own Node process. Importing the package does not initialize Steam. The first successful client or server initialization fixes the process mode. Do not call the package's client `init()` in the server process, even after server shutdown.

Provide the game's Steam App ID through your Steam launch environment or a development `steam_appid.txt` in the working directory. `product` identifies the product to the master server; it does not select the App ID. The host also needs Valve's dedicated-server runtime for its platform.

```js
const { gameServer } = require('@kyeshmz/steamworks.ts')
const product = process.env.GAME_PRODUCT
if (!product) throw new Error('Set GAME_PRODUCT to your game’s product identifier')

gameServer.init({
    bindIp: '0.0.0.0',
    gamePort: 27015,
    queryPort: 27016,
    mode: 'Authentication',
    version: '1.0.0',
    product,
    description: 'Example Game',
    listing: { serverName: 'Practice server', mapName: 'arena', maxPlayers: 12 },
})

let timer
try {
    gameServer.logOnAnonymous()
    timer = setInterval(() => gameServer.runCallbacks(), 30)
} catch (error) {
    gameServer.shutdown()
    throw error
}

function stop() {
    clearInterval(timer)
    gameServer.shutdown()
}
process.once('SIGINT', stop)
process.once('SIGTERM', stop)
```

Use `logOn(token)` instead of `logOnAnonymous()` for a game server login token. Fetch it from your server's secret store. Do not put it in client builds or logs. Both methods submit a request, not a promise of connection. `status()` reports `Ready`, `Connecting`, `Connected`, `Disconnected`, or `Failed`. Only `Connected` includes a `steamId` bigint. Call `runCallbacks()` regularly on the server, not the client, to observe changes. To submit a new logon after a terminal failure, shut down and reinitialize.

Advertising starts disabled. Use `setAdvertising(true)` when the server is ready for players. `updateListing({ serverName, mapName, maxPlayers, botPlayers, passwordProtected })` updates browser fields. The last two fields are optional and remain unchanged when omitted. `botPlayers` must be an integer between zero and `maxPlayers`. `maxPlayers` sets advertised capacity; it does not enforce a connection limit. Use `setGameData(data)`, `setGameTags(tags)`, `setModDir(directory)`, `setKeyValue(key, value)` and `clearAllKeyValues()` for browser metadata and rules. Empty game data, mod directory and rule values are allowed; tags and keys must be nonblank. The byte limits are 2047 for game data, 127 for tags, 31 for mod directory, and 255 for rule keys and values. All must be NUL-free. Hosted SDR gameplay listeners are available as described below; the application owns any ordinary UDP socket used for shared queries. `shutdown()` unregisters callbacks and drops the native handles. It returns `true` once per initialized server. It does not claim a Steam logoff acknowledgment.

Authenticate connecting peers on the server itself. Register `onValidateAuthTicketResponse(handler)` before accepting peers. It returns a listener whose `disconnect()` stops future dispatch. A later registration replaces the current one. Disconnecting an older listener does not remove its replacement. The handler remains registered if its listener object is garbage-collected, until explicit disconnection, replacement, or server shutdown. This lifetime also applies to the other server listener setters. Already queued JavaScript callbacks may still run. The event carries `{ steam_id, owner_steam_id, response }`. Both IDs are exact `bigint` values. The license owner can differ from the peer. This event belongs to the game server. The client package's `SteamCallback.ValidateAuthTicketResponse` listens on the local client and does not see the server's peers.

Call `beginAuthSession(steamId64, ticket)` with the peer's claimed Steam ID as a nonzero unsigned 64-bit `bigint` and their session ticket as a nonempty `Buffer`. A thrown error such as `invalid ticket` means Steam rejected the submission and no session started. A normal return means Steam accepted the submission, not that Steam authorized the peer. Deny access while validation is pending, including when no event arrives. Only `response === 0` accepts the peer. Keep listening after success. Steam can revoke validation later, for example when the issuer cancels the ticket or signs in elsewhere. Remove access immediately and call `endAuthSession(steamId64)`. Also end the session when the peer disconnects or you abandon a pending request. Ending a session does not cancel the issuer's ticket. End every session before `shutdown()`; after shutdown the auth methods fail with `not initialized`.

Use `onServerEvent(handler)` for server-side `approve`, `deny`, `kick`, and `group` events. Events carry a `kind` discriminator and a `user` bigint. Approve also carries `owner`, deny carries numeric `reason` and `text`, kick carries numeric `reason`, and group carries `group`, `member`, and `officer`. Group and owner IDs are bigints. The deny reason is Steam's numeric `EDenyReason`. Use `requestUserGroupStatus(user, group)` to request a group result. A `true` return means Steam accepted the request, not that the user is a member. Keep the listener registered while awaiting the event. Registration and replacement are independent of the auth listener. Disconnect both listeners when done.

Use a dotted IPv4 bind address and distinct integer ports. The game port range is 1–65535. The query port range is 1–65534, or 65535 for Steam's shared-socket mode. In shared mode, pass incoming query packets to `handleIncomingPacket(buffer, address, port)` and repeatedly call `getNextOutgoingPacket()` until it returns `null`. Send each returned `{ address, port, data }` through the application's own UDP socket. Addresses are dotted IPv4 strings and packet ports are 1–65535. Each incoming packet must contain 1–16384 bytes. Each outgoing call returns at most one packet with at most 16384 bytes. The application owns the socket, routing, scheduling, and rate limiting. These methods reject calls when shared mode is not configured.

Choose `NoAuthentication`, `Authentication`, or `AuthenticationAndSecure`. Required configuration text must be nonblank and NUL-free. Limits in UTF-8 bytes are 31 for map names, 63 for server names, versions, products and descriptions, and 1024 for tokens. Maximum players must be an integer from 1 to 255.

## Server networking and diagnostics

These methods use the dedicated-server SDK interfaces and the existing `gameServer.runCallbacks()` pump. They do not initialize or use the local client. `diagnostics()` returns `{ appId, serverRealTime }`, where the latter is Steam's Unix time in seconds.

For legacy P2P sessions, retain the listener returned by `onP2pSessionRequest(handler)`. Each request has a `steamId` bigint. Decide whether to call `acceptP2pSession(steamId)` and later `closeP2pSession(steamId)`. `p2pSessionState(steamId)` returns connection state, numeric error, relay use, queue sizes, and an optional remote IPv4 address and port, or `null` when Steam has no session. This subset does not expose legacy P2P packet send or receive.

For Steam Networking Messages, retain the listener returned by `onMessageSessionRequest(handler)`. A request's `steamId` is omitted when the remote identity is not a Steam ID. For a recognized peer, call `acceptMessageSession(steamId)` or `closeMessageSession(steamId)`. `sendMessageToUser(steamId, data, channel, reliable)` sends a Buffer on a nonnegative signed 32-bit channel. `receiveMessages(channel, maxMessages)` returns `{ data, steamId? }` objects. The batch limit is 1–32 and each message is limited to 512 KiB. Session acceptance is an application decision, not authentication. Listener replacement and disconnection follow the same rules as auth listeners.

For Steam Datagram Relay hosted dedicated servers, set up the SDK's `SDR_LISTEN_PORT` and ticket generator separately. `openHostedListenSocket(virtualPort)` opens one server-owned hosted listener. It does not generate tickets. Virtual ports are 0–65535. Poll `takeHostedEvents(maxEvents)` after pumping callbacks. Its `events` contain `{ connection, state, steamId?, endReason }`. The connection is an opaque bigint scoped to this listener and process, not an SDK handle. State 1 requests acceptance, 2 is finding a route, 3 is connected, 4 is closed by the peer, and 5 is a local problem. Call `acceptHostedConnection(connection)` for an authorized request. Use `sendHostedMessage(connection, data, reliable)`, `receiveHostedMessages(connection, maxMessages)`, and `closeHostedConnection(connection)` for its traffic and lifecycle. Sends return a bigint message number. The same 512 KiB and 1–32 message bounds apply. Terminal events automatically close and invalidate their connection; queued inbound messages on that connection are discarded.

At most 128 hosted child connections and 256 queued hosted events are retained. Excess incoming connections are closed. If the event queue overflows, its oldest event is discarded and `takeHostedEvents` reports a nonzero `dropped` count since the previous poll. Since a dropped event may contain an unseen connection token, close and reopen the listener when recovering from overflow. `closeHostedListenSocket()` closes tracked children before the listener. `shutdown()` does the same, including when the application has not explicitly closed them. Steam closes any remaining connections belonging to a closed listener. Do not pass these tokens to the client networking API.

This is a scoped subset, not every interface exposed by upstream `Server`. Other utils operations, legacy P2P packet transport, Networking Messages channel-specific close and session-failure callbacks, and sockets operations such as IP listeners, outgoing connections, poll groups, configuration, lanes, and authentication-status APIs are not exposed here. The older server UGC and server-issued authentication-ticket interfaces are also not duplicated by this change.
