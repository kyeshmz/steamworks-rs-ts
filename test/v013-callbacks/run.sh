#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
root="$PWD/target/v013-callbacks-fixture"
check=check.cjs
if [ -n "${BASELINE_PACKAGE:-}" ]; then
    root="$root-baseline"
    check=baseline.cjs
fi
sh test/linux-sdk-fixture.sh "${BASELINE_PACKAGE:-.}" test/v013-callbacks/sdk.c "$root"
timeout --signal=TERM --kill-after=2s 20s node --expose-gc "test/v013-callbacks/$check" "$root/package"
