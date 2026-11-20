# Fork features

Use `const client = require('@kyeshmz/steamworks.ts').init(appId)` in the main process. The [declarations](../client.d.ts) list all public methods and parameter types.

| Area | What applications can do |
|---|---|
| Leaderboards | Find or create a board, upload scores and details, download ranked entries, and read board metadata. |
| Friends | Enumerate friends by relationship or [named group](friends-groups.md), read avatars and game/lobby activity, request rich presence, send game invites, and handle join notifications. |
| Input | Find action origins and glyphs, change action-set layers, read motion data, set the lightbar, open the binding panel, and trigger vibration and haptics. |
| Lobbies | Filter by strings, numbers, slots or distance, exchange chat buffers, invite members, and change ownership, type or member limit. |
| Networking | Use [Steam Networking Messages](networking-messages.md), [P2P or direct-IP sockets with poll groups](networking-sockets.md), or legacy P2P with explicit channels. These transports are not interchangeable. |
| Dedicated server | Use the package's separate [`gameServer` entry point](game-server.md) to initialize a server, update its listing, pump callbacks, check connection state and shut down. |
| Authentication | Start and end [peer authentication sessions](auth-sessions.md), listen for validation on the [dedicated server](game-server.md), or request an [encrypted app ticket](encrypted-app-ticket.md) for your backend. |
| Achievements and stats | Fetch global unlock percentages, localized attributes and icons, and get or set float stats. |
| Cloud and Workshop | Read or write binary saves asynchronously, group related writes, query quota, select per-file sync platforms, and read or update [Workshop metadata, tags and collection children](workshop.md). |
| Overlay and screenshots | Detect Big Picture and overlay availability, place notifications, hook capture requests, submit RGB pixels, import JPEGs with thumbnails, and receive completion events. |
| Apps and DLC | Read launch parameters, list DLC, request installation or removal, and check download progress. |
| Voice | [Capture compressed audio on request and decode received chunks to PCM](voice.md). The application controls recording, networking and playback. |
| Inventory | [Read items, exchange them according to a Steam recipe, and get serialized consume or exchange receipts](inventory-reads.md). Trading and purchases are separate operations. |

`init()` reuses the process's first successful Steam client. Calling it again with a different App ID throws. Start a separate process to use another App ID. This keeps asynchronous inventory reads and their native result handles attached to the session that created them.

Register `SteamInputDeviceConnected` and `SteamInputDeviceDisconnected` before calling `client.input.enableDeviceCallbacks()`. Inventory definition and full-update notifications are available through the same callback registry. Use `client.runtime.pumpStats()` to inspect pump timing; pass `true` to read and reset the counters. The runtime delivers callbacks to handlers rather than maintaining the event-drain queue used by some other bindings.

## What the checks prove

`test/package-smoke.js` loads the built addon and checks native argument validation without a Steam account. `test/typescript` compiles a consumer against the declarations. `node test/networking-sockets.js` tests invalid socket handles, message sizes, ports and peer IDs without opening a connection. The release workflow is configured for Linux x64, Windows x64, and both Mac architectures. GitHub Actions is disabled on this repository.

Use `node test/steam-readonly.js 480` with a signed-in Steam client to check reads. `test/inventory-definitions.js` needs an App ID and a known item definition. Overlay rendering, physical input, two-account networking and writes to Steam need separate live tests.

## Contributions

Leaderboard and persona bindings draw on upstream PRs [#198](https://github.com/ceifa/steamworks.js/pull/198) (fixfell7), [#208](https://github.com/ceifa/steamworks.js/pull/208) (DavidBCD), [#186](https://github.com/ceifa/steamworks.js/pull/186) (Dustin Lacewell) and [#211](https://github.com/ceifa/steamworks.js/pull/211) (Spencer McWilliams). Workshop file types, overlay events, cloud metadata and lobby chat draw on [#191](https://github.com/ceifa/steamworks.js/pull/191), [#205](https://github.com/ceifa/steamworks.js/pull/205), [#207](https://github.com/ceifa/steamworks.js/pull/207) and [#210](https://github.com/ceifa/steamworks.js/pull/210). The [audit](coverage/README.md) identifies additional contributors and compares alternative implementations.
