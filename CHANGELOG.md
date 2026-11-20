# Changelog

## [1.0.0](https://github.com/kyeshmz/steamworks-rs-ts/compare/v0.4.0...v1.0.0) (2026-11-20)


### ⚠ BREAKING CHANGES

* init now reuses the active Steam client; switching App IDs requires a new process. This preserves native result handles across concurrent calls.

### Features

* add a dedicated Steam game server entry point [skip ci] ([41243a9](https://github.com/kyeshmz/steamworks-rs-ts/commit/41243a9da152fdae1744707e4cd1552775235719))
* add leaderboards, friends, controller glyphs, lobby search and modern P2P messaging ([db61299](https://github.com/kyeshmz/steamworks-rs-ts/commit/db61299ff8702d755ed49ccaa6717c1ba84487fc))
* add named friends groups and inventory transaction receipts [skip ci] ([5dc8a8c](https://github.com/kyeshmz/steamworks-rs-ts/commit/5dc8a8c30f73599a398cfb78d465c5a7ce1f6f46))
* add P2P sockets, inventory reads, Workshop queries and voice [skip ci] ([9772188](https://github.com/kyeshmz/steamworks-rs-ts/commit/9772188587f3af825b45cd8026f365ff5567732d))
* expose client initialization state and overlay activation details [skip ci] ([47280da](https://github.com/kyeshmz/steamworks-rs-ts/commit/47280daa68a2eb59c7fd98670017c556bcfeb80c))
* expose encrypted app tickets and dedicated server peer validation [skip ci] ([fe1bd8b](https://github.com/kyeshmz/steamworks-rs-ts/commit/fe1bd8bce3b8110272fdd8b68e99407594dfa9ab))
* integrate Steamworks 0.13 and verify release packages across platforms ([f579312](https://github.com/kyeshmz/steamworks-rs-ts/commit/f579312a9650972c595abf0a4406edce507bb1f2))
* introduce the Steamworks.ts package website [skip ci] ([0c792a4](https://github.com/kyeshmz/steamworks-rs-ts/commit/0c792a4c549594d9bbf63af62b9869b8c4d867d9))
* ship Linux ARM64 and verify native and emulated CPU architectures ([7a74548](https://github.com/kyeshmz/steamworks-rs-ts/commit/7a74548464be23b4f614b9f2a304366e1a5488de))
* validate peer tickets and add direct IP sockets and poll groups [skip ci] ([154e1b2](https://github.com/kyeshmz/steamworks-rs-ts/commit/154e1b2a5d42be89b2f54b7fab54e2f14fd0bac3))


### Bug Fixes

* **ci:** resolve tarball dependencies in a fresh consumer ([c13af3c](https://github.com/kyeshmz/steamworks-rs-ts/commit/c13af3c93d5e2958fdf2e8ab7d3992f023be5f12))

## Changelog

This fork starts from upstream version 0.4.0. Release Please adds entries here as part of each release PR.
