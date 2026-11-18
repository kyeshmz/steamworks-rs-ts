#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
test "$(uname -s)" = Darwin || { echo "Game server SDK fixture requires macOS" >&2; exit 1; }
cargo build --locked --offline
root=target/game-server-fixture
dist=$root/dist/osx
mkdir -p "$dist"
addon="$dist/steamworksjs.darwin-$(node -p process.arch).node"
cp target/debug/libsteamworksjs.dylib "$addon"
otool -L "$addon" | grep -q '@loader_path/libsteam_api.dylib'
cp index.js "$root/"
cc -std=c11 -c test/game-server-fixture/sdk.c -o "$root/sdk.o"
nm -u "$addon" | sed -n 's/^_\(Steam[A-Za-z0-9_]*\)$/\1/p' | LC_ALL=C sort -u > "$root/imports"
nm -gUj "$root/sdk.o" | sed 's/^_//' | LC_ALL=C sort -u > "$root/exports"
LC_ALL=C comm -23 "$root/imports" "$root/exports" | awk '{print "void " $1 "(void) { __builtin_trap(); }"}' > "$root/unimplemented.c"
cc -dynamiclib "$root/sdk.o" "$root/unimplemented.c" -o "$dist/libsteam_api.dylib"
node test/game-server-fixture/check.js
