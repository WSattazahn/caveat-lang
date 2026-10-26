#!/usr/bin/env bash
# Run a browser QA command once more if it fails or passes its time limit,
# leaving a warning on the run so a flake stays visible. A real failure
# fails both attempts.
#   .github/ci/retry-once.sh <minutes per attempt> <command> [args...]
set -uo pipefail
minutes="$1"; shift
timeout --kill-after=30s "${minutes}m" "$@" && exit 0
status=$?
echo "::warning::'$*' failed (exit $status); trying once more"
timeout --kill-after=30s "${minutes}m" "$@"
