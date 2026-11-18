# Read the local user's inventory

`init(appId)` initializes Steam once. Later calls with the active App ID or no argument reuse that client. A different explicit App ID throws without interrupting pending inventory requests or Workshop watchers. To change games, start a new process. If the first initialization fails, fix the cause and retry.

```js
const { inventory } = require('@kyeshmz/steamworks.ts').init(appId)
const { timestamp, items } = await inventory.getAllItems()
if (items.length) {
    const ids = [items[0].itemId]
    const selected = await inventory.getItemsByID(ids)
    const [row] = await inventory.getItemProperties(ids)
    console.log(timestamp, selected.items, row.properties)
}
```

`timestamp` is Steam server time in Unix seconds. An item's `itemId` is a `bigint`; `definition`, `quantity` and `flags` are numbers. `quantity` is Steam's unsigned 16-bit result field. For larger stacks, use the `quantity` string property when Steam supplies it. `getAllItemProperties()` returns property rows for the full inventory. Steam can return cached results. These reads do not grant, spend, or transfer anything.

`getItemsById` is the spelling in #209's generated JavaScript API. `getItemsByID` is an alias matching the Steam SDK name.

Each call owns its native result; there is no handle for JavaScript to release. Pending results time out after 30 seconds. A Steam error, an unreadable property, or a single SDK buffer over 64 MiB rejects the promise instead of returning a partial snapshot. Instance IDs outside the unsigned 64-bit range are rejected rather than truncated.

On macOS, run `sh test/inventory-fixture/run.sh` to exercise the Node addon against a local SDK stand-in, including failures and handle cleanup. It requires a C compiler, Node, and cached Cargo dependencies. It does not contact Steam or replace the SDK under `dist/`.

With Steam signed in and a configured test app, run `node test/inventory-reads.js <appId>` against the built package. A nonempty inventory is needed to check an individual item and its properties.
