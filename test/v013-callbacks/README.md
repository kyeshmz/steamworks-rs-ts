# Callback fixture

Run `sh test/v013-callbacks/run.sh` after `pnpm run build:debug`. The fixture replaces the Steam SDK with an offline Linux x64 implementation.

To check a previously built 0.4.0 package in a separate Node process, run `BASELINE_PACKAGE=/absolute/path/to/package sh test/v013-callbacks/run.sh`. The addon must be built against Steamworks 0.11. The script copies its loader and addon into this checkout's fixture directory and checks the same legacy callback payloads without starting Steam.

`GlobalStatsReceived` and `DeleteItemResult` listeners observe broadcast callbacks when Steam supplies them. They do not complete or correlate an individual API request. The upstream dispatcher routes call-result completions by API call handle separately from broadcast callback IDs. Use the corresponding request's completion instead of a broadcast listener to track that request.

The fixture verifies that a `DeleteItemResult` listener does not receive or consume a handle-specific `workshop.deleteItem` completion, and that a separately delivered broadcast reaches the listener. It exercises bounded relay debug text with a full non-NUL-terminated buffer and invalid UTF-8. It does not establish whether a live Steam service will emit these broadcasts.

Callback registrations remain active if their JavaScript handle is garbage-collected. Explicitly disconnect a retained handle or replace the registration to stop delivery. Disconnecting a stale handle must not remove its replacement. The fixture checks these cases and the complete persona payload, including numeric flag bits.

`callback.register` preserves the published 0.4.0 TypeScript declarations, including four declarations that do not match their runtime values. Use `callback.registerTyped` for the actual runtime shapes. The connected event is `null`, error reasons are Steam error variant names, and lobby member state changes are variant names rather than numbers. Both methods use the same listener registry, replace one another, and produce the same runtime payloads.
