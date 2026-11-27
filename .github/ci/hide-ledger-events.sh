#!/usr/bin/env bash
# A job that can carry round 1's result (scripts/ci-reuse/README.md) skips
# itself when a push only adds lines to the ledger's events, so what it does
# must never depend on them. In each full run, put them out of the job's
# reach:
# - The file leaves the checkout, so reading it fails and a walk of the
#   folders does not find it.
# - Git gives one placeholder line for its contents, so a read through git
#   (git show, git archive, git grep, git cat-file) gets that line and never
#   the events.
# HEAD and the index still name the real contents, and git skips the missing
# file (skip-worktree), so `git status` stays clean and the build's record of
# the revision and of a clean tree does not change.
set -euo pipefail
events=ledger/events.jsonl
real=$(git rev-parse --verify "HEAD:$events")
placeholder=$(printf '%s\n' "$events is hidden from this job: .github/ci/hide-ledger-events.sh" | git hash-object -w --stdin)
git replace -f "$real" "$placeholder"
git update-index --skip-worktree -- "$events"
rm -- "$events"
# A git that ignored the replacement would still show the events.
if [[ "$(git cat-file blob "HEAD:$events")" != "$(git cat-file blob "$placeholder")" ]]; then
  echo "::error::git still shows $events to this job"
  exit 1
fi
echo "Took $events out of this checkout and out of git's reach; this job must not read it."
