#!/bin/sh
set -eu
ulimit -c 0
cd "$(dirname "$0")/../.."
test "$(uname -s):$(uname -m)" = Linux:x86_64 || { echo "Requires Linux x64" >&2; exit 1; }
root=$(pwd)
mkdir -p target/v013-server-browser
out=$(mktemp -d "$root/target/v013-server-browser/run.XXXXXX")
mkdir -p "$out/tmp" "$out/package/dist/linux64"
export TMPDIR="$out/tmp" CARGO_NET_OFFLINE=true PATH="$HOME/.cargo/bin:$PATH"
addon="$root/dist/linux64/steamworksjs.linux-x64-gnu.node"
cp index.js "$out/package/index.js"
cp "$addon" "$out/package/dist/linux64/steamworksjs.linux-x64-gnu.node"
cc -std=c11 -fPIC -DSteamAPI_ManualDispatch_RunFrame=Fixture_Original_RunFrame -c test/deviation-native/sdk.c -o "$out/base.o"
c++ -std=c++11 -fPIC -I native/valve -c test/v013-server-browser/sdk.cpp -o "$out/browser.o"
nm -D --undefined-only "$addon" | awk '{print $NF}' | sed -n 's/^\(Steam[A-Za-z0-9_]*\)\(@.*\)\{0,1\}$/\1/p' | LC_ALL=C sort -u > "$out/imports"
nm -g --defined-only "$out/base.o" "$out/browser.o" | awk '{print $NF}' | LC_ALL=C sort -u > "$out/exports"
LC_ALL=C comm -23 "$out/imports" "$out/exports" | awk 'BEGIN {print "#include <stdio.h>\n#include <stdlib.h>"} {print "void " $1 "(void) { fputs(\"Unexpected SDK call: " $1 "\\n\", stderr); abort(); }"}' > "$out/unimplemented.c"
cc -fPIC -c "$out/unimplemented.c" -o "$out/unimplemented.o"
c++ -pthread -shared "$out/base.o" "$out/browser.o" "$out/unimplemented.o" -o "$out/package/dist/linux64/libsteam_api.so"
timeout --signal=TERM --kill-after=2s 20s node --expose-gc test/v013-server-browser/check.js "$out/package"
