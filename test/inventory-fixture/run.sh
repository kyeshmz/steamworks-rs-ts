#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
if [ "$#" = 0 ]; then cargo build --locked --offline; fi
root=target/inventory-fixture
case "$(uname -s):$(uname -m)" in
    Darwin:*) platform=osx; library=libsteam_api.dylib; binary=libsteamworksjs.dylib; addon=steamworksjs.darwin-$(node -p process.arch).node ;;
    Linux:x86_64) platform=linux64; library=libsteam_api.so; binary=libsteamworksjs.so; addon=steamworksjs.linux-x64-gnu.node ;;
    *) echo "Unsupported inventory fixture host" >&2; exit 1 ;;
esac
dist=$root/dist/$platform
mkdir -p "$dist"
cp "${1:-target/debug/$binary}" "$dist/$addon"
cp index.js "$root/"
cc -std=c11 -fPIC -c test/inventory-fixture/sdk.c -o "$root/sdk.o"
if [ "$platform" = osx ]; then
    nm -u "$dist/$addon" | sed -n 's/^_\(Steam[A-Za-z0-9_]*\)$/\1/p' | LC_ALL=C sort -u > "$root/imports"
    nm -gUj "$root/sdk.o" | sed 's/^_//' | LC_ALL=C sort -u > "$root/exports"
    link=-dynamiclib
else
    nm -D --undefined-only "$dist/$addon" | awk '{print $NF}' | sed -n '/^Steam/p' | LC_ALL=C sort -u > "$root/imports"
    nm -g --defined-only "$root/sdk.o" | awk '{print $NF}' | LC_ALL=C sort -u > "$root/exports"
    link=-shared
fi
LC_ALL=C comm -23 "$root/imports" "$root/exports" | awk '{print "void " $1 "(void) { __builtin_trap(); }"}' > "$root/unimplemented.c"
cc "$link" -fPIC "$root/sdk.o" "$root/unimplemented.c" -o "$dist/$library"
node test/inventory-fixture/check.js
