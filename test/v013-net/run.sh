#!/bin/sh
set -eu
ulimit -c 0
cd "$(dirname "$0")/../.."
test "$(uname -s):$(uname -m)" = Linux:x86_64 || { echo "Requires Linux x64" >&2; exit 1; }
mkdir -p target/v013-net
cc -std=c11 -fPIC -shared test/v013-net/sdk.c -o target/v013-net/libnetwork-fixture.so
LD_PRELOAD="$PWD/target/v013-net/libnetwork-fixture.so" node test/v013-net/check.js
