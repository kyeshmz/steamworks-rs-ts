[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Chat](https://img.shields.io/discord/663831597690257431?label=chat&logo=discord)](https://discord.gg/H6B7UE7fMY)

# Steamworks.ts

Steamworks.ts brings Steam lobbies, achievements, inventory, networking, and dedicated servers to Node.js and Electron games through one typed native module. It is based on [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js).

[Package website](website/) · [API reference](client.d.ts) · [Issue tracker](https://github.com/kyeshmz/steamworks.ts/issues/1)

The [fork and issue audit](docs/coverage/README.md) compares implementations and lists remaining work. See [the fork feature guide](docs/fork-features.md) for the APIs available here and their verification status.

The [parity ledger](docs/coverage/parity.md) tracks donor features and remaining differences across both the `steamworks.js` and `steamworks-ffi-node` networks. With the pinned source objects available, `pnpm parity:check` reproduces it locally.

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

The release workflow targets Linux x64, Windows x64, and both macOS architectures. GitHub Actions is currently disabled.

## Releases

Use Conventional Commits (`fix:`, `feat:`, or `feat!:`). After CI passes on `main`, Release Please opens or updates a PR with the version and [changelog](CHANGELOG.md). Merging the release PR creates a GitHub release and publishes the tested tarball to npm.

For the first publication, configure the repository secret `NPM_TOKEN` with publish access to `@kyeshmz/steamworks.ts`. After the package exists, configure npm trusted publishing for `kyeshmz/steamworks.ts`, workflow `publish.yml`, and remove the bootstrap token. A failed npm publication can be retried by dispatching the Publish workflow on the existing release tag.

### Testing Electron

After building, run `pnpm --dir test/electron start` to launch the [Electron example](./test/electron).

Click "activate overlay" to test the overlay.
