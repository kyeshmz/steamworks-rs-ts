#!/bin/sh
set -eu
ulimit -c 0
root=$(CDPATH='' cd "$(dirname "$0")/../.." && pwd)
game=$(CDPATH='' cd "${1:?Pass the Deviation Game repository path}" && pwd)
rev=$(git -C "$game" rev-parse --verify "${2:-HEAD}^{commit}")
mode=${3:-vendor}
case "$mode" in vendor|fixture) ;; *) echo 'Expected vendor or fixture' >&2; exit 2 ;; esac
test "$(uname -sm)" = 'Linux x86_64'
mkdir -p "$root/target/deviation-game"
run=$(mktemp -d "$root/target/deviation-game/run.XXXXXX")
app="$run/app"
mkdir -p "$app/src/main" "$run/tar" "$run/tmp"
export TMPDIR="$run/tmp"
cd "$root"
if [ -n "${4:-}" ]; then
    tarball=$(realpath "$4")
    test -s "$tarball"
else
    corepack pnpm pack --pack-destination "$run/tar"
    set -- "$run"/tar/*.tgz
    test "$#" = 1 && test -s "$1"
    tarball=$1
fi
src=apps/electron-vite
git -C "$game" show "$rev:$src/src/main/steamInterface.ts" > "$app/src/main/steamInterface.ts"
git -C "$game" show "$rev:$src/src/types.ts" > "$app/src/types.ts"
git -C "$game" show "$rev:$src/electron-builder.yml" > "$app/electron-builder.yml"
git -C "$game" show "$rev:$src/package.json" > "$run/projector.json"
git -C "$game" show "$rev:package.json" > "$run/workspace.json"
cd "$app"
node - "$tarball" <<'JS'
const fs = require('node:fs')
const game = require('../projector.json')
fs.writeFileSync('package.json', JSON.stringify({
    name: 'deviation-steam-consumer', private: true, version: '1.0.0', main: 'main.cjs',
    packageManager: require('../workspace.json').packageManager,
    dependencies: { 'steamworks.js': 'file:' + process.argv[2] },
    devDependencies: { electron: game.devDependencies.electron },
}))
JS
corepack pnpm --ignore-workspace --config.node-linker=hoisted install --ignore-scripts
node -e "const a=require('node:assert/strict'); a.equal(require('steamworks.js/package.json').name,'@kyeshmz/steamworks.ts'); a.equal(require.resolve('steamworks.js'),process.cwd()+'/node_modules/steamworks.js/index.js')"
"$game/node_modules/.bin/tsc" --strict --skipLibCheck --noEmit --esModuleInterop --types node,electron --target es2022 --module node16 src/main/steamInterface.ts
"$game/node_modules/.bin/esbuild" src/main/steamInterface.ts --bundle --platform=node --format=cjs --external:steamworks.js --outfile=adapter.cjs
cp "$root/test/deviation-game/main.cjs" .
version=$(node -p "require('electron/package.json').version")
test "$(node -p "require(process.argv[1]).version" "$game/node_modules/electron/package.json")" = "$version"
"$game/node_modules/.bin/electron-builder" --linux dir --x64 --config electron-builder.yml \
    --config.electronDist="$game/node_modules/electron/dist" --config.electronVersion="$version" \
    --config.directories.output="$run/built" --publish never
bundle="$run/built/linux-unpacked"
if [ -n "${CHROME_DEVEL_SANDBOX:-}" ]; then
    helper=$(realpath "$CHROME_DEVEL_SANDBOX")
    test -f "$helper" && test "$(stat -c '%u:%a' "$helper")" = 0:4755 || { echo "Expected a root-owned, mode 4755 Chromium sandbox helper" >&2; exit 1; }
    mv "$bundle/chrome-sandbox" "$run/chrome-sandbox-original"
    ln -s "$helper" "$bundle/chrome-sandbox"
fi
sdk="$bundle/resources/app.asar.unpacked/node_modules/steamworks.js/dist/linux64"
for file in steamworksjs.linux-x64-gnu.node libsteam_api.so; do
    cmp "$sdk/$file" "node_modules/steamworks.js/dist/linux64/$file"
done
unset E2E_DEBUG_STEAM STEAM_API_KEY STEAM_VERIFICATION_SECRET LD_PRELOAD LD_LIBRARY_PATH
unset FIXTURE_CASE FIXTURE_RESTART
if [ "$mode" = fixture ]; then
    fixture=$(sh "$root/test/deviation-native/run.sh" "$app/node_modules/steamworks.js" sdk-path)
    cp "$fixture" "$sdk/fixture.so"
    mv -f "$sdk/fixture.so" "$sdk/libsteam_api.so"
    export FIXTURE_RESTART=0 FIXTURE_CASE=deviation-game
fi
printf 'Testing game %s with packaged Electron %s at %s\n' "$rev" "$version" "$bundle"
timeout 30s xvfb-run -a "$bundle/Deviation Game" --mute-audio --user-data-dir="$run/profile"
