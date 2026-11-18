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

Advertising starts disabled. Use `setAdvertising(true)` when the server is ready for players. `updateListing({ serverName, mapName, maxPlayers })` updates all three browser fields. `maxPlayers` sets advertised capacity; it does not enforce a connection limit. The wrapper does not set bot counts, host gameplay sockets, or implement player authentication. `shutdown()` drops the native handles and returns `true` once per initialized server. It does not claim a Steam logoff acknowledgment.

Use a dotted IPv4 bind address and distinct integer ports. The game port range is 1–65535 and the query port range is 1–65534. Shared-socket query mode is unsupported. Choose `NoAuthentication`, `Authentication`, or `AuthenticationAndSecure`. All text must be nonblank and NUL-free. Limits in UTF-8 bytes are 31 for map names, 63 for server names, versions, products and descriptions, and 1024 for tokens. Maximum players must be an integer from 1 to 255.
