#!/bin/sh
set -eu
case "$(uname -s):$(uname -m):$(node -p process.arch)" in
    Linux:x86_64:x64) platform=linux64; addon=steamworksjs.linux-x64-gnu.node ;;
    Linux:aarch64:arm64) platform=linuxarm64; addon=steamworksjs.linux-arm64-gnu.node ;;
    *) echo "Fixture requires native Linux x64 or arm64" >&2; exit 1 ;;
esac
package=${1:?Expected a built package directory}
fixture=${2:?Expected the SDK fixture C file}
out=${3:?Expected the fixture output directory}
dist="$out/package/dist/$platform"
mkdir -p "$dist" "$out/tmp"
export TMPDIR="$out/tmp"
cp "$package/index.js" "$out/package/index.js"
cp "$package/dist/$platform/$addon" "$dist/$addon"
cc -std=c11 -fPIC -c "$fixture" -o "$out/sdk.o"
export LC_ALL=C
nm -D --undefined-only "$dist/$addon" | awk '{print $NF}' | sed -n 's/^\(Steam[A-Za-z0-9_]*\)\(@.*\)\{0,1\}$/\1/p' | sort -u > "$out/imports"
nm -g --defined-only "$out/sdk.o" | awk '{print $NF}' | sort -u > "$out/exports"
comm -23 "$out/imports" "$out/exports" | awk 'BEGIN {print "#include <stdio.h>\n#include <stdlib.h>"} {print "void " $1 "(void) { fputs(\"Unexpected SDK call: " $1 "\\n\", stderr); abort(); }"}' > "$out/unimplemented.c"
cc -pthread -fPIC -shared "$out/sdk.o" "$out/unimplemented.c" -o "$dist/libsteam_api.so"
