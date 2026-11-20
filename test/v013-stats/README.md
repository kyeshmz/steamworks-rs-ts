# Stats bindings

`stats.requestGlobalStats(historyDays)` requests zero to 60 days and resolves to the completion's game ID as a `bigint`. It rejects I/O failure or a 30-second timeout. The official steamworks 0.13.1 request callback discards the SDK operation's `EResult`. Resolution does not confirm a successful refresh. Check the global getters for availability and remember that a getter can return cached data. A timed-out request cannot be cancelled through this crate; its callback remains registered until Steam completes it or the process exits.

`stats.requestUserStats(steamId)` submits a request for a nonzero unsigned 64-bit Steam ID. A zero SDK call handle throws; a returned value does not confirm completion or success. The current addon does not expose completion of this call. After Steam has supplied the user's stats, `getUserStatInt`, `getUserStatFloat`, and `getUserAchievement` read them by ID and name. A missing or unavailable value is `null`. User data can also be cached.

`achievement.getIconRgba(name)` returns `{ width, height, rgba }` with four bytes per pixel or `null` when the icon is unavailable, invalid, or exceeds 16 MiB. `getIcon` remains unchanged.

Run `sh test/v013-stats/run.sh` after `pnpm run build:debug` on native Linux x64 or ARM64. This isolated SDK fixture uses the addon's regular callback pump and does not connect to Steam. It exercises reversed global call completions, I/O failure, an SDK operation error that the crate discards, timeout, user-stat reads, and RGBA bounds.
