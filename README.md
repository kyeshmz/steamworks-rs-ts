[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Steamworks.ts

**Steamworks.ts is an independently maintained fork of [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js)** for Node.js and Electron games. It builds on the original Rust bindings with typed APIs for lobbies, achievements, inventory, networking, and dedicated servers.

Development here continues independently after upstream activity slowed and [users asked for a maintained fork](https://github.com/ceifa/steamworks.js/issues/199). This project has its own roadmap and package name, `@kyeshmz/steamworks.ts`. It is neither an official upstream successor nor a Valve product.

[Package website](website/) · [API reference](client.d.ts) · [Issue tracker](https://github.com/kyeshmz/steamworks.ts/issues/1)

The [fork and issue audit](docs/coverage/README.md) compares implementations and lists remaining work. See [the fork feature guide](docs/fork-features.md) for the APIs available here and their verification status.

The [parity ledger](docs/coverage/parity.md) tracks donor features and remaining differences across both the `steamworks.js` and `steamworks-ffi-node` networks. With the pinned source objects available, `pnpm parity:check` reproduces it locally.

## Other Steamworks packages for JavaScript

| Package | When to consider it |
|---|---|
| [steamworks.js](https://github.com/ceifa/steamworks.js) | The original Rust-backed Node module. Consider it if an existing game depends on the upstream `steamworks.js` package and API. |
| [steamworks-ffi-node](https://github.com/ArtyProf/steamworks-ffi-node) | A TypeScript wrapper that calls the Steamworks SDK through Koffi FFI. Consider it if you want its manager-based API and can supply the SDK redistributables. |
| [Greenworks](https://github.com/greenheartgames/greenworks) | A native plugin for integrating Steamworks with NW.js and Electron. Consider it when maintaining a game already built around Greenworks; check binary compatibility with your runtime version. |

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

Until the first npm release, build a tarball for your platform from this checkout. Install the resulting `.tgz` in your game:

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm run build
corepack pnpm pack
# In your game directory:
pnpm add /path/to/kyeshmz-steamworks.ts-0.4.0.tgz
```

### Electron

Initialize Steam in the Electron main process. Expose only the operations the renderer needs through a sandboxed [preload](test/electron/preload.js) and [main-process handlers](test/electron/main.js). The example keeps context isolation enabled and Node integration disabled.

To make the steam overlay working, call the `electronEnableSteamOverlay` on the end of your `main.js` file:

```js
require('@kyeshmz/steamworks.ts').electronEnableSteamOverlay()
```

For the production build, copy the relevant distro files from `sdk/redistributable_bin/{YOUR_DISTRO}` into the root of your build. If you are using electron-forge, look for [#75](https://github.com/ceifa/steamworks.js/issues/75).


## How to build

Use [Node.js](https://nodejs.org/en/) 22.13 or newer on the 22.x line, [Rust](https://www.rust-lang.org/tools/install), and a native toolchain for your platform. You need a signed-in [Steam](https://store.steampowered.com/about/) client to exercise live client APIs.

From the repository root, run `corepack enable`, `pnpm install --frozen-lockfile`, and `pnpm run build:debug` to build the library.

### CI coverage

The [release workflow](.github/workflows/publish.yml) builds the native addon for each target and checks the **packed release tarball** in Node.js and the pinned Electron main process. On macOS, Apple Silicon and Intel have separate native build and runtime jobs. Electron asserts its platform and architecture before loading the binary.

| Target | Native runner | Runtime checks |
|---|---|---|
| macOS Apple Silicon (arm64) | `macos-15` | Node.js + Electron |
| macOS Intel (x64) | `macos-15-intel` | Node.js + Electron |
| Windows x64 | `windows-latest` | Node.js + Electron |
| Linux x64 | `ubuntu-latest` | Node.js + Electron under Xvfb |

Other gates cover Rust formatting, Clippy and unit tests, strict TypeScript compilation in a clean package consumer, and offline Steam client/server fixtures on both Macs. These checks do not sign in to Steam or validate overlay rendering. GitHub Actions is currently disabled on this repository.

## Releases

Use Conventional Commits (`fix:`, `feat:`, or `feat!:`). After CI passes on `main`, Release Please opens or updates a PR with the version and [changelog](CHANGELOG.md). Merging the release PR creates a GitHub release and publishes the tested tarball to npm.

For the first publication, configure the repository secret `NPM_TOKEN` with publish access to `@kyeshmz/steamworks.ts`. After the package exists, configure npm trusted publishing for `kyeshmz/steamworks.ts`, workflow `publish.yml`, and remove the bootstrap token. A failed npm publication can be retried by dispatching the Publish workflow on the existing release tag.

### Testing Electron

After building, run `pnpm --dir test/electron start` to launch the [Electron example](./test/electron).

Click "activate overlay" to test the overlay.
