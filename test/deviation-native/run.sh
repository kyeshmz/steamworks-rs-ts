#!/bin/sh
set -eu
ulimit -c 0
cd "$(dirname "$0")/../.."
root=$(pwd)
mkdir -p "$root/target/deviation-native"
out=$(mktemp -d "$root/target/deviation-native/run.XXXXXX")
mkdir -p "$out/tmp" "$out/package/dist/linux64"
export TMPDIR="$out/tmp"
unset FIXTURE_CASE
test "$(uname -s):$(uname -m)" = Linux:x86_64 || { echo "Requires Linux x64" >&2; exit 1; }
package=${1:-$root}
package=$(cd "$package" && pwd)
addon="$package/dist/linux64/steamworksjs.linux-x64-gnu.node"
test -f "$package/index.js" && test -f "$addon" || { echo "Missing package loader or Linux addon in $package" >&2; exit 1; }
sh test/linux-sdk-fixture.sh "$package" test/deviation-native/sdk.c "$out"
if [ "${2:-}" = sdk-path ]; then
  echo "$out/package/dist/linux64/libsteam_api.so"
  exit 0
fi
echo "Package: $package"
if [ "${2:-}" = overlap ]; then
  result=0
  for case in overlap-session overlap-web overlap-late-session overlap-late-web overlap-failure overlap-cancel overlap-length; do
    echo "Case=$case"
    FIXTURE_CASE="$case" FIXTURE_RESTART=0 timeout --signal=TERM --kill-after=2s 20s node test/deviation-native/overlap.js "$out/package" "$case" || {
      status=$?
      echo "Public fixture failed for $case, exit=$status (124 means timeout)" >&2
      result=1
    }
  done
  exit "$result"
fi
test -z "${2:-}" || { echo "Unknown mode: $2" >&2; exit 2; }
for restart in 0 1; do
  echo "Restart=$restart"
  FIXTURE_RESTART=$restart timeout --signal=TERM --kill-after=2s 20s node test/deviation-native/check.js "$out/package" "$restart" || {
    status=$?
    echo "Public fixture failed for restart=$restart, exit=$status (124 means timeout)" >&2
    exit "$status"
  }
done
