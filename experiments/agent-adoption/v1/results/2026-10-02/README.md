# Public evidence archive: 2026-10-02 agent-adoption study

This is a privacy-filtered derivative of the frozen A01/A02/A03 submission
snapshots and external receipts. Original trial files were not edited. See
[the study results](../../RESULTS.md) for interpretation and the frozen v1
protocol/task/scorer one directory above the results directory.

All three trials had bootstrap working-directory/permission problems that
caused outside-task directory listings or file-name enumeration. This archive
retains those deviations and their event locations, but removes their returned
listings/errors. **These are not clean cold-run observations.** The artifact
rubric is separate from compliance with the procedural scope. Three synthetic
runs from the same configured model family do not establish population or
production reliability. Per-trial backend/model metadata were not reported by
the JSON stream; the equivalent non-JSON preflight is a different observation.

## Contents

- `A01/`, `A02/`, `A03/`: frozen `submission/` files, authored receipts,
  `manual-review.json`, and external results/evaluator/process receipts.
- Each `external/commands.jsonl` includes only `command_execution` events from
  the external CLI stream. `sourceLine` is the original one-based JSONL line;
  item IDs, event order, started/completed status and exits are retained.
  `originalLineSha256` hashes the original event line without its newline.
  Non-command events (including thread IDs and agent messages) are excluded.
- `registration.json`, `run-started.json`, `run-summary.json`, compact package
  and security input reports, and small setup/version/scorer-control receipts.
- `redaction-manifest.json`: every copied source's original and public hashes,
  transformations, designated bootstrap output omissions, and raw transcript
  identities. Raw external CLI streams remain private and are not included.
- `archive-manifest.json`: hashes of all public archive files except itself.

## Integrity and reading the evidence

All authored `.cav` files and every `first.scenarios.json` and
`tracker.scenarios.json` are **byte-identical** to the frozen snapshots. Their
hashes were checked against each original submission manifest. First and final
versions, including failures and repairs, remain distinct where the agent
changed them. No evaluator or trial script was rerun to create this archive.

Machine-specific paths/usernames and process/session identifiers in other files
are replaced with placeholders. `<TRIAL_A01>` (and A02/A03) means that trial's
workspace; `<PACKAGE_A01>` means its installed Caveat package; `<REPO>`,
`<PRIVATE_RUN>`, `<USER_HOME>`, `<NODE>` and `<DRIVE_ROOT>` conceal local paths.
`<ARCHIVE>` denotes this archive. Relative evidence references in manual reviews
are remapped to public archive locations where possible. Archived helper scripts
and command strings can contain placeholders and are evidence, not an assurance
that they can be rerun unchanged.

`registration.sha256`, original external submission manifests, receipt hashes
and internal source hashes describe the **original private bytes**. Sanitized
files may therefore differ from those hashes; use `redaction-manifest.json` for
the explicit original-to-public mapping, and `archive-manifest.json` to verify
public bytes. Original manifests were retained rather than silently rewritten.

The selected outside-task output fields are omitted in full: A01 source lines
5/7/9, A02 6/7/9/11, A03 5/7/11. Nonempty reconstructed A02 bootstrap stream files
are also replaced by omission notices. Other authored bootstrap summaries were
already partial and did not contain the raw inventory; their path metadata is
normalized. A02's large file-name enumeration was truncated upstream, so the
retained raw transcript does not prove what names appeared in discarded output.
The inspected bootstrap command types and retained outputs do not show private
evaluator/gold/other-trial code being read; this is not a guarantee of context
purity or a claim that the scope violation had no influence.

## Deliberate exclusions

No `node_modules`, package tarball, raw external transcript, unfiltered preflight
stream, or redundant full package/security build logs are copied. Their relevant
identities/checks remain in registration and compact reports. The public archive
contains authored receipts (including repeated public-document excerpts) because
they are part of the submitted evidence, not substituted rerun output.

A targeted scan found no high-confidence private-key/API-token markers in the
frozen authored submissions. Path normalization, explicit inventory omission and
hash verification are documented controls, not certification that arbitrary
source or prose can never contain sensitive information. Preserve the private
originals for audit; do not publish them to compensate for redaction.
