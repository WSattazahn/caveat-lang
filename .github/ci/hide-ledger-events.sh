#!/usr/bin/env bash
# A job that can carry round 1's result (scripts/ci-reuse/README.md) skips
# itself when a push only adds lines to the ledger's events, so it must never
# read them. In each full run, take the file out of the checkout: a read then
# fails in that run instead of passing unseen. Git keeps the file as it is in
# the commit (skip-worktree), so `git status` stays clean and the build's
# record of a clean tree does not change.
set -euo pipefail
events=ledger/events.jsonl
git update-index --skip-worktree -- "$events"
rm -- "$events"
echo "Took $events out of this checkout; this job must not read it."
