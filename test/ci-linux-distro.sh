#!/bin/sh
set -eu

if [ "$#" -ne 5 ]; then
    echo "Usage: $0 <tarball> <image> <os-id> <version-id|-> <node-major>" >&2
    exit 2
fi

tarball=$(realpath "$1")
image=$2
os_id=$3
version_id=$4
node_major=$5
case "$node_major" in
    ''|*[!0-9]*) echo "Node major must be a number" >&2; exit 2 ;;
esac

root=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
node=$(realpath "$(command -v node)")
mkdir -p "$root/target/platform-ci/linux"
scratch=$(mktemp -d "$root/target/platform-ci/linux/run.XXXXXXXX")
tar -xzf "$tarball" -C "$scratch" --no-same-owner
test -s "$scratch/package/package.json"
test -s "$scratch/package/dist/linux64/steamworksjs.linux-x64-gnu.node"
test -s "$scratch/package/dist/linux64/libsteam_api.so"
echo "Tarball $(sha256sum "$tarball")"
echo "Extracted package $scratch/package"

docker run --rm --network none --platform linux/amd64 \
    --mount "type=bind,src=$node,dst=/ci-node,readonly" \
    --mount "type=bind,src=$scratch/package,dst=/ci-package,readonly" \
    --mount "type=bind,src=$root/test/package-smoke.js,dst=/ci-package-smoke.js,readonly" \
    --mount "type=bind,src=$root/test/networking-sockets.js,dst=/ci-networking-sockets.js,readonly" \
    --entrypoint /bin/sh "$image" -eu -c '
        ulimit -c 0
        . /etc/os-release
        cat /etc/os-release
        test "$ID" = "$1" || { echo "Unexpected distro ID: $ID" >&2; exit 1; }
        actual_version=${VERSION_ID:-}
        expected_version=$2
        if [ "$expected_version" = - ]; then
            test "${BUILD_ID:-}" = rolling || { echo "Expected rolling release" >&2; exit 1; }
        else
            case "$actual_version" in
                "$expected_version"|"$expected_version".*) ;;
                *) echo "Unexpected distro version: $actual_version" >&2; exit 1 ;;
            esac
        fi
        test "$(uname -m)" = x86_64 || { echo "Expected x86_64" >&2; exit 1; }
        /ci-node -e "
            const assert = require(\"node:assert/strict\")
            assert.equal(process.platform, \"linux\")
            assert.equal(process.arch, \"x64\")
            assert.equal(process.versions.node.split(\".\")[0], process.argv[1])
            const glibc = process.report.getReport().header.glibcVersionRuntime
            assert.ok(glibc, \"Expected glibc runtime\")
            console.log(\"Node \" + process.versions.node + \", ABI \" + process.versions.modules + \", N-API \" + process.versions.napi + \", glibc \" + glibc)
        " "$3"
        /ci-node /ci-package-smoke.js /ci-package
        /ci-node /ci-networking-sockets.js /ci-package/dist/linux64/steamworksjs.linux-x64-gnu.node
    ' sh "$os_id" "$version_id" "$node_major"
