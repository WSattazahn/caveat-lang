#!/usr/bin/env bash
# Run a command that downloads something, such as `rustup toolchain install`,
# up to five times, 20 seconds apart, while it fails. One dropped connection
# to a download server should not fail a job: rustup gives up on the first
# one, and a single connect timeout to static.rust-lang.org failed a job
# while five other jobs of its run fetched the same toolchain. Each retry
# leaves a warning on the run. A command that fails five times, or that is
# missing or cannot run (exit 127 or 126), fails the step with its status.
#   .github/ci/retry-download.sh <command> [args...]
set -uo pipefail
attempts=5
for ((attempt = 1; attempt <= attempts; attempt++)); do
  "$@" && exit 0
  status=$?
  # A command that is missing or cannot run fails the same way each time.
  if [ "$status" -eq 126 ] || [ "$status" -eq 127 ] || [ "$attempt" -eq "$attempts" ]; then
    echo "::error::'$*' failed (exit $status, attempt $attempt of $attempts)"
    exit "$status"
  fi
  echo "::warning::'$*' failed (exit $status, attempt $attempt of $attempts); trying again in 20 s"
  sleep 20
done
