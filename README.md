[![Build Status](https://github.com/kyeshmz/steamworks.ts/actions/workflows/publish.yml/badge.svg)](https://github.com/kyeshmz/steamworks.ts/actions/workflows/publish.yml)
[![npm](https://img.shields.io/npm/v/@kyeshmz/steamworks.ts.svg)](https://npmjs.com/package/@kyeshmz/steamworks.ts)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Chat](https://img.shields.io/discord/663831597690257431?label=chat&logo=discord)](https://discord.gg/H6B7UE7fMY)

# Steamworks.ts

Steamworks.ts tracks [actionable upstream issues](https://github.com/kyeshmz/steamworks.ts/issues/1). It is based on [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js).

The [fork and issue audit](docs/coverage/README.md) compares implementations and lists remaining work. See [the fork feature guide](docs/fork-features.md) for the APIs available here and their verification status.

The [parity ledger](docs/coverage/parity.md) tracks donor features and remaining differences across both the `steamworks.js` and `steamworks-ffi-node` networks. With the pinned source objects available, `pnpm parity:check` reproduces it locally.

A modern implementation of the Steamworks SDK for HTML/JS and NodeJS based applications.

## Why

I used [greenworks](https://github.com/greenheartgames/greenworks) for a long time and it's great, but I gave up for the following reasons.

* It's not being maintained anymore.
* It's not up to date.
* It's not context-aware.
* You have to build the binaries by yourself.
* Don't have typescript definitions.
* The API it's not trustful.
* The API implement callbacks instead of return flags or promises.
* I hate C++.

## API

```js
const steamworks = require('@kyeshmz/steamworks.ts')

// You can pass an appId, or don't pass anything and use a steam_appid.txt file
const client = steamworks.init(480)

// Print Steam username
console.log(client.localplayer.getName())

// Tries to activate an achievement
if (client.achievement.activate('ACHIEVEMENT')) {
    // ...
}
```

See the [TypeScript declarations](client.d.ts) for supported calls and their signatures.

## Installation

To use steamworks.js you don't have to build anything, just install it from npm:

```sh
pnpm add @kyeshmz/steamworks.ts
```

### Electron

Initialize Steam in the Electron main process. Expose only the operations the renderer needs through a sandboxed [preload](test/electron/preload.js) and [main-process handlers](test/electron/main.js). The example keeps context isolation enabled and Node integration disabled.

To make the steam overlay working, call the `electronEnableSteamOverlay` on the end of your `main.js` file:

```js
require('@kyeshmz/steamworks.ts').electronEnableSteamOverlay()
```

For the production build, copy the relevant distro files from `sdk/redistributable_bin/{YOUR_DISTRO}` into the root of your build. If you are using electron-forge, look for [#75](https://github.com/ceifa/steamworks.js/issues/75).


## How to build

> You **only** need to build if you are going to change something on steamworks.js code, if you are looking to just consume the library or use it in your game, refer to the [installation section](#installation).

Make sure you have the latest [node.js](https://nodejs.org/en/), [Rust](https://www.rust-lang.org/tools/install) and [Clang](https://rust-lang.github.io/rust-bindgen/requirements.html). We also need [Steam](https://store.steampowered.com/about/) installed and running.

From the repository root, run `corepack enable`, `pnpm install --frozen-lockfile`, and `pnpm run build:debug` to build the library.

CI builds for Linux x64, Windows x64, and both macOS architectures.

## Releases

Use Conventional Commits (`fix:`, `feat:`, or `feat!:`). After CI passes on `main`, Release Please opens or updates a PR with the version and [changelog](CHANGELOG.md). Merging the release PR creates a GitHub release and publishes the tested tarball to npm.

For the first publication, configure the repository secret `NPM_TOKEN` with publish access to `@kyeshmz/steamworks.ts`. After the package exists, configure npm trusted publishing for `kyeshmz/steamworks.ts`, workflow `publish.yml`, and remove the bootstrap token. A failed npm publication can be retried by dispatching the Publish workflow on the existing release tag.

### Testing Electron

After building, run `pnpm --dir test/electron start` to launch the [Electron example](./test/electron).

Click "activate overlay" to test the overlay.
