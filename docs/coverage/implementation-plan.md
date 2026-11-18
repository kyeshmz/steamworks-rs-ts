# Implementing the six confirmed parity gaps

Baseline: [`0e77809`](https://github.com/kyeshmz/steamworks.ts/commit/0e77809491c31b42263ee730afd847416544b828). The [ledger](parity-ledger.md) distinguishes confirmed gaps from operations still awaiting semantic review. This plan covers the confirmed gaps only.

| Work package | Deliverable | Acceptance evidence |
|---|---|---|
| Inventory | Exchange items and return Steam-serialized receipts for consume and exchange operations. | A local SDK fixture checks the quantities passed to Steam, the exact receipt bytes returned and destruction of result handles on failure and success. No real items are consumed. |
| Friends | List named Steam friends groups and the Steam IDs belonging to one group. | Group IDs retain their native signed range; member IDs retain 64-bit precision. Empty groups and invalid input have defined behavior. |
| Encrypted app tickets | Request and retrieve an encrypted app ticket with optional application data. | The promise follows its own request to completion, rejects failures, and returns owned ticket bytes. No publisher key is needed to request a ticket. |
| Dedicated-server authentication | Begin and end a peer session and surface validation or revocation on the server callback pump. | A server fixture distinguishes submission from authorization, checks peer and owner IDs, and verifies teardown. |

Four GLM-5.3 implementation workers use independent worktrees. The parent owns integration, declaration generation, parity rules, and package verification. Rerun `pnpm test:bindings`, the affected SDK fixtures, and `pnpm parity:check` locally. Install the packed release tarball in a clean TypeScript consumer before marking a gap implemented. Steam-backed behavior requires separate live evidence.
