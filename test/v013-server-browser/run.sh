#!/bin/sh
set -eu
ulimit -c 0
cd "$(dirname "$0")/../.."
case "$(uname -s):$(uname -m):$(node -p process.arch)" in
    Linux:x86_64:x64) platform=linux64; addon_name=steamworksjs.linux-x64-gnu.node ;;
    Linux:aarch64:arm64) platform=linuxarm64; addon_name=steamworksjs.linux-arm64-gnu.node ;;
    *) echo "Requires native Linux x64 or arm64" >&2; exit 1 ;;
esac
root=$(pwd)
mkdir -p target/v013-server-browser
out=$(mktemp -d "$root/target/v013-server-browser/run.XXXXXX")
mkdir -p "$out/tmp" "$out/package/dist/$platform"
export TMPDIR="$out/tmp" CARGO_NET_OFFLINE=true PATH="$HOME/.cargo/bin:$PATH"
addon="$root/dist/$platform/$addon_name"
cp index.js "$out/package/index.js"
cp "$addon" "$out/package/dist/$platform/$addon_name"
cc -std=c11 -fPIC -DSteamAPI_ManualDispatch_RunFrame=Fixture_Original_RunFrame -c test/deviation-native/sdk.c -o "$out/base.o"
c++ -std=c++11 -fPIC -I native/valve -c test/v013-server-browser/sdk.cpp -o "$out/browser.o"
nm -D --undefined-only "$addon" | awk '{print $NF}' | sed -n 's/^\(Steam[A-Za-z0-9_]*\)\(@.*\)\{0,1\}$/\1/p' | LC_ALL=C sort -u > "$out/imports"
nm -g --defined-only "$out/base.o" "$out/browser.o" | awk '{print $NF}' | LC_ALL=C sort -u > "$out/exports"
LC_ALL=C comm -23 "$out/imports" "$out/exports" | awk 'BEGIN {print "#include <stdio.h>\n#include <stdlib.h>"} {print "void " $1 "(void) { fputs(\"Unexpected SDK call: " $1 "\\n\", stderr); abort(); }"}' > "$out/unimplemented.c"
cc -fPIC -c "$out/unimplemented.c" -o "$out/unimplemented.o"
c++ -pthread -shared "$out/base.o" "$out/browser.o" "$out/unimplemented.o" -o "$out/package/dist/$platform/libsteam_api.so"
timeout --signal=TERM --kill-after=2s 20s node --expose-gc test/v013-server-browser/check.js "$out/package"
