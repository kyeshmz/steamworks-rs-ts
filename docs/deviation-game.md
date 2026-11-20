# Use the fork as Deviation Game's `steamworks.js`

Deviation Game imports `steamworks.js` in the Electron main process. Vite leaves that import external. Electron Builder unpacks `node_modules/steamworks.js/**` so the native addon can load its sibling Valve library. Keep **the dependency key** `steamworks.js`. Installing only `@kyeshmz/steamworks.ts` alongside it leaves the game on the original module.

## Try a local build

With `steamworks-rs-ts` cloned directly under `outdraw-turbo`, run these commands in the fork:

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm run build
mkdir -p target/packages/steamworks-0.13.1
corepack pnpm pack --pack-destination target/packages/steamworks-0.13.1
```

For a local trial, change this **one entry** in the game's `apps/electron-vite/package.json`:

```json
"steamworks.js": "file:../../steamworks-rs-ts/target/packages/steamworks-0.13.1/kyeshmz-steamworks.ts-0.4.0.tgz"
```

Run `corepack pnpm install` from the game workspace root to update its lockfile. No import, IPC handler, Vite external, or `asarUnpack` change is necessary. From `apps/electron-vite`, check what actually resolved:

```sh
node -p "require('steamworks.js/package.json').name"
```

Expect `@kyeshmz/steamworks.ts`. To switch back, restore the game's prior dependency (`^0.4.0` at the revision below) and its matching lockfile, then run `corepack pnpm install --frozen-lockfile`. The same check should now print `steamworks.js`.

A local build contains only the target you compiled. Use a tarball containing **all** release binaries for a multi-platform game build, including both `.node` files for the universal Mac app. A local `file:` path must be available on every machine that runs the install. Give a rebuilt tarball a fresh filename and update the dependency and lockfile together so the previous integrity hash cannot select stale bytes. Once a tested scoped version is published to your registry, a portable dependency value is `npm:@kyeshmz/steamworks.ts@<published-version>`.

## What the checks cover

The reference consumer is game commit `9304b71d7752d288c80827a9c5bc8d77152a8c3f`, which locked `steamworks.js@0.4.0`. The checks cover its main-process API, not every API ever exported by upstream. This fork also has different client reinitialization semantics: repeated initialization reuses the first successful client and a different app ID is rejected.

On Linux x64 with the fork built and the game checkout's dependencies installed:

```sh
sh test/deviation-native/run.sh ../node_modules/steamworks.js
node test/deviation-game/surface.cjs ../node_modules/steamworks.js .
sh test/deviation-native/run.sh .
sh test/deviation-native/run.sh . overlap
sh test/deviation-game/pack.sh .. 9304b71d7752d288c80827a9c5bc8d77152a8c3f
sh test/deviation-game/pack.sh .. 9304b71d7752d288c80827a9c5bc8d77152a8c3f fixture
```

The first two commands drive the same JavaScript calls through each real addon against an offline SDK fixture. They check relaunch, identity, ownership, stats, achievements, binary tickets, and cancellation. The fork-only `overlap` suite delivers callbacks out of order, delivers a late reply after a timeout, and checks malformed ticket lengths and repeated cancellation.

The packaging check reads the adapter, IPC constants, Electron version and builder config from the selected game revision. In a separate application under `target/deviation-game/` it installs the fork tarball under the old name, typechecks the unchanged adapter, bundles it with the dependency external, and launches the packaged Electron main process under Xvfb. It compares the unpacked `.node` file and SDK library to the installed files. The optional fixture run replaces only the SDK **in that test output** and calls the adapter's registered IPC handlers. Use the non-fixture package to test native loading; never distribute the fixture package.

The surface comparison starts each package in a separate Node process. Fully restart the game when swapping packages: loading two addons that require different versions of the same `libsteam_api.so` into one Linux process can make the dynamic linker reuse the first copy.

The offline checks do not contact the Steam Partner API. To try the **real account path on Linux x64**, open Steam with a test account licensed for `3104020` and run:

```sh
sh test/deviation-game/pack.sh .. 9304b71d7752d288c80827a9c5bc8d77152a8c3f live /path/to/verified-package.tgz
```

The `live` mode loads the matching Valve library and runs the unmodified game's Steam adapter. It checks online status, identity, entitlement, the game's stat and achievement reads, and ticket issuance. The final ticket verification requires the game's normal `STEAM_API_KEY` and `STEAM_VERIFICATION_SECRET` in the local environment. Missing prerequisites fail the check rather than selecting a mock. This does not exercise a visible Shift+Tab overlay or an installed Steam depot launch.
