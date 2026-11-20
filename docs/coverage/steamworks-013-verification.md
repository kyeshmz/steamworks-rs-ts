# Steamworks 0.13.1 release verification

The Linux x64 release was tested against Deviation Game commit `9304b71d7752d288c80827a9c5bc8d77152a8c3f`, its locked `steamworks.js@0.4.0`, and its packaged Electron **44.9.2**. The candidate was `kyeshmz-steamworks.ts-0.4.0.tgz`, SHA-256 `89c1226685a60c154e7d7cbb37e117c5513033758aed536674cbbfce47c6f531`. Both Electron runs installed that exact local tarball under the key `steamworks.js`. No game file needed to change.

| Check | Observed result |
|---|---|
| SDK identity | All seven tracked redistributables matched `steamworks-sys` 0.13.0. The tarball contained the Linux x64 release addon and SDK 1.64 library. |
| Rust, Node and TypeScript | Formatting, Clippy with warnings denied, all three Rust unit tests, release build, binding smoke tests and focused consumer typechecks passed. |
| Existing API | All 258 inspected root and native paths from the game's installed npm package matched, including enum values and class members. A separate process using that original package confirmed persona flag bits, listener lifetime and legacy event payloads. |
| Client and dedicated server | The offline fixtures exercised actual native bindings for callbacks, keyboard, controller input, content, networking, stats, achievements, Timeline, Remote Play, server lists, and dedicated server lifecycle and networking. |
| Tickets | The same sequential calls passed against both addons. The candidate also passed all seven overlap cases, including reversed replies, late delivery after timeout, idempotent cancellation and malformed web-ticket lengths. |
| Packaged Electron with Valve library | The unchanged main-process adapter typechecked against the candidate. The packaged process loaded both native files from `app.asar.unpacked/node_modules/steamworks.js/dist/linux64`. |
| Packaged Electron with fixture SDK | The unchanged adapter's IPC handlers returned the expected identity, app ownership, stats, achievements and ticket values. An installed Chromium sandbox helper was used on this Linux host; sandboxing was not disabled. |
| Donor comparison | `pnpm parity:check` reproduced the [pinned ledger](parity-ledger.md). This checks surface inventory, not live behavior of every listed operation. |

For the [commands and dependency swap](../deviation-game.md) and the [upstream delta and known API limitations](../steamworks-0.13.md), see the linked guides. Windows, macOS, live Steam authentication, real multiplayer/relay traffic and visible overlay rendering were not tested in this run.
