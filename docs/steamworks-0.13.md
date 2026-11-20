# Steamworks 0.13 in the Node addon

The package uses the crates.io releases `steamworks = 0.13.1` and `steamworks-sys = 0.13.0`, locked in `Cargo.lock`. The latter supplies the generated native bindings and Valve SDK 1.64. These version numbers are independent of the npm package version.

`build.js` reads the locked crate metadata, checks the target's redistributables against the copies in `sdk/redistributable_bin`, and builds and packages with the same SDK. Copy the native addon and Valve library from the **same build**. In particular, SDK 1.64 needs Apps v009, Friends v018 and UserStats v013. The older SDK packaged with `steamworks.js@0.4.0` is not an interchangeable shared library.

## Existing callers

The root import, relaunch check, `init`, and the [Deviation Game swap procedure](deviation-game.md) stay the same. Steam now initializes local user stats itself; there is no `RequestCurrentStats` call in SDK 1.64. A stats operation can still fail if Steam has no data or the stat is not configured.

The callback pump still runs in the addon. In 0.13 dropping a Rust `CallbackHandle` unregisters its event type, so the addon owns the native registration. As before, a caller may ignore the return value of `callback.register`; keep the JS handle if you want to call `disconnect()` later. A new listener of the same kind replaces the old one. Gamepad keyboard calls own their dismissal handles internally. The full-screen call returns the submitted UTF-8 string or `null`; the floating call returns `true` on dismissal and `false` if it cannot open. A second request of the same kind cannot replace one already waiting.

Steam Input accepts SDK 1.64 action origins `0` through `494`. `Controller.getDigitalActionData(handle)` returns `{ pressed, active }`; `getAnalogActionData(handle)` returns `{ x, y, mode, active }`. The existing `isDigitalActionPressed` and `getAnalogActionVector` remain available. Friends may now report the persona state `"invisible"`. `friends.hasFriend(id, filter?)` checks a relationship and `friends.getNickname(id)` returns the local nickname or `null`. `utils.setWarningHandler(fn)` receives `{ severity, message }` for SDK warnings; omit the function to clear it.

## What changed upstream and where to find it

This is a comparison with the [previous Rust pin](https://github.com/Noxime/steamworks-rs/commit/fbb79635b06b4feea8261e5ca3e8ea3ef42facf9), not a count of every function in Valve's C API. The [0.13.1 source](https://github.com/Noxime/steamworks-rs/tree/bfc8210d854f80e1483e153fb2f0a1430bb63db8/src) changes 25 Rust source files. The public signatures are in `client.d.ts`, with event payloads in `callbacks.d.ts`.

| Changed upstream area | JavaScript counterpart |
|---|---|
| `lib`, `error`, `callback` | `init`, the existing callback pump, `callback.register` and handler-specific setters. Rust's manager types, callback traits, raw dispatch and mutex changes are implementation details. Old `SteamCallback` numbers stay fixed. |
| `app`, `friends`, `user`, `utils` | `apps.launchQueryParam`, `friends.getRichPresence`, `overlay.activateInviteDialogConnectString`, `utils.isSteamInBigPictureMode`, `localplayer.getSteamIdDetails` and `localplayer.userHasLicenseForApp`. Ticket requests retain separate handle-specific waiters. |
| `input`, `screenshots`, `remote_play` | Controller origins and action data, `ScreenshotRequested`/`ScreenshotReady`, and [Remote Play sessions and invitations](remote-play.md). |
| `matchmaking`, `matchmaking_servers` | Lobby filters, member data, `Lobby.setGameServer`/`getGameServer`, `LobbyEnter`/`LobbyCreated`, and the [six server-browser lists](server-browser.md). Server details are copied while the request is owned; release a list when finished. |
| `networking`, `networking_messages` | Channel-specific P2P reads and writes, `networking.getP2PSessionState`, `tryAcceptP2PSession`/`tryCloseP2PSession`, and `networking_messages.getSessionConnectionInfo`. The existing void accept/close methods keep their return values. |
| `networking_sockets`, `networking_sockets_callback`, `networking_types`, `networking_utils` | `networking_sockets` exposes batch sends, bounded receives, poll groups, connection status and metadata, lane configuration, authentication and relay status. Use callbacks or returned buffers rather than Rust's iterators, caller-managed allocation and ownership transfer. `sendMessages` returns one result per message. |
| `remote_storage`, `ugc` | `cloud.getSyncPlatforms`/`setSyncPlatforms`, `shareFile` and `forgetFile`; `workshop` supports playtime tracking, update language, locally disabled subscriptions, delete results, and ID/total queries. |
| `user_stats`, `user_stats/stat_callback`, `user_stats/stats` | Per-user stat submission and reads, global integers/floats and history, achievement unlock times, RGBA icons and stats/achievement callbacks. Existing leaderboard entry downloads inherit the upstream details-length fix. |
| `timeline` | `timeline.setTimelineGameMode`, `setTimelineStateDescription`, `clearTimelineStateDescription`, `addTimelineEvent`. The binding passes title, description and icon in the SDK's argument order. |
| `server` | The root [`gameServer`](game-server.md) uses the game-server context for login, listing, validation, group and client events, and shared-socket packet forwarding. |

## Interpreting results

- `callback.register` preserves upstream 0.4.0's TypeScript declarations. Four old annotations disagree with what both addons actually deliver: server-connected delivers `null`, the two server failure/disconnect callbacks carry string reasons, and lobby chat member-state changes are strings. Use `callback.registerTyped` for accurate inference. Both names register the same listener, and the payload is identical.
- `stats.requestGlobalStats(days)` resolves to the completion's game ID. Upstream 0.13.1 discards the operation's `EResult`; resolution is **not** proof of a successful refresh. Read the relevant getter and allow for cached data. `requestUserStats` is submission-only. Stats reads return `null` for missing values.
- `callback.SteamCallback.GlobalStatsReceived` and `DeleteItemResult` observe **broadcasts**, when Steam supplies them. Handle-specific completions take a different path. Await the request to track a particular operation. Upstream's Workshop deletion wrapper treats `EResultNone` as success.
- New global stats, playtime, Cloud share and ID/total query promises stop waiting after 30 seconds. The Rust crate does not expose a cancellation handle for the underlying call result; a late result releases it, while an undelivered result can remain registered for the client lifetime.
- [Server browsing](server-browser.md) supports list requests and cancellation. It does not wrap ping, player-details or rules queries. A copied server entry is not evidence of live reachability. The package does not expose unsafe pointers or Rust's blocking socket iterators to JavaScript.
- Use `workshop.getAllItemIds` or `getUserItemIds` to fetch only IDs. The existing detailed-query methods return full rows even if passed `onlyIds: true`. The [SDK playtime limit](https://partner.steamgames.com/doc/api/ISteamUGC#StartPlaytimeTracking) is 100 items per request. A rejected Workshop update setter may print an upstream Rust assertion; published builds use unwinding and turn that rejection into a JavaScript error. Custom builds that force `panic=abort` cannot recover from that assertion.

## Reproduce the checks

After `corepack pnpm install --frozen-lockfile` and `corepack pnpm run build`, run `corepack pnpm test:bindings`. On Linux x64 or ARM64 run `corepack pnpm test:steamworks-013`, `sh test/inventory-fixture/run.sh` and `sh test/game-server-fixture/run.sh`. The fixtures drive the public Node API with synthetic SDK responses, including out-of-order completions, listener replacement, release and timeouts. The [Deviation Game checks](deviation-game.md#what-the-checks-cover) separately exercise the unchanged adapter in a packaged Linux x64 Electron process.
