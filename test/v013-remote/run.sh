#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
out="$PWD/target/v013-remote"
sh test/linux-sdk-fixture.sh . test/v013-remote/sdk.c "$out"
for case in normal too-many; do
    FIXTURE_CASE="$case" timeout --signal=TERM --kill-after=2s 20s node test/v013-remote/check.js "$out/package" "$case"
done
