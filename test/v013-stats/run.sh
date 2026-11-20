#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
root=$(pwd)
mkdir -p "$root/target/v013-stats"
out=$(mktemp -d "$root/target/v013-stats/run.XXXXXX")
sh test/linux-sdk-fixture.sh . test/v013-stats/sdk.c "$out"
FIXTURE_RESTART=0 timeout 45s node test/v013-stats/check.cjs "$out/package"
