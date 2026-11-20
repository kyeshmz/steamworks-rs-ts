[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform CI](https://github.com/kyeshmz/steamworks-rs-ts/actions/workflows/publish.yml/badge.svg?branch=main)](https://github.com/kyeshmz/steamworks-rs-ts/actions/workflows/publish.yml)

# Steamworks.ts

**Steamworks.ts is an independently maintained fork of [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js)** for Node.js and Electron games. It builds on the original Rust bindings with typed APIs for lobbies, achievements, inventory, networking, and dedicated servers.

Development here continues independently after upstream activity slowed and [users asked for a maintained fork](https://github.com/ceifa/steamworks.js/issues/199). This project has its own roadmap and package name, `@kyeshmz/steamworks.ts`. It is neither an official upstream successor nor a Valve product.

[Package website](website/) · [API reference](client.d.ts) · [Issue tracker](https://github.com/kyeshmz/steamworks-rs-ts/issues/1)

The [fork and issue audit](docs/coverage/README.md) compares implementations and lists remaining work. See [the fork feature guide](docs/fork-features.md) for the APIs available here and their verification status.

The [parity ledger](docs/coverage/parity.md) tracks donor features and remaining differences across both the `steamworks.js` and `steamworks-ffi-node` networks. With the pinned source objects available, `pnpm parity:check` reproduces it locally.

## Other Steamworks packages for JavaScript

| Package | When to consider it |
|---|---|
| [steamworks.js](https://github.com/ceifa/steamworks.js) | The original Rust-backed Node module. Consider it if an existing game depends on the upstream `steamworks.js` package and API. |
| [steamworks-ffi-node](https://github.com/ArtyProf/steamworks-ffi-node) | A TypeScript wrapper that calls the Steamworks SDK through Koffi FFI. Consider it if you want its manager-based API and can supply the SDK redistributables. |
| [Greenworks](https://github.com/greenheartgames/greenworks) | A native plugin for integrating Steamworks with NW.js and Electron. Consider it when maintaining a game already built around Greenworks; check binary compatibility with your runtime version. |

The [Koffi and Rust comparison](docs/koffi-vs-rust.md) explains when a TypeScript FFI binding is a better fit than this Rust addon.

The addon uses [`steamworks` 0.13.1](https://docs.rs/steamworks/0.13.1/steamworks/) and the Valve SDK 1.64 redistributables supplied by `steamworks-sys` 0.13.0. See the [0.13 integration notes](docs/steamworks-0.13.md) for the JavaScript API and verification limits.

## API

Pass an app ID to `init`, or omit it and provide a `steam_appid.txt` file.
The example prints the Steam username and activates an achievement.

```js
const steamworks = require('@kyeshmz/steamworks.ts')

const client = steamworks.init(480)
console.log(client.localplayer.getName())

if (client.achievement.activate('ACHIEVEMENT')) {
    console.log('Achievement activated')
}
```

See the [TypeScript declarations](client.d.ts) for supported calls and their signatures.

## Installation

For Deviation Game or an existing `steamworks.js` import, see [swap the dependency without changing game code](docs/deviation-game.md).

Until the first npm release, build a tarball for your platform from this checkout:

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm run build
corepack pnpm pack
```

Then install the resulting `.tgz` in your game directory:

```sh
pnpm add /path/to/kyeshmz-steamworks.ts-0.4.0.tgz
```

### Electron

Initialize Steam in the Electron main process. Expose only the operations the renderer needs through a sandboxed [preload](test/electron/preload.js) and [main-process handlers](test/electron/main.js). The example keeps context isolation enabled and Node integration disabled.

To make the steam overlay working, call the `electronEnableSteamOverlay` on the end of your `main.js` file:

```js
require('@kyeshmz/steamworks.ts').electronEnableSteamOverlay()
```

For production, unpack the native addon and the matching Valve library **together** from the package's `dist/` directory. With Electron Builder, unpack the installed package path; for Deviation Game that is `node_modules/steamworks.js/**`. See the [packaged example and checks](docs/deviation-game.md#what-the-checks-cover).


## How to build

Use [Node.js](https://nodejs.org/en/) 22.13 or newer on the 22.x line, [Rust](https://www.rust-lang.org/tools/install), and a native toolchain with C++14 support for the server-browser callback bridge. You need a signed-in [Steam](https://store.steampowered.com/about/) client to exercise live client APIs.

From the repository root, run `corepack enable`, `pnpm install --frozen-lockfile`, and `pnpm run build:debug` to build the library. The build regenerates `client.d.ts` from the Rust signatures. The root `index.d.ts` is checked in; update it when the root exports in `index.js` change.

### CI coverage

Open [public Platform CI runs](https://github.com/kyeshmz/steamworks-rs-ts/actions/workflows/publish.yml) and select a revision to see the results by CPU architecture. The [workflow](.github/workflows/publish.yml) builds **five native addons** and packs them into one tarball. The Node and Electron jobs install that tarball as `steamworks.js` and check the process architecture before loading it.

| Operating system | Native CPU / process | Additional execution mode |
|---|---|---|
| GNU/Linux | x86-64 and ARM64 (AArch64), each on its own runner | — |
| macOS | Intel x86-64 and Apple Silicon ARM64 | x64 Node and Electron on Apple Silicon using Rosetta |
| Windows | x86-64 | x64 Node and Electron on Windows 11 ARM using Windows emulation |

Each of the seven execution modes has Node 22, 24 and 26 jobs and one check with the pinned Electron main process. Linux x64 and ARM64 also run the native SDK fixtures. One Linux runner image per CPU is sufficient for this matrix.

**Windows on ARM requires an x64 process.** The pinned Steam SDK has no Windows ARM64 DLL. Separate Node 22 jobs verify that native Windows ARM64 and 32-bit x86 processes get an explicit unsupported-architecture error. Those are rejection tests, not support claims. There is no 32-bit ARM build.

The `Platform compatibility` gate requires all runtime and rejection tests to pass. Pull requests and pushes to `main` or `ci/**` trigger the checks. Live Steam login, overlay rendering and Steam Deck hardware are outside the scope of these jobs.

## Releases

Use Conventional Commits (`fix:`, `feat:`, or `feat!:`). After CI passes on `main`, Release Please opens or updates a PR with the version and [changelog](CHANGELOG.md). Merging the release PR creates a GitHub release and publishes the tested tarball to npm.

For the first publication, configure the repository secret `NPM_TOKEN` with publish access to `@kyeshmz/steamworks.ts`. After the package exists, configure npm trusted publishing for `kyeshmz/steamworks-rs-ts`, workflow `publish.yml`, and remove the bootstrap token. A failed npm publication can be retried by dispatching the Publish workflow on the existing release tag.

### Testing Electron

After building, run `pnpm --dir test/electron start` to launch the [Electron example](./test/electron).

Click "activate overlay" to test the overlay.
