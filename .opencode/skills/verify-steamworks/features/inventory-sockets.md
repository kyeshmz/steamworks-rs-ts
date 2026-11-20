## Sub-features
Read one or all inventory items and their properties. Open a P2P listener, authorize a peer, exchange binary messages and close both handles.

## How to get to it (user POV)
Initialize `@kyeshmz/steamworks.ts` with a Steam App ID. Use `client.inventory` and `client.networking_sockets`. See [inventory](../../../../docs/inventory-reads.md) and [sockets](../../../../docs/networking-sockets.md).

## Driving it with Node
After the native build, run `node test/networking-sockets.js` to check socket bounds. Run `sh test/inventory-fixture/run.sh` on Linux x64, Linux ARM64 or macOS to check success, failure, timeout and cleanup against the SDK stand-in. For an authorized live inventory, run `node test/inventory-reads.js <appId>`. For a local Steam self-connection, set `STEAM_SOCKET_SELF_TEST_APP_ID` to the authorized App ID and run the socket test again.

## Gotchas
A native load, fixture, or self-connection does not prove two-account relay behavior. A signed-in test account and two machines are needed to check invitations and remote delivery end to end.
