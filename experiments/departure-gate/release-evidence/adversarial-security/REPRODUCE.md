# Reproduce the bounded adversarial pass

Use a fresh extraction of `Caveat_Adversarial_590fae5.zip`. Node.js 24 and Python 3 were used on Windows. The bundled WASM runs through Node; no Rust, npm installation, network access or original checkout is needed for these probes. No Linux execution of this bundle is claimed.

Read `REVIEW.md`, the three inventories/plans and `identity-notes.md` first. `cells.json` identifies every runtime/package file. Candidate and published rc.15 have different supported APIs and deliberately different lifecycle expectations. Never substitute one version's wrapper for the other's.

## Check preserved results without executing Caveat

```text
python -B review/verify-results.py --check
```

This read-only checker verifies the final report binding, selected source/runtime identities, command streams, result rows, recorded input/output artifacts and preserved preparation failures. It does not load WASM, rerun the probes, authenticate their history or access the original checkout. Run it on the intact extraction before adding new replay outputs.

## Fresh execution

From the extracted bundle root, choose unused output names. The commands below preserve original results and create separate receipts:

```text
python -B record-run.py replay-archive -- node tests/archive-marker-probes.mjs --out replay-archive
python -B record-run.py replay-withdrawal -- node tests/withdrawal-probes.mjs --root . --plan withdrawal-plan-003.json --out replay-withdrawal.json
python -B record-run.py replay-post -- node tests/post-collection-probes.mjs --out replay-post
python -B record-run.py replay-contracts -- node --test cells/candidate/test/archive.test.mjs cells/candidate/test/archive-report.test.mjs cells/candidate/test/archive-history.test.mjs cells/candidate/test/restore-contract.test.mjs
```

Expected final counts are 103 M/A cases, 40 W cell cases plus 20 comparisons of shared fields, 14 P cases and 30 repository tests. All expected malformed-save failures must be classified restore errors; a runtime trap, load failure or arbitrary exception is not a passing refusal. The selected final probes include acceptance controls, so an implementation rejecting everything cannot pass.

The root-level fresh output names above need no parent directory preparation. If you choose nested output paths, create the parent first. Existing outputs and receipts are not overwritten. A failed attempt should remain available when a subsequent correction uses a new name.

## Find a specific reproducer

- M01–M13: the mutated save is `archive-marker-results-001/candidate-Mxx-save.json`; use `fixtures/marker-journal.cav` with the candidate cell's `runtime.restore(source, savedText)`.
- M14/M15: the corresponding saves intentionally restore; accompanying report JSON demonstrates conservative archive reporting.
- A00–A20: each immediate/delayed, live/restored combination retains its supplied `*-archive.json` and `*-report.json`. Use `fixtures/nested-withdrawal.cav`, the preserved nested save and the candidate's public `explain` API. The `z@1` acceptance check prevents a blanket rejection from passing.
- W01–W20: `withdrawal-results-003.json.artifacts/` holds each case's exact source, input/output saves, event outcomes, loaded runtime identity and ledger; the final summary maps IDs to expectations and comparisons.
- P10–P19: `results/post-collection-002/` holds every malformed save and refusal alongside the authentic post-collection save and fixture. P00–P02 preserve valid restore, rollback and rc.15 migration controls.

Authentic controls pass original save text unchanged. JSON parsing and re-encoding is used only for deliberately edited positive/integer fixtures; it is not an exact-save procedure for arbitrary values such as signed zero.

## Preserved history and unavailable inputs

W plans/results 001 and 002 and P command 001 are preparation evidence. Their failures are explicitly classified in the report; the final results do not overwrite or count them as correct malformed-save refusals. Earlier inventory versions are preserved too.

Muse's original F256/F358/F362/F363/F364 source and expectations are absent. `historical-findings.json` records the search boundary; these new tests do not establish that those exact findings were reproduced.

`SHA256SUMS` verifies the delivered files; the sibling package-verification record gives the ZIP hash and validation result. Local receipt exports and hashes do not authenticate execution or external history. Runtime tests remain separate from agent/tool prompt-injection testing.
