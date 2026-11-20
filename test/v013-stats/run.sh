#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
root=$(pwd)
test "$(uname -s):$(uname -m)" = Linux:x86_64 || { echo "Requires Linux x64" >&2; exit 1; }
mkdir -p "$root/target/v013-stats"
out=$(mktemp -d "$root/target/v013-stats/run.XXXXXX")
sh test/linux-sdk-fixture.sh . test/v013-stats/sdk.c "$out"
FIXTURE_RESTART=0 timeout 45s node test/v013-stats/check.cjs "$out/package"
