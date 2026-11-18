# What is still missing?

Not all audited capabilities are exposed. For example, the inventory read APIs do not exchange items or return signed transaction receipts, and filtering friends by relationship does not list members of a named friends group. The [generated ledger](parity-ledger.md) separates reviewed gaps from spelling differences and unreviewed operations. It is not a percentage of working Steam features.

## What to do next

| Priority | Evidence and decision |
|---|---|
| 1. Validate existing contracts | [BrunoCaxito's friend list](https://github.com/BrunoCaxito/steamworks.js/blob/0d2d573782841f72e02cc519c57a090a054c3628/src/api/friends.rs) accepts relationship filters and returns personas. Local `friends.getFriends(filter?)` returns IDs; use `friends.getPersona(id)` for details. Test two accounts and a nondefault filter. |
| 1. Exercise networking and auth with two clients | [PR 179](https://github.com/ceifa/steamworks.js/blob/ede25cc2151ac6482639ea5365359f4a9d56fad6/src/api/networking_sockets.rs) exposes an IP listener; [ArtyProf sockets](https://github.com/ArtyProf/steamworks-ffi-node/blob/f5ce2b7dc194672ed45f7352c9e325d12451028f/src/internal/SteamNetworkingSocketsManager.ts) exposes poll groups. Local bindings expose both. Verify message attribution, channel isolation, close and reconnect. Like [ArtyProf user](https://github.com/ArtyProf/steamworks-ffi-node/blob/f5ce2b7dc194672ed45f7352c9e325d12451028f/src/internal/SteamUserManager.ts), local `auth` can begin and end validation of peer tickets. Immediate acceptance of `beginAuthSession` is not a successful validation response. Verify success and rejection through `ValidateAuthTicketResponse`, then end the session. |
| 2. Test voice and event delivery | Use `voice.getAvailableVoice()` for the non-consuming byte count and `voice.getVoice()` to read audio. Test on a real microphone. Local hotplug and inventory update identifiers correspond to those in [jespertheend](https://github.com/jespertheend/steamworks.js/blob/7a4813fbe9e34ef574042dacbbd69dcc306e9e1b/src/api/callback.rs) and [spencermcw](https://github.com/spencermcw/steamworks.js/blob/f8b2a24404766ad45a9890fa0fc7bd5f261c0336/src/api/callback.rs). Verify callback delivery and disconnect with an actual controller or configured inventory. |
| 3. Test storage boundaries | Like [ArtyProf cloud](https://github.com/ArtyProf/steamworks-ffi-node/blob/f5ce2b7dc194672ed45f7352c9e325d12451028f/src/internal/SteamCloudManager.ts), local cloud exposes `beginFileWriteBatch` and `endFileWriteBatch`. Test with disposable files in an authorized app; a batch hint is not an atomic transaction. |
| 4. Keep server and client contracts separate | [whaqzhzd's server](https://github.com/whaqzhzd/steamworks.js/blob/8180e59214cb0073e4d153deb73986deecf23e2b/src/api/p2p/server.rs) has individual setters for server name, map and capacity. Local `require('@kyeshmz/steamworks.ts').gameServer.updateListing({serverName, mapName, maxPlayers})` sets all three together. It does not expose the donor's dedicated-server ticket-validation event. Local `runtime.pumpStats` measures callback timing; [cursoracc1](https://github.com/cursoracc1/steamworks.js/blob/d5c5e4e3567ff4ef94099a566f992818fd7df59f/src/events.rs) also reports queued/dropped events from its own queue. |

## How to read the ledger

- **Same name / name variant:** qualified member name exists in local declarations, possibly after case or namespace normalization. No signature, data shape, availability or behavior guarantee.
- **Alias:** reviewed counterpart for the operation, not a drop-in replacement. For example ArtyProf's `friends.getPersonaName()` asks for the *local* persona; our `friends.getPersonaName(id)` asks for someone else's. The appropriate local counterpart is `localplayer.getName()`. Gurigraphics' `SteamRemoteStorage.readFile` promises a boolean in its TS client but its C# handler returns UTF-8 text.
- **Partial:** the donor expresses a material capability that the local operation lacks. **Gap:** the donor has source for the operation and the local public bindings were inspected for an equivalent. **Review:** not classified. This last group includes real omissions, aliases, convenience compositions and historical APIs; do not count every row as missing.

Each row counts a qualified name once per comparison group, not a distinct Steam operation, project, overload, or passing test. See [rules](parity-rules.json) for curated decisions. Recheck them against both implementations when changing the API. No donor package was installed, built or executed.

## Coverage and limits

The input is the existing [frozen census](snapshot.json), not the public network as of today: 121 fork records (one unavailable) and 73 upstream PR heads. For related trees, the TypeScript parser reads `client.d.ts` and `index.d.ts` at each pinned tip. We count function/class-method/getter paths and additions to `callback.SteamCallback` present in neither that tip's merge base nor the pinned upstream. This removes old upstream history and deduplicates repeated additions across forks. Source IDs in each row resolve to full SHAs in the census. For example, `jq -r '.pulls[] | select(.[0]==179) | .[1]' docs/coverage/snapshot.json` identifies the exact IP-listener source.

For [Gurigraphics](https://github.com/Gurigraphics/steamworks.js/tree/473866aeb6b6b3b3ed78bb989f43ac1a79a6a817) we read the TypeScript modules actually re-exported by `client/index.ts`, not every prototype file. For [independent ArtyProf](https://github.com/ArtyProf/steamworks-ffi-node/tree/f5ce2b7dc194672ed45f7352c9e325d12451028f) we read `src/steam.ts`, its public manager types, the additional logger/overlay class exports and helper functions re-exported through `src/types/index.ts`.

### ArtyProf fork network

GitHub reported 10 forks and network size 10. All [ten public default branches](parity-ffi-forks.json) were fetched by name and pinned. Nine tips are identical to or ancestors of pinned ArtyProf main and therefore introduce no fork-specific changes. Only [kiranipiebox](https://github.com/kiranipiebox/steamworks-ffi-node/commit/83786f1d62708d63312a1aa588a8223e9a12aabb) diverges from its merge base `7741550942964f6428517ff0a4cf90c53ca2e902`. Across all ten: **zero additional public method paths**, one implementation change.

The divergent commit changes three source files and one documentation file. Ignoring whitespace, it replaces the fixed `SteamAPI_SteamApps_v008` accessor with `v009` in the loader, core and apps manager. It does not add an Apps operation. [Pinned ArtyProf main](https://github.com/ArtyProf/steamworks-ffi-node/blob/f5ce2b7dc194672ed45f7352c9e325d12451028f/src/internal/SteamLibraryLoader.ts) instead probes versioned accessors and selects the highest available one, including v009 when exported. Our Rust binding uses the pinned SDK's v008 accessor in `src/api/apps.rs`; neither runtime compatibility with another SDK nor identical ABI behavior follows from matching JS method names.

The renderer lists *all* paths changed since each merge base and compares divergent public method paths with both base and pinned main. Recheck the logic review with `git diff -w 7741550942964f6428517ff0a4cf90c53ca2e902 83786f1d62708d63312a1aa588a8223e9a12aabb`. Comparing only tip to main would wrongly attribute intervening main changes to this fork. This snapshot does not inspect nondefault branches, private/detached repositories, unmerged ArtyProf PR heads or later changes.

The local comparison uses generated `client.d.ts`, `index.d.ts` and direct root exports in `index.js`. It excludes native `runCallbacks`, which `index.js` drives internally. It records the native `game_server` methods under the public root `gameServer`, not the object returned by `init()`. The report hashes these inputs. A matching name does not prove the native binary or published tarball contains it. Both async and sync methods count as one path. Fields, overload changes, enum values, callback payload types, callback timing, lifecycle, packaging, host support, type-only exports and modified behavior under existing names are not exhaustively compared. Donors may omit or ship stale declarations. Consult the [source-delta index](deltas.md) for additional Rust candidates and non-API changes before concluding a feature is absent upstream.

## Reproduce offline

Use the repository's locked dependencies (TypeScript 5.5.3), Node and Git. First obtain the Git objects following [the census instructions](README.md#reproduce-or-refresh). For the independent repository, fetch the exact commit into the local object store if necessary:

```sh
jq -r '(["ArtyProf/steamworks-ffi-node", .upstream], (.forks[] | [.[0], .[2]])) | @tsv' docs/coverage/parity-ffi-forks.json |
while read -r repo pin; do
  git cat-file -e "$pin^{commit}" 2>/dev/null ||
    git fetch --no-tags --no-write-fetch-head "https://github.com/$repo.git" "$pin" || exit 1
done
```

Once these objects and the lockfile dependencies are available, the ledger does not contact the network. Run `node scripts/audit-parity.cjs --check` from the repository root to check the recorded result. After a local binding change, rebuild normally to regenerate `client.d.ts`, run `node scripts/audit-parity.cjs`, review the diff, then run `--check`. The renderer writes only `parity-ledger.md`; it never initializes Steam. Missing Git objects fail the command rather than silently substituting a newer revision. Retain the pinned Git objects for archival replay.
