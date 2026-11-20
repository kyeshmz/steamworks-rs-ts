#!/bin/sh
set -eu
ulimit -c 0
cd "$(dirname "$0")/../.."
case "$(uname -s):$(uname -m):$(node -p process.arch)" in
    Linux:x86_64:x64|Linux:aarch64:arm64) ;;
    *) echo "Requires native Linux x64 or arm64" >&2; exit 1 ;;
esac
mkdir -p target/v013-net
cc -std=c11 -fPIC -shared test/v013-net/sdk.c -o target/v013-net/libnetwork-fixture.so
LD_PRELOAD="$PWD/target/v013-net/libnetwork-fixture.so" node test/v013-net/check.js
