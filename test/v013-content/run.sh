#!/bin/sh
set -eu
cd "$(dirname "$0")/../.."
out="$PWD/target/v013-content"
sh test/linux-sdk-fixture.sh . test/v013-content/sdk.c "$out"
if [ "${1:-}" = timeout ]; then
  pids=""
  for case in content-timeout content-timeout-create content-timeout-query; do
    FIXTURE_CASE="$case" timeout --signal=TERM --kill-after=2s 40s node test/v013-content/check.cjs "$out/package" &
    pids="$pids $!"
  done
  result=0
  for pid in $pids; do wait "$pid" || result=1; done
  exit "$result"
else
  timeout --signal=TERM --kill-after=2s 20s node test/v013-content/check.cjs "$out/package"
fi
