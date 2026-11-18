# What to adopt from upstream Steamworks.js

Start with the package users install. Then port small bindings onto that package, one API at a time. Do not replace this fork with the most active default branch. For example, the inventory PR changes the SDK binaries as well as the API, while the two leaderboard PRs offer different names for the same operations.

This is a decision record against [upstream `80c5fd7`](https://github.com/ceifa/steamworks.js/commit/80c5fd7c4958ba90cf87403c3f967b8a6d748473), not a list of features shipped by this fork. Check the current `src`, `client.d.ts`, and packed module before implementing any row.

## What the audit covers

| Evidence | Coverage |
|---|---|
| [Fork index](forks.md) | 121 public fork records; 120 default branches fetched. Of those, 70 tips are already in upstream history, 49 have a nonempty delta since their merge base, and one has unrelated history. |
| [Issue ledger](issues.md) | 137 upstream issue records, each assigned one workstream: 52 open and 85 closed. PRs are counted separately. |
| [PR ledger](pulls.md) | All 73 fetched PR head refs, including all 17 PRs reported open by GitHub. The GitHub list did not return a state for ref #73. |
| [Diff evidence](deltas.md) | Changed source paths and candidate Rust function names, with identical diffs collapsed across forks and PRs. |
| [Snapshot](snapshot.json) | Exact base and tip hashes, branch names, issue numbers, PR numbers, titles and recorded states. |

`yorkzero831/steamworks.js` has no fetched tip. [Gurigraphics](https://github.com/Gurigraphics/steamworks.js/tree/473866aeb6b6b3b3ed78bb989f43ac1a79a6a817) has no common ancestor. It runs a TypeScript client and a C# server; use its matchmaking and friends code as a design reference, not a patch for the Rust addon. A fork with a small or empty Rust diff can still change the loader or ship different binaries.

## Adoption order and acceptance tests

### 1. Prove the release before expanding the API

- **Packaging and releases.** [#183](https://github.com/ceifa/steamworks.js/issues/183) reports different GitHub and npm APIs; [#192](https://github.com/ceifa/steamworks.js/issues/192), [#153](https://github.com/ceifa/steamworks.js/issues/153), and [#194](https://github.com/ceifa/steamworks.js/issues/194) concern loading or packaging. Build from one tagged source with a frozen pnpm lockfile. Install the packed tarball into a clean consumer with no Rust toolchain. On each supported OS/architecture, load the addon and import its declarations. Treat loading from a checkout as a different test. Keep a changelog and link each release to its source.
- **Adapt examples, not workarounds.** [PR #115](https://github.com/ceifa/steamworks.js/pull/115) supplies an electron-builder example. Extend it to run the packaged app with its native files outside the asar. [PR #180](https://github.com/ceifa/steamworks.js/pull/180) duplicates the platform switch and hard-codes the upstream package name. Test a scoped install and an alias before changing resolution. Do not ship [PR #190](https://github.com/ceifa/steamworks.js/pull/190)'s `npm run build` install-time hook for binary consumers. Do not adopt [PR #49](https://github.com/ceifa/steamworks.js/pull/49)'s `--no-sandbox` as the Linux default. Test Steam-launched Linux and macOS builds separately from host Node.
- **Separate the SDK upgrade.** [PR #196](https://github.com/ceifa/steamworks.js/pull/196) changes the `steamworks-rs` revision, callback ownership and redistributables. Other donors use still newer revisions. Pick one library/SDK pair and rebuild all targets from known sources; do not mix DLLs or `.node` files from forks. Exercise callbacks through subscribe, disconnect, shutdown and re-init. In particular, verify the new registry releases old handlers. [#171](https://github.com/ceifa/steamworks.js/issues/171) asks for detach; [#165](https://github.com/ceifa/steamworks.js/issues/165) reports a dropped auth channel. Neither is proved fixed by a dependency bump.

### 2. Start with independently testable bindings

| Capability and evidence | Port and acceptance test |
|---|---|
| Overlay status and callbacks. [#203](https://github.com/ceifa/steamworks.js/pull/203), [#205](https://github.com/ceifa/steamworks.js/pull/205), [#177](https://github.com/ceifa/steamworks.js/pull/177), [Spillmaker](https://github.com/Spillmaker/steamworks.js/tree/2aba0e2bd81fb952df5bff7aa0495430fa76500d) | Expose one enabled-status query and one activation event, not three synonymous wrappers. Enabled does not mean currently open. Register, open, close, disconnect; verify the event payload and absence of later deliveries. This does not repair [Linux rendering #195](https://github.com/ceifa/steamworks.js/issues/195) or [video #148](https://github.com/ceifa/steamworks.js/issues/148). |
| Cloud metadata. [#207](https://github.com/ceifa/steamworks.js/pull/207) | Add `isFilePersisted` and `fileTimestamp` alongside existing storage methods. Document timestamp units and missing-file behavior using a disposable save. This is not sync-platform selection ([#133](https://github.com/ceifa/steamworks.js/issues/133)) or asynchronous file I/O ([#35](https://github.com/ceifa/steamworks.js/issues/35)). |
| Workshop creation and updates. [#191](https://github.com/ceifa/steamworks.js/pull/191), [#175](https://github.com/ceifa/steamworks.js/issues/175), [JDeffner](https://github.com/JDeffner/steamworks.js) | Add an optional file type without breaking `createItem()` or the existing app ID argument. Default to `Community`. Stage language, metadata, key/value tags, content descriptors, child IDs and query options separately. Test omitted options and round-tripping the metadata on an authorized test item. [PR #172](https://github.com/ceifa/steamworks.js/pull/172) already landed deletion; do not port it again. In [#197](https://github.com/ceifa/steamworks.js/issues/197) the sample treats progress status as a result code: capture the completion result before changing upload behavior. |
| Friends and avatars. [#211](https://github.com/ceifa/steamworks.js/pull/211), [#186](https://github.com/ceifa/steamworks.js/pull/186), [#169](https://github.com/ceifa/steamworks.js/issues/169), [#104](https://github.com/ceifa/steamworks.js/issues/104) | Reuse a single `requestUserInformation` operation. Separate "not cached yet" from "no avatar"; keep the 64-bit Steam ID intact and document RGBA dimensions. #211 reads one persona, not a friends list. Compare [Pyrofennec21](https://github.com/Pyrofennec21/steamworks.js) for list, rich presence and avatar reads, and [pavelsima](https://github.com/pavelsima/steamworks.js) for invites. Verify a known friend and an uncached user before and after notification. |
| Leaderboards, float stats and achievements. [#208](https://github.com/ceifa/steamworks.js/pull/208), [#198](https://github.com/ceifa/steamworks.js/pull/198), [#170](https://github.com/ceifa/steamworks.js/issues/170), [#86](https://github.com/ceifa/steamworks.js/issues/86) | Choose one namespace and score/result shape. Both PRs implement find/create, upload and download. #198 also exposes metadata and cache management; #208 also caches handles by name. Scope handles to the active client. Test an unknown board, score details, keep-best, rank and cache invalidation after re-init. [Jackson0ne](https://github.com/Jackson0ne/steamworks.js) offers achievement percentage, attributes and icon; [JDeffner](https://github.com/JDeffner/steamworks.js) adds float stats. Neither requires importing a second leaderboard API. |

### 3. Gate callback-heavy work on real devices or two Steam clients

| Capability and evidence | Integration decision |
|---|---|
| Input. [#202](https://github.com/ceifa/steamworks.js/issues/202), [#185](https://github.com/ceifa/steamworks.js/issues/185), [#56](https://github.com/ceifa/steamworks.js/issues/56), [jespertheend](https://github.com/jespertheend/steamworks.js), [Slakinov](https://github.com/Slakinov/steamworks.js) | Start with manifest path, frame update, and controller enumeration. Then add action origins, PNG/SVG glyphs, binding panel, motion and hotplug callbacks. Compare [timelessnesses](https://github.com/timelessnesses/steamworks.js-inputs) for vibration/lightbar and #177 for action binds. Preserve existing controller methods rather than copying a wholesale rewrite. Use a real controller, an action manifest, and reconnect it. A browser `Gamepad` result does not test Steam Input. |
| Matchmaking and chat. [#210](https://github.com/ceifa/steamworks.js/pull/210), [#168](https://github.com/ceifa/steamworks.js/issues/168), [BrunoCaxito](https://github.com/BrunoCaxito/steamworks.js), [koyuuji](https://github.com/koyuuji/steamworks.js) | #210 sends bytes and retrieves the entry named in `LobbyChatMsg`; it does not implement filtered lobby discovery. Split list filters, member data, ownership/size changes and chat. Verify two clients can find the intended lobby and receive the same payload, including a message with NUL bytes. Test the 4 KB boundary and leaving the lobby. |
| Networking. [#178](https://github.com/ceifa/steamworks.js/pull/178), [#142](https://github.com/ceifa/steamworks.js/pull/142), [#124](https://github.com/ceifa/steamworks.js/issues/124), [cursoracc1](https://github.com/cursoracc1/steamworks.js) | #178 adds channels to legacy P2P; it does not migrate transport. #142 and [MikalDev](https://github.com/MikalDev/steamworks.js) offer Networking Messages; cursoracc1 exposes connection-oriented sockets, event pumping, authentication and relay status. Keep legacy calls compatible and name new transports separately. Test request/accept/reject, channel isolation, disconnect/reconnect and relay availability with two accounts. Handle connection failure without an unconditional accept. |
| Inventory. [#209](https://github.com/ceifa/steamworks.js/pull/209), [#164](https://github.com/ceifa/steamworks.js/issues/164), [#152](https://github.com/ceifa/steamworks.js/issues/152), [#55](https://github.com/ceifa/steamworks.js/issues/55) | Prefer #209 as the comparison point for reads, item properties, serialized results and consume/exchange, rather than stacking three inventory modules. [flowtycz](https://github.com/flowtycz/steamworks.js) separately covers prices, purchases and result-handle disposal. First test a read against a configured item definition. Gate writes and purchases behind explicit test-app approval; test ownership mismatch, expired results and quantities beyond 65,535 before depending on signed results. Do not equate a successful compile with a validated economy. |

### 4. Keep narrow features and unsupported environments separate

- Compare [SteelWaffe](https://github.com/SteelWaffe/steamworks.js) for DLC installation/progress, [jespertheend](https://github.com/jespertheend/steamworks.js) for URL launch parameters and Big Picture, and [closed PR #201](https://github.com/ceifa/steamworks.js/pull/201) for screenshots. These do not require importing those forks' friends or leaderboard code. Test only with an app that owns the relevant DLC or library entry.
- Evaluate [closed PR #204](https://github.com/ceifa/steamworks.js/pull/204) for additional Workshop previews separately from its build and SDK changes. Compare [skyatgit](https://github.com/skyatgit/steamworks.js) for localized updates. Do not treat a formatting-only change to `workshop.rs` as a Workshop feature.
- Use closed support reports as test cases, not as open requests or evidence that upstream resolved a bug. For example, [#60](https://github.com/ceifa/steamworks.js/issues/60) motivates a lossless ID round-trip; [#149](https://github.com/ceifa/steamworks.js/issues/149) motivates a packaged Forge launch. For Tauri, React Native and NW.js, document which native host was actually tested; an Electron test establishes none of these. Prioritize a reproducible report over broad compatibility claims.

## Reproduce or refresh

From the repository root, run `python3 scripts/audit-upstream.py --check`. This compares a fresh report against the files here. It reads local Git objects and the snapshot; it does not contact GitHub or require Steam. Run the same command without `--check` to regenerate the four indexes.

A fresh clone needs the pinned objects. These commands fetch only if the object is missing. Old objects may no longer be served if a fork disappears or rewrites history; preserve a mirror for long-term replay.

```sh
jq -r '(["ceifa/steamworks.js",.upstream], (.forks[] | select(.[2]!=null) | [.[0],.[2]]), (.pulls[] | ["ceifa/steamworks.js",.[1]])) | @tsv' docs/coverage/snapshot.json |
while read -r repo sha; do
  git cat-file -e "$sha^{commit}" 2>/dev/null || git fetch --no-tags "https://github.com/$repo.git" "$sha" || exit 1
done
python3 scripts/audit-upstream.py --check
```

To capture a **new** census, use a fresh full clone so a stale ref cannot stand in for an unavailable fork. With `gh`, `jq`, Python and Git installed, run:

```sh
git fetch https://github.com/ceifa/steamworks.js.git \
  '+refs/heads/main:refs/remotes/upstream/main' '+refs/pull/*/head:refs/pr/*'
gh api --paginate 'repos/ceifa/steamworks.js/forks?per_page=100' \
  --jq '.[] | [.full_name,.default_branch,.updated_at] | @tsv' > .git/upstream-forks.tsv
while read -r repo branch updated; do
  git fetch --no-tags "https://github.com/$repo.git" "+refs/heads/$branch:refs/audit/$repo" ||
    printf 'Unavailable: %s\n' "$repo" >&2
done < .git/upstream-forks.tsv
gh issue list -R ceifa/steamworks.js --state all --limit 10000 \
  --json number,state,title,url,body > .git/upstream-issues.json
python3 scripts/audit-upstream.py --capture
python3 scripts/audit-upstream.py --check
```

`--capture` reads the staged fork/issue lists and fetches PR **metadata** with `gh pr list`; it rejects an open PR whose local head is stale. Update the script's issue-to-workstream map when issue numbers change. Review the adoption decisions and counts after a refresh. Raise the CLI limit or paginate explicitly if either listing reaches 10,000.

The census excludes private forks, detached repositories, nondefault fork branches other than fetched PR heads, and issue comments. Fork tips are not simultaneous GitHub observations. The exact-diff comparison catches identical patches at the blob level but not independent ports of the same idea. A historical delta can also describe functionality upstream later added differently. No donor binaries were executed, and no Steam-backed operation was verified by this audit.
