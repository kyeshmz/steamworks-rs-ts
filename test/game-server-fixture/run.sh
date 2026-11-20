#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
if [ "$#" = 0 ]; then cargo build --locked --offline; fi
root=target/game-server-fixture
case "$(uname -s)" in
Darwin)
    dist=$root/dist/osx
    mkdir -p "$dist"
    addon="$dist/steamworksjs.darwin-$(node -p process.arch).node"
    cp "${1:-target/debug/libsteamworksjs.dylib}" "$addon"
    otool -L "$addon" | grep -q '@loader_path/libsteam_api.dylib'
    ;;
Linux)
    test "$(node -p process.arch)" = x64
    dist=$root/dist/linux64
    mkdir -p "$dist"
    addon="$dist/steamworksjs.linux-x64-gnu.node"
    cp "${1:-target/debug/libsteamworksjs.so}" "$addon"
    ;;
*) echo "Unsupported fixture platform" >&2; exit 1 ;;
esac
cp index.js "$root/"
cc -std=c11 -fPIC -c test/game-server-fixture/sdk.c -o "$root/sdk.o"
if [ "$(uname -s)" = Darwin ]; then
    nm -u "$addon" | sed -n 's/^_\(Steam[A-Za-z0-9_]*\)$/\1/p' | LC_ALL=C sort -u > "$root/imports"
    nm -gUj "$root/sdk.o" | sed 's/^_//' | LC_ALL=C sort -u > "$root/exports"
else
    nm -D -u "$addon" | awk '{print $NF}' | grep '^Steam[A-Za-z0-9_]*$' | LC_ALL=C sort -u > "$root/imports"
    nm -g --defined-only "$root/sdk.o" | awk '{print $NF}' | LC_ALL=C sort -u > "$root/exports"
fi
LC_ALL=C comm -23 "$root/imports" "$root/exports" | awk '{print "void " $1 "(void) { __builtin_trap(); }"}' > "$root/unimplemented.c"
if [ "$(uname -s)" = Darwin ]; then
    cc -dynamiclib "$root/sdk.o" "$root/unimplemented.c" -o "$dist/libsteam_api.dylib"
else
    cc -shared -fPIC "$root/sdk.o" "$root/unimplemented.c" -o "$dist/libsteam_api.so"
fi
node --expose-gc test/game-server-fixture/check.js
