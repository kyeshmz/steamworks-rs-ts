## Sub-features
Initialize a dedicated game server, set its listing, request logon, pump callbacks and shut down.

## How to get to it (user POV)
Import `gameServer` from `@kyeshmz/steamworks.ts` in a separate process. See the [server guide](../../../../docs/game-server.md).

## Driving it with Node
On Linux x64, Linux ARM64 or macOS, run `sh test/game-server-fixture/run.sh`. It builds a test-local SDK stand-in and drives the public Node API through initialization failure, retry, duplicate initialization, listing, logon, callbacks and shutdown. Run `sh test/inventory-fixture/run.sh` to verify server initialization is rejected in an already initialized client process.

## Gotchas
The fixture never signs in to Valve or publishes a server. A master-server listing or real peer connection requires separate live evidence.
