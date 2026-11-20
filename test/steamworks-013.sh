#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
for suite in v013-callbacks v013-content v013-net v013-remote v013-server-browser v013-stats; do
    sh "test/$suite/run.sh"
done
sh test/v013-content/run.sh timeout
