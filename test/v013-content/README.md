# Steamworks 0.13 content fixture

After `pnpm run build:debug`, run `sh test/v013-content/run.sh` on native Linux x64 or ARM64. Run `sh test/v013-content/run.sh timeout` to withhold cloud share, item creation, and detailed query callbacks and check their 30-second timeouts. The fixture replaces the Steam SDK library in an isolated package under `target/v013-content`. It does not initialize the live Steam client.

The fixture exercises workshop update language, subscribed item filtering and a shorter result count, playtime tracking success and failure, deletion failure, download suspension, cloud platform masks, cloud sharing and forgetting, lobby game server data, combined lobby filters, detailed item calls with `onlyIds`, all-item and user ID queries, and all-item total queries. It checks large 64-bit IDs and invalid inputs.

`pnpm exec tsc --strict --noEmit --target es2020 --module Node16 --moduleResolution Node16 test/v013-content/types.ts` checks the generated public declarations after the build.

These checks do not prove actual Steam service behavior, other operating systems, or the behavior of every workshop query option.

Subscribed-item enumeration rejects a reported count above 100,000 before allocating the ID buffer. [Valve documents a maximum of 100 items for playtime tracking](https://partner.steamgames.com/doc/api/ISteamUGC#StartPlaytimeTracking). After N-API converts the JavaScript array, the wrapper enforces 1 to 100 entries before converting the IDs or calling Steam.

Detailed item queries always request full rows, even when `onlyIds` is true, so their existing result shapes remain intact. Other query settings remain in effect. Callers who need only IDs can use `getAllItemIds` or `getUserItemIds`; those request ID-only results from Steam. `onlyTotal` remains available on detailed queries.

The upstream `steamworks` 0.13.1 `delete_item` wrapper reports `EResultNone` as success, unlike other non-OK results. The fixture covers a non-OK error result, not `EResultNone`. Correcting that upstream interpretation requires access to its private call-result registration or a change to the shared dispatcher. The 30-second timeouts stop waiting but do not unregister an upstream callback; a late callback is ignored, while a callback that never arrives remains registered until client teardown. For a timed-out query, its Steam query handle also remains pending until its callback arrives.

The upstream update setters assert on Steam rejection. This wrapper catches those unwinds outside the dispatch lock and returns an error, but the default Rust panic hook still prints the assertion. A panic-abort build or a custom aborting panic hook cannot recover this way. Avoiding the panic itself requires an upstream API that exposes nonpanicking update setters while preserving its private call-result registration.
