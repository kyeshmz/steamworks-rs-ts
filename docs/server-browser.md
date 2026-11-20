# Steam server browser

`client.matchmaking_servers.requestList(appId, kind, filters)` starts an asynchronous Steam list request and returns a `ServerList`. Kinds are `internet`, `lan`, `friends`, `favorites`, `history`, and `spectator`. The App ID must be an integer from 1 through 4294967295. Filters are optional `{ key, value }` pairs. At most 32 filters are accepted. Keys must contain 1 to 255 UTF-8 bytes and values at most 255 bytes. Neither may contain NUL. LAN does not accept filters.

Poll `list.isRefreshing()` while the package callback pump runs. `list.count()` returns Steam's current count. `list.getServer(index)` copies the indexed result into a JavaScript object, or returns null if the index is beyond the current count or Steam has no details for it. Valid indices are integers from 0 through 99999. A server's `steamId` is a `bigint`, and copied results remain usable after the request is released. `pingMs` preserves negative values reported by Steam.

Call `list.release()` after reading results. `list.cancel()` also releases the request, including an active request. Both return true once and false on later calls. After release, polling, count and detail access throw. The native finalizer also releases a forgotten request, but callers should release explicitly. These methods are synchronous and do not wait for the network. Application timeouts should cancel a request rather than leave it running indefinitely.

```js
const list = client.matchmaking_servers.requestList(appId, 'favorites')
try {
  const deadline = Date.now() + 15000
  while (list.isRefreshing()) {
    if (Date.now() >= deadline) throw new Error('Server list timed out')
    await new Promise(resolve => setTimeout(resolve, 100))
  }
  const results = []
  for (let i = 0; i < Math.min(list.count(), 100000); i++) {
    const server = list.getServer(i)
    if (server) results.push(server)
  }
} finally {
  list.release()
}
```

The API reports the list's refreshing state and copied details. It does not expose per-server response callbacks or the refresh-completion response code, so an empty list cannot distinguish all Steam failure reasons. Ping, player-details queries, rules queries, individual-server refresh and repeat refresh of an existing list are not exposed.

## Ownership and ABI

The stock `steamworks` 0.13.1 wrapper does not expose a list cancellation handle and refuses to release while refreshing. This addon instead owns the SDK handle and calls `ReleaseRequest`, which the Valve header specifies cancels an active request. A stateless C++ subclass of `ISteamMatchmakingServerListResponse` has process lifetime and handles all response methods without retaining request state. It is compiled against the unmodified Valve headers from `steamworks-sys` 0.13.0. Those headers retain their Valve copyright notices.

All addon list operations and releases take the same lock as the client callback pump. Results are copied before releasing that lock. The addon does not make Rust-defined C++ virtual tables. It does not use the upstream wrapper's unchecked `get_server_details` conversion.

`sh test/v013-server-browser/run.sh` builds an offline Linux x64 stand-in using the same SDK headers. It invokes the compiled callback object's real C++ virtual methods and exercises the public Node API, argument validation, copied results, completion, cancellation and repeated release. No live Steam request is made. Windows and macOS execution and real Steam behavior remain unverified by this fixture.
