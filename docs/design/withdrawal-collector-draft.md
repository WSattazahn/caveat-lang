# Withdrawal collector — reviewable draft contract

Status: **implementation authorized for review; semantic acceptance pending**,
2026-10-07. The owner explicitly authorized work on
`codex/withdrawal-collector-draft`, created from
`f5ec8294efe2be24705f234ef75e5f5459aa5e89`. PR #174 and its baseline remain
separate. This grant permits a reviewable implementation and its evidence; it
does not approve merge, release, a universal boundedness claim, or every
provisional choice below. Earlier pasted external review text was feedback,
not the implementation grant.

The implementation source is frozen for verification at
`c1fe15cfcb1cac6c069ba5b00f1c7238df7e9853`; later documentation receipts do not
change that measured source identity. The
[source inventory](../../experiments/departure-gate/WITHDRAWAL-ROOT-INVENTORY.md)
records baseline consumers and mutation sites. The earlier
[reachability proposal](../../experiments/departure-gate/WITHDRAWAL-REACHABILITY-DESIGN.md)
and [measurements](../../experiments/departure-gate/withdrawal-retention-results.json)
remain historical design evidence from their recorded dirty build. They are
not test results for this collector. The accepted
[departure](../../spec/caveat-departure-0.1.md),
[lineage compaction](../../spec/caveat-lineage-compaction-0.1.md),
[withdrawal](../../spec/caveat-withdrawal-0.1.md) and
[windows](../../spec/caveat-windows-0.1.md) requirements remain the baseline;
the proposed differences are named here for review.

## Problem and success criterion

A valid self-withdrawal or mutual withdrawal cycle can retain retired records
solely through reason pins. In contrast, a real path from a current record,
own ground, permission, pending qualification or later executable read must
retain the exact subjects and reasons it needs. The draft should release an
unreachable group in the event that makes it collectible, without inventing
reasons, weakening grounds, suppressing qualifications, changing current
decision status or using host archive data during execution.

This is a conditional release rule, not a proof that windowed programs use
bounded space. A reachable reason chain can grow. So can deliberately retained
own grounds, independently required history, and conservatively protected
regions. Archive storage is separate from live session storage and still grows
when a host retains history.

## Provisional decisions implemented for review

### D1. Additional candidates leave ordinary departure unchanged

`D0` is the complete set produced by existing ordinary departure, including
its same-event withdrawal cascade. The collector considers an additional set
`C` of retired, departable records in an affected withdrawal dependency region.
It does not make declarations, current occurrences, live window members,
non-window histories or decision revisions newly departable.

Existing non-withdrawal pins retain their exact scope and multiplicity:
state own grounds; own grounds of in-force commitments; `because` and
`permitted_by` of non-retired journal entries; scheduled targets; and grants
of **every held permission record**. Inherited provenance is not silently
promoted to own grounds. One holder's release does not erase another's pin.

A withdrawal creates a directed subject → concrete-reason dependency. Its
existence does not independently root either endpoint. A required/reachable
subject retains the exact true reason, any withdrawal of that reason, and
other required dependencies transitively. An incoming reason path from a
retained subject outside a candidate group prevents collection. A self-edge
or mutual internal edges alone do not.

The additional group can include a record that is not itself withdrawn but
is retained solely as a reason of an unreachable withdrawn subject. This is
still withdrawal-driven collection: ordinary zero-pin eligible records are
already in `D0`, and any non-withdrawal pin vetoes membership in `C`.

`D0` and `C` are planning sets. Publish one globally ordered departure batch,
not separate batches whose order depends on work-queue traversal. Resolve
relationships before deleting their subjects/reasons.

### D2. Transferred provenance causes conservative candidate-only vetoes

The draft conservatively protects exact evidence referenced by independently
retained computational holders: state lineage and grounds, commitment bases
and grounds, stream/series selection qualifications, pending guards and
qualifiers owned by source declarations. This covers both a direct later
withdrawal lookup and a transfer that might reach a later history read,
predicate, decision, guard or explicit citation.

This protection applies **only to deciding whether a record belongs to `C`**.
It does not add global permanent pins, alter `own_evidence`, or stop `D0` from
compacting the provenance it is already permitted to compact. A protected
record remains protected while the actual holder/reference persists; replacing
or clearing that holder must remove the corresponding protection and queue an
affected region for reconsideration. Failure to clear an index entry is a
retention bug, not a new semantic root.

Metadata owned by a candidate record is conditional: its exact reading
provenance, qualifiers and withdrawal reason are outgoing dependencies of that
owner. They are not external roots merely because a metadata map stores them.
A live/current/nondepartable owner remains required. A group whose only owner
paths are internal can still be collectible. Unknown or unproved transfer
paths conservatively veto their affected additional candidates until covered;
the draft must report that limit rather than claim collection of every
unreachable cycle.

Markers and archive references are historical representations, not executable
exact-name edges. Stale binding caches, cues already delivered to a host and
host snapshots are not roots. The semantic inputs of recomputed bindings still
are covered by their real holders. No host archive lookup participates in
source execution or retention.

This provisional rule deliberately over-retains some histories. Optimizing a
veto away requires a separate consumer/transfer argument and paired test; no
CPython analogy or reference-count argument substitutes for that proof.

### D3. Same-event transaction and restore

Evaluate due actions and authored effects as before. At the existing
end-of-effects, pre-binding departure boundary, compute the final eligible
batch, retain exact archive payloads, compact permitted references and remove
the records. Recompute affected bindings, settle archive provenance and publish
only if the event succeeds. A failure during effects **or final bindings** must
restore numeric values, graph, qualifications, pins, proposed indexes/queues,
current selections, numbering, decision status and preexisting undrained
archive exactly.

Do not defer a proved collectible group to a later event or add a new rejection
to avoid collector work. An unknown-path veto means the group was not proved
collectible; it is not a time-budget deferral. Runtime implementation failure is
not permission to return an ordinary accepted/rejected result with lost data.

Restore reconstructs derived indexes once from validated held records. It
departs nothing on load, starts with an empty archive and preserves valid older
cyclic saves. Candidates may depart on the next accepted event; a refused event
cannot migrate a save. Do not change the save schema or weaken forged-save
validation merely to accommodate a new index. Any necessary change must be
identified separately and reviewed with positive older-save and negative
forged-save cases.

### D4. Exact archive data and host rendering

Each collected subject retains its original withdrawal
`{evidence,because,sequence,event}` in the archive, with its reading data,
qualifications and removed relations. A complete matching archive must allow
the true relationships to be joined across records, including self-links,
mutual links and nested provenance of one departing owner naming another.
An archive entry is not assumed to be a closed per-record graph. Original
ordering and concrete occurrence identities survive; reasons are never
replaced with a summary name or current alias.

For this draft, host `explain`/`dependents` should be able to render archived
withdrawal relationships when provided sufficient matching archive data,
including the reverse reason relationship. Render archived results as
historical, with retirement/departure metadata, rather than implying that they
are current permission or decision state. Current executable predicates and
dispatch must remain independent of that rendering.

The helper contract must distinguish exact, unavailable and conflicting
evidence. Missing ancestors or conflicting duplicate records must not produce
an exact claim. Do not infer exact membership from a marker range; do not use
an unrelated history/record merely because its name matches. Validate source,
history, occurrence and sequence consistency using the available snapshot and
archive metadata. Hash agreement checks consistency, not authenticity or a
unique session history: two branches of the same program can share source and
occurrence names. The precise same-source branch boundary must be stated in
the API record and tests, not concealed by an `exact` label.

Baseline `archiveResolver` supplies exact membership names only; it does not
reconstruct historical withdrawal lists. New rendering and cross-record joins
are therefore a reviewable API change. The narrower draft API keeps existing
live lists unchanged and adds a separate optional historical section:

```text
archive?: {
  scope: "provided records and their referenced closure",
  authenticated: false,
  records: [{record, status: "complete" | "unavailable",
             dependencies: string[], unresolved: string[], entry?: ArchiveRecord}],
  unresolved: string[],
  withdrawals: Withdrawal[],
  reasonForWithdrawals?: Withdrawal[],
  relations: [{from, relation, to}]
}
```

`complete` means the named supplied record and its referenced closure can be
joined consistently. It does not mean all departed records of the session were
supplied or their content authenticated. An omitted record might hold an
otherwise undiscoverable incoming relation; this scoped result cannot prove
that no such relation exists. Missing/conflicting dependencies or
unresolvable marker membership propagate `unavailable` through dependent
records, including cycles. Raw entries and derived historical relationships
are exposed as resolved only for complete rows. A separate historical heading
in text output makes this boundary visible. No supplied archive keeps baseline
live behavior; a baseline empty live-withdrawal list is not a claim that a
departed record was never withdrawn.

For `explain`, the section covers supplied records and referenced marker
members. For `dependents`, historical withdrawal/reverse-reason/typed-relation
results match the query, while the record-closure scope remains explicit.
Family and caveat queries use actual identities and typed relations, not
numeric-range guesses. The draft joins reasons, relation endpoints, frozen
reading provenance, qualification metadata and journal references across
records; a missing predecessor is not silently treated as irrelevant.

A name still held by the supplied current snapshot terminates this historical
reference lookup. The helper does not replay that held record's mutable current
metadata into the archived past. An entry's `holders` field is its departure-time
census, not a current executable root or an index of every historical incoming
dependency. These limits are part of the supplied-reference-closure scope.

Disconnected collected cycles may have no live holder and therefore no marker
that anchors their archive entries. The draft emits the existing source-scoped
record-leaf form for **every** departure, including ordinary departures with
no marker holder. This adds archive provenance items, not new save fields or
executable roots. The leaf identifies source, history, occurrence and departure
sequence; it does not hash/authenticate the entire record payload or disambiguate
all branches of the same source. Test and document those limits, rather than
using a matching leaf to overclaim a unique authentic session history.

### D5. Snapshot and outcome compatibility have an explicit domain

Retirement does not globally filter reactive snapshots: held retired symbols,
relations and withdrawals are exported. Collection consequently changes raw
snapshot inventory, `retired`, departure effects, marker/archive fields and
historical explanation representation. Byte-for-byte equality of those
outputs is not a possible collector promise. Current commitment status still
comes from its graph record and current series revision; deleting an old reason
must not clear `open`, retarget permission, or change qualified predicate
results. Snapshot consumers need the documented retired → departed transition.

For event sequences where both versions can admit the same storage allocation,
compare executable values, binding values/caveats, true reasons/own grounds,
current status and permission results, and accepted/refused classification with
origin/code. Compare exact historical dependencies through the complete matching
archive rather than accepting a subset or a marker range as equality.

There is one named **provisional capacity-only difference**: removing extra
records can let a future event fit below `MAX_WINDOWED_RECORDS`, where the
retaining baseline would return its held-history limit refusal. Keep
`history_room` at its existing pre-append point. The collector does not run early
to rescue an event already refused there, change the threshold, or suppress
other refusals. At the limit, compare each implementation against its actual
held count and verify atomic rollback. Final owner acceptance of this scoped
outcome difference remains required; the broad baseline phrase “departure never
changes an outcome” must not be used to hide it.

Programs without any window remain byte-compatible in outcomes, snapshots,
views and saves, with no departures or archive items. Existing ordinary
departure cases must retain their record eligibility, record ordering and
executable behavior even if they have transfer metadata that would conservatively
veto **additional** collection. The per-record archive-leaf addition above is
an intentional archive representation change, not a new eligibility rule.

### D6. Incremental work has a measured, nonconstant worst case

The proposed implementation maintains owner → exact-reference dependencies
and an inverse index. Real create/replace/merge/clear/retire/depart mutation
sites dirty the owners whose references changed. Candidate regions are queued
on relevant retirement, root release and edge/owner changes. One restore/load
rebuild and debug full-index audits are allowed; an unconditional full-session
root/edge reconstruction every accepted event is not this draft.

An affected region can be discovered through incoming/outgoing dependencies,
then tested for directed reachability from independent/external required
holders. Trial reachability or SCC work is `O(V_region + E_region)` plus changed
reference/index-maintenance and archive/compaction work at the graph-visit level.
The implementation's ordered-map/set lookups add logarithmic factors and
string-comparison costs; the visit count is not a linear wall-time guarantee.
The initial draft
indexes use `Arc<BTreeMap<...>>`; the first copy-on-write mutation in a
transaction can clone a whole retained index, including nested maps/sets.
That cost is additional to the region traversal and is not counted by its
vertex/edge counters. Do not describe total event cost as proportional only
to changed references or the affected region. Include a tiny mutation beside
a large unrelated retained region in whole-event allocation/time measurements.
The no-unconditional-full-trace requirement is not a claim of persistent-map
logarithmic updates.

The traversed region itself may be the whole retained graph when one root is
released. No constant-dispatch or universal live-memory bound follows. Debug
audit cost must be separated from release-build measurements, without disabling
its correctness checks.

Report examined vertices/edges, changed references, queued work, retained
records, live-save bytes, archive volume, peak temporary/retained memory where
measurable, and dispatch distributions at increasing history lengths. Distinguish
measurements from asymptotic reasoning and from metrics the implementation
cannot yet expose. Preserve the registered C3 fixture/thresholds and outcome
population; do not replace them with a friendlier cycle-only benchmark.

## Acceptance checklist and evidence record

Every row is **pending draft evidence** at creation of this document. A future
update may mark a row observed only with the exact command, revision/build
identity, fixture, outcome and receipt/report path. A green baseline run or an
old design probe does not discharge a draft row. Failed checks remain recorded.

| ID | Required paired case / observation | Acceptance condition |
| --- | --- | --- |
| C01 | Self and mutual cycles, with no external required read | Old groups collect in the first eligible accepted event; all true reasons remain in exact archive records; current/declared records remain. |
| C02 | Required pin reachability and long reachable reason chain | Own-ground, held-grant and current-record paths retain their transitive true reason closure. Reachable chain growth is reported, not “fixed” by dropping a reason. |
| C03 | Two independent pins; release one, then last | One release retains; accepted final release collects all eligible connected records that same event. Test both self and mutual groups. |
| C04 | Refused final release and final binding failure | Full save/view/status/index/numbering/undrained archive match before refusal; retry succeeds deterministically. |
| C05 | Scheduled target and guard | Target retains through retirement until due; guard dependencies survive transfer; last release can collect only after required dependencies are gone. Failed due event rolls back. |
| C06 | Journal retention and retirement | `because` and grant pins retain while entry is live; retiring journal releases its contributions only; remaining independent pins still retain; archive order remains global. |
| C07 | In-force/superseded commitments and held permissions | Own-ground pin scope and all-held-grant scope remain exact; old bases/grounds conservatively veto extra collection where still readable; decision open/status, permission predicates/refusal remain correct. |
| C08 | Skipped guards, procedure arguments, selection qualifiers | The actual owner/transfer protects candidates; overwriting/clearing the last holder removes the veto. No fabricated observation or inherited-to-own promotion. |
| C09 | Observation/examination/reopening/predicate qualifiers and late qualification | Each persistent owner create/merge/clear path matches a full debug index audit; candidate-owned metadata is conditional and cannot self-root an otherwise dead group. |
| C10 | Later history read and explicit citation | Retained history still adds dynamic `withdrawn` and predicates carry the real reason. Grounded/ungrounded citation decisions are unchanged in the shared admission domain. |
| C11 | Current-save restore and valid rc.15/rc.16 older cyclic saves | No load-time collection or archive publication; deterministic rebuild; next accepted event behavior equals uninterrupted execution; malformed saves remain refused. |
| C12 | Frequent, late and never drain; failed event with preexisting archive | Identical execution/save and concatenated ordered archive across drain schedules; drain storage kept separate from runtime roots; failure preserves prior buffer. |
| C13 | Exact archive joins and host reports | Self/mutual withdrawals, nested record/qualifier dependencies, copied/merged/replaced holder traces and reverse reasons reconstruct exactly with complete matching archive; no partial range presented as exact. |
| C14 | Missing, mismatched, conflicting or branched archive data | Explicit unavailable/conflict behavior; no fabricated complete history; documented source/session authenticity limit; legacy/no-archive behavior covered. |
| C15 | Snapshot/view/decision status and binding cache consumers | Intended representation differences are explicit; retained executable status/grounds/permission/caveats preserved; incremental and full binding evaluation agree. |
| C16 | Ordinary departure and no-window populations | Candidate-only vetoes never alter ordinary compaction; all delivered non-window fixtures remain byte-compatible in the registered comparison. |
| C17 | Storage-capacity boundary | Held-count limit checks remain pre-append and atomic; only the documented capacity difference occurs after earlier accepted collection; no collector refusal/deferral introduced. |
| C18 | Long self/mutual, retained chain, last-pin release, tiny-change/large-unrelated-region, C3 | Current collector build measured at increasing sizes with runtime identity; whole-event COW/allocation cost separated from traversal counters and live versus archive storage; C3 registered save/dispatch and outcome/digest gates pass unchanged. |
| C19 | Required repository/package/browser checks | Run the affected runtime workflow, native and WASM/kit/package gates on the actual draft; report all failures, skipped platforms and browser limitations. |
| C20 | Independent code/source review | Audit every dependency mutation/clear site, conditional ownership, transaction rollback, restore validation, archive joining and limit boundary against this contract. |

Evidence must distinguish native and WASM behavior, debug index verification,
release performance, direct API calls and rendered host explanations. Source
review and tests complement one another; neither proves all future programs
bounded. Final review must decide whether the provisional semantic/API choices
are acceptable and whether known conservative limits are sufficiently explicit.

## Working acceptance evidence map

This is a preliminary map of **specific observed assertions**, not a declaration
that C01–C20 are accepted. Preliminary native tests ran against the evolving
draft based on `f5ec829`. The final source is now frozen at `c1fe15c`; clean-build
artifact identity is captured below. Reruns remain pending only where explicitly
stated. The original failed attempt remains in its log. WASM host-consumer
results and measured profile comparison are recorded separately. Registered
C3/corpus gates, package/Chrome checks and the scoped CAVEAT audit have now
passed locally. Remote c1 CI is reported separately: core semantics, runtime proofs,
reproducible WASM and both worker checks pass. Legacy Door 3D QA failed
(run 37708806805, job 113095624904) because c1 corrupted the workflow
assertion's expected badge encoding. A one-line ASCII JavaScript escape repair
preserves the assertion; WF7 passes the exact local WebKit route harness.
The workflow repair is commit `440ae92`; the old head finished 17/18 checks
with this sole failure. Remote CI for the corrected final head remains pending. No baseline result
substitutes for a collector result.

Clean build identity from `dist/build-info.json`: revision
`c1fe15cfcb1cac6c069ba5b00f1c7238df7e9853`, `clean:true`, `compiled:true`,
runtime source fingerprint
`09cabbda0c0277c991f396357693692fad063145bc594849a2a90b21e7eb16be`.
Full WASM SHA-256 is
`06f6fbac21b9a0e1c066b9dad80a80d89683f4d8b2b796c43e7f9c850857b131`;
reactive-only WASM is
`0554b21cbc24e9d094975ea53be05075ee1a1233a8e6719b4373f00cf8235bdb`.
Compiler: Rust `1.98.1`, host `x86_64-pc-windows-msvc`, wasm-bindgen `0.2.104`.
This identifies the compiled draft, not a release. Draft PR #175 is stacked on
unchanged PR #174; neither this record nor a passing check authorizes merging
or publishing.

Durable copies of the profile, build, release-effects, native summary and
capacity receipts, with reproduction details and their raw hashes, are in
[the Windows c1fe15c evidence directory](../../experiments/departure-gate/collector-results/windows-c1fe15c/README.md).

Receipt identifiers used below:

- **F869:** Final native full-profile run observed **869 passed**, including
  all 20 collector tests and 20 departure tests, in
  `test-results/collector-final-native-full-tests.log` (one ignored capacity
  test; 82 test binaries). Command: set `RUST_MIN_STACK=16777216`, then
  `cargo test --locked --manifest-path runtime/Cargo.toml --all-targets`.
  The large ignored
  capacity gate remains separately required. Strict Clippy passed for full
  and reactive-only profiles (`collector-final-native-full-clippy.log` and
  `collector-final-native-reactive-clippy.log`); formatting passed
  (`collector-final-native-fmt.log`). Commands were
  `cargo clippy --locked --manifest-path runtime/Cargo.toml --all-targets -- -D warnings`,
  the same with `--no-default-features` before `--all-targets`, and
  `cargo fmt --manifest-path runtime/Cargo.toml --all -- --check`.
  Reactive-only tests also passed **857/857**, with one ignored capacity test
  and 82 binaries, using `cargo test --locked --manifest-path runtime/Cargo.toml
  --no-default-features --all-targets` and the same stack setting. The exact
  commands, exit codes and final source hashes are recorded in
  `test-results/collector-final-native-summary.json`; runtime source matches
  `c1fe15c`. These default/reactive feature runs do not replace the
  `collector-metrics` assertions in N20.
  This broad gate supplements the named assertions below; it does not imply
  every acceptance property was tested.
- **N20/O20:** `cargo test --locked --manifest-path runtime/Cargo.toml --features
  collector-metrics --test withdrawal_collection --test departure`, observed
  **20/20 collector and 20/20 departure tests passed** in
  `test-results/collector-draft-focused-final.log`. Collector tests are in
  [`withdrawal_collection.rs`](../../runtime/tests/withdrawal_collection.rs).
  Departure tests are in [`departure.rs`](../../runtime/tests/departure.rs).
  This is the final debug working-source run whose files are included in
  `c1fe15c`, not a substitute for the clean artifact receipt.
- **Earlier attempts:** `test-results/collector-draft-focused.log` preserves
  the earlier collector attempt with **9/11 passed and two failures**: an incorrect
  `hud` assertion path and an invalid fixture attempting permission with an
  already-withdrawn grant, alongside 20 passing departure tests. The later
  collection log records 19 passing tests. N20 adds the restored uncreated
  commitment-owner regression. The original mixed log is not an all-green gate.
- **U68:** `test-results/collector-draft-consumers.log` observed **68/68 passed**
  across `decision_journal`, `grounds`, `late_qualification`, `permission`,
  `qualification_dependencies`, `reactive_procedures`, `windows` and `withdrawal`.
  Invocation: `cargo test --locked --manifest-path runtime/Cargo.toml --features
  collector-metrics --test withdrawal --test qualification_dependencies --test
  late_qualification --test decision_journal --test permission --test
  reactive_procedures --test windows --test grounds`. Only named assertions
  below contribute to each row; F869 reran these functional assertions at the
  frozen source.
- **K1:** The explicit ignored release test
  `capacity_refusal_is_preappend_and_atomic_then_prior_collection_frees_room`
  in [`withdrawal_capacity.rs`](../../runtime/tests/withdrawal_capacity.rs)
  passed at the real **65,536 held-record threshold**, using a synthesized
  source-possible **14,306,948-byte** save through the normal restore validator.
  Final `test-results/collector-final-capacity.log` records **1/1 passed in
  17.33s**, using `cargo test --release --locked --manifest-path runtime/Cargo.toml
  --no-default-features --features collector-metrics --test withdrawal_capacity
  -- --ignored --nocapture`. The earlier 19.69s pre-freeze result remains in
  `collector-draft-capacity.log`; it is superseded for final-source verification.
  The synthetic large save is not an authenticated historical run.
- **H13:** `node --test C:/Dev/caveat-lang/kit/test/archive-history.test.mjs
  C:/Dev/caveat-lang/kit/test/archive.test.mjs`, independently observed **13/13
  passed** with Node `v24.11.1`. Private receipts under
  `C:/Dev/GPT_SandBox_Web/.cache/caveat-continuation/` are
  `collector-review-archive-helper.log` and
  `collector-review-archive-helper-receipt.json`; the latter records the exact
  command, UTC times and unchanged before/after SHA-256 hashes for both helper
  implementations and tests. These are synthetic host-helper tests, not WASM
  collector execution.
- **W56:** Actual clean collector reactive-WASM host checks passed **56/56**
  in private `collector-kit-focused.log` under the same Vessel cache directory.
  Command: `node --test kit/test/archive-history-runtime.test.mjs
  kit/test/archive-runtime.test.mjs kit/test/archive-history.test.mjs
  kit/test/archive.test.mjs kit/test/archive-report.test.mjs
  kit/test/explain.test.mjs kit/test/dependents.test.mjs kit/test/types.test.mjs
  kit/test/serve.test.mjs`. It uses the clean `c1fe15c` reactive WASM hash recorded
  above. The three actual-collector tests named below passed, including real
  CLI/authoring/serve paths; existing sparse/copy/replace/overlap/family archive
  cases passed too. T329 records full kit completion; BPK records installed
  package and actual Chrome execution.
- **T329:** `npm run test` in `C:/Dev/caveat-lang/kit` passed **329/329**, with
  zero failed/skipped tests, in **118.711s** against the clean built runtime.
  Private `collector-kit-full-unrestricted.log` records the authorized rerun.
  The original `collector-kit-full.log` preserves eight sandbox permission
  failures (`EPERM`/Windows temporary-directory access); the rerun passed
  without source changes. This is not browser rendering or package validation.
- **P62:** Native release profiles compare clean `f5ec829` and clean `c1fe15c`
  with the same harness SHA-256
  `1e6de5ad17ca4c6b06e2eef91c7ff65f73774e0cdb9a6d4fa73af1053c0b1357`.
  Commands: `node experiments/departure-gate/collector-profile.mjs
  --only=baseline --baseline-root=test-results/collector-profile-smoke/baseline-source
  --output=test-results/collector-profile-final`, then the same with
  `--only=candidate`. The harness verifies the reused baseline source's Git
  blobs. Five workloads at 60/300/1000/3000 cycles, three trials each, plus two
  late-drain runs produce **62 passing paired rows**. Reports are
  `test-results/collector-profile-final/{baseline,candidate,comparison}.json`.
  The comparison proves its named fixture assertions, not corpus-wide outcome
  equivalence. Executable hashes and all per-trial measurements are retained.
- **E8:** `test-results/collector-release-effects.json` reports **eight passing
  actual-WASM rows** (self/mutual at 60/300/1000/3000): release, drain, exact
  restore and following accepted no-op, using the clean reactive artifact
  identified above. It isolates serialized last-event effect bytes from
  retired-record counts; it is not a heap measurement.
- **M8:** Eight final native allocation-state rows in
  `test-results/collector-release-memory-reproduced/report.json` verify unchanged
  `c1fe15c` runtime/build inputs and the standard build fingerprint before and
  after execution. Reproduce with `node experiments/departure-gate/collector-memory.mjs
  --output test-results/collector-release-memory-reproduction`. This separately
  hashed supplemental harness was added after the runtime source freeze. Each
  row has zero retired dynamic records/effects after the accepted no-op,
  identical logical save hashes across restore, and zero residual requested
  bytes after dropping the final session. A durable copy is
  `collector-results/windows-c1fe15c/release-memory.json` under the departure gate.

- **G15:** Registered `node experiments/departure-gate/run.mjs` passes on
  clean c1 artifacts: 146 no-window programs (116 executed, 30 skipped) with
  exact rc.15 dispatch/save/restored-save digests; 15 windowed fixtures × 32
  seeds yield 480 matching outcome rows. The C3 fixture has 44,460 matching
  accepted outcomes and 2,646-byte adapter saves at 30/60 cycles. The final
  recorder rerun `run.mjs --skip-sweeps` reuses the protected native sweep
  receipts and supplies the consistent frozen C3/restore/pin batch here; late
  C3 median/p99 are 115.1/252.1µs. Exact published rc.15 restore reaches all 61
  source-derived state-write paths; five programs complete the registered pin
  census. Public `departure-gates.json` and its component JSON reports retain
  that batch. Path/guard checking is not an instruction-coverage trace.
- **BPK:** `PLAYWRIGHT_CHANNEL=chrome node scripts/test-kit-package.mjs`
  passes the installed CLI/library/guides/Python/browser checks. The private
  1,011,990-byte tarball has SHA-256
  `8aa06bcd179dfaebfac45a70f28566c0af451fb468b0468f874d9edd3fb89f83`.
  Node 24.11.1 and actual Chrome 154.0.8037.98 additionally agree on mutual
  reasons, nested reading/withdrawal joins, restore, missing-archive handling
  and unchanged live lists. Public `kit-verification.json`,
  `package-report.json` and `installed-smoke.json` record these observations.
  The 57-test Python caller has one documented POSIX-only child skip; its
  separate 21-test surface passes. This is execution/report-consumer evidence,
  not a visual-design acceptance claim. The temporary package staging was
  removed; no release was published.
- **A2:** The bounded local CAVEAT audit at 2026-10-08T00:58:29.849Z returned
  literal `accepted:true` at private store sequence 2. It reran 50 named
  current-WASM/archive-consumer tests and the registered gate command with
  protected native sweep inputs. Public `audit-plan.json` declares its inputs
  and commands; `caveat-audit-summary.json` exposes status, scope, limitations
  and command/output hashes without private keys, ledger or output text. This
  redacted summary is not a standalone authenticated proof. Native/profile
  checks, package/browser evidence and remote CI are separate from its scope.

- **WF7:** The c1 remote Legacy Door 3D QA failure was caused by an
  accidentally misencoded expected badge in `.github/workflows/runtime.yml`,
  not an observed DOM mismatch. Replacing the middle dot in the JavaScript
  assertion with ASCII `\u00b7` preserves the exact expected value. The
  extracted corrected workflow script passes all seven routes in WebKit 26.0
  on Windows and captures ten screenshots. Public `door3d-qa.json` records
  workflow/script hashes and the successful corruption-negative assertion.
  Screenshots remain in the local QA artifact directory. This workflow-only
  correction is later than c1 and does not change measured runtime inputs;
  repair commit `440ae92` records the one-line fix. Local success does not
  substitute for the pending corrected-head CI run.

- **Q192:** The independently reviewed
  [`current-computation.mjs`](../../experiments/departure-gate/current-computation.mjs)
  pairs the pinned clean f5 reactive WASM (`daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59`)
  with clean c1 (`0554b21cbc24e9d094975ea53be05075ee1a1233a8e6719b4373f00cf8235bdb`).
  All 15 registered fixtures × 32 runs × 400 attempts pass: 480 full initial
  snapshots and 192,000 current-computation/outcome comparisons, comprising
  142,393 accepted and 49,607 refused attempts. Baseline has 45,247 departures;
  candidate has 113,856. Public `current-computation.json` records exact
  projection paths, exclusions, sources, artifacts, harness hash and per-run
  input/result digests. All 480 per-run accepted/refused counts also match the
  registered native sweep. State/binding values, caveats, actual own grounds,
  current decision/permission fields, current history members, cues and
  scheduled scalar fields compare with types, array order and negative zero
  preserved. Full historical provenance and raw retired/departed inventories
  are excluded; this is not proof of every future trace or unselected field.
  Both artifacts, all fixture bytes and the harness remain unchanged through
  execution. The diagnostic driver is later than c1, not a runtime change.

| Row | Specific observed evidence | Remaining evidence / scope limit |
| --- | --- | --- |
| C01 | N20 `isolated_self_and_mutual_cycles_depart_with_both_directions_archived`: 16 cycles each, exact archived self/mutual reasons, one leaf per record and only declaration retirement remaining. | P62 extends isolated self/mutual to 3,000 cycles with zero retired dynamic records; W56/E8 exercise actual WASM. This remains finite fixture coverage. |
| C02 | N20 `reachable_chain_survives_and_unchanged_events_do_not_trace_it`; `permission_grants_and_historical_commitment_bases_keep_exact_reasons`; self/mutual own-ground release tests. | P62 preserves the entire reachable-chain save and reports its growing memory/cost; conservative retention and its overhead remain an owner tradeoff. |
| C03 | N20 `final_independent_root_release_collects_and_failures_roll_back` and `a_mutual_group_waits_for_two_roots_and_archives_atomically_after_retry` retain after first release, collect after final accepted release. | F869 reruns these finite functional cases at frozen source; semantic acceptance remains separate. |
| C04 | The same N20 self/mutual tests cover explicit refusal and final-binding failure; mutual case preserves a preexisting undrained batch through retry. `failed_due_event_keeps_pending_target_and_guard_indexes` covers a failed due event. | P62 measures repeated large-region final-binding failures with unchanged save/no archive. G15 adds 480 passing classified-outcome rows; finite fixtures do not prove arbitrary schedules. |
| C05 | N20 `pending_target_pins_survive_until_the_last_scheduled_application`, `pending_guard_lineage_and_its_transfer_are_conservative_roots`, and `failed_due_event_keeps_pending_target_and_guard_indexes`; O20 `each_scheduled_target_pins_until_its_own_qualification_applies`. | F869 reruns these named paths at frozen source; no claim about every schedule. |
| C06 | N20 `journal_retirement_releases_only_its_pin_while_other_owners_retain`; U68 `the_journal_records_each_decision_change_in_order`, `the_journal_keeps_its_newest_entries`; O20 archive ordering assertions. | Under D2, commit/reopen references remain in a held basis/graph and grants in held permission records. A cycle's final *journal-only* root is therefore not manufactured as a fixture; retirement must preserve these other roots. G15 includes the registered windowed outcome population; it does not independently compare every possible journal payload. |
| C07 | N20 `permission_grants_and_historical_commitment_bases_keep_exact_reasons`, `reopened_commitment_keeps_reason_graph_and_current_status`; O20 `permission_grants_remain_pinned_after_a_revision_is_superseded`; U68 `permission_withdrawn_asks_about_the_grant_recorded_on_the_current_revision`, `a_missing_withdrawn_or_mismatched_grant_is_refused_and_the_session_goes_on`. | N20 explicit-citation checks pass; G15 adds paired baseline classified outcomes within its named population. |
| C08 | N20 `transferred_skipped_guard_lineage_vetoes_only_additional_collection`, `procedure_arguments_and_skipped_procedure_guards_keep_their_actual_holders`, `stream_and_series_selection_holders_veto_until_successful_overwrite`; series case correctly retains transferred basis after selection reset. | F869 reruns these finite functional cases at frozen source; semantic acceptance remains separate. The test does not falsely demand clearing a still-held basis. |
| C09 | N20 `a_skipped_uncreated_commitment_is_a_real_owner_on_restore_and_clear`, `record_owned_qualification_inside_a_cycle_is_not_an_external_root`, `declaration_qualifiers_are_roots_and_successful_predicate_clear_releases`, `reopened_commitment_keeps_reason_graph_and_current_status`; U68 `everything_built_on_the_evidence_gains_the_caveat` and `qualifying_is_idempotent_reported_and_atomic`. Debug events audit collector indexes against rebuilt maps. | Mutation inventory review complements these finite paths; universal path coverage is not claimed. |
| C10 | N20 `later_readable_history_preserves_the_reason_until_the_holder_departs` checks later withdrawn predicate, reason retention and exact archived reading provenance; `explicit_citations_keep_exact_old_membership_and_refuse_current_alias_substitution` accepts the held old member, refuses current-alias substitution with `evaluation/ungrounded_citation`, preserves save on refusal and collects after clear. U68 `reading_withdrawn_history_again_keeps_the_withdrawal`. | F869 reruns these finite citation paths. G15 compares its registered corpus outcomes, not a newly claimed baseline pair for every citation assertion. |
| C11 | N20 `authentic_older_saves_rebuild_without_collecting_until_an_accepted_event` uses captured `f5ec829` **and published rc.15** self/mutual saves, preserves save/no archive on load and refused event, then collects on accepted no-op. Other N20 cases restore current saves. O20 forged marker/inherited tests; U68 forged retirement/permission/withdrawal tests. | Frozen collector source and F869 are recorded. G15 additionally restores the published rc.15 C3 save across all 61 source-derived state-write paths. Each old fixture retains its own runtime identity; the versions are not conflated. |
| C12 | N20 `isolated_self_and_mutual_cycles_depart_with_both_directions_archived` compares per-event drain against late drain for identical save and concatenated archive. Mutual retry test preserves preexisting archive. O20 `the_archive_hands_each_departed_record_over_once` and `transfer_metadata_is_transactional_and_read_queries_do_not_publish_it`. | W56/E8 observe actual WASM drain/restore. P62 self/mutual 1,000-cycle per-event versus late drains have identical final save and archive NDJSON hashes. |
| C13 | N20 `ordinary_departing_holder_releases_a_cycle_in_the_same_batch`, conditional-qualification archive assertion and exact self/mutual reasons; O20 `archive_roots_reconstruct_post_departure_copies_replacements_and_overlap`; H13 `archive history joins mutual withdrawals, once-stored relations and nested payload markers without changing live reports`. | W56 passes real collector nested closure and CLI/authoring/serve paths. BPK executes installed-package mutual/nested historical reports, restore and missing archive cases in actual Chrome and Node with equal results. |
| C14 | H13 missing/conflicting/source mismatch cases, malformed-object regressions, future occurrence check including `journal`-named stream, duplicate/cycle/copy determinism, and `consistent payload edits and omitted unrelated records cannot prove authenticity or global completeness`. | W56 adds runtime-produced missing-dependency and restore/no-archive cases. BPK passes package and actual Chrome consumers; authentication/exhaustiveness remains explicitly unsupported. |
| C15 | N20 later-read and reopened-commitment tests assert binding/status preservation and snapshot equality after restore. Debug `run_event` invokes `check_incremental_bindings` against full evaluation, including these success/failure paths. H13 compares historical-report current fields with no-archive current fields. | W56 verifies current live report fields and restored snapshot reports. BPK preserves live report lists; G15 adds complete no-window save/dispatch digests. Q192 independently pairs f5/c1 actual WASM for all 15 registered windowed fixtures: 480 full initial snapshots and 192,000 exact current-computation/outcome comparisons pass. The report lists every selected field and omitted provenance/history field; this closes the finite population gap, not a general equivalence proof. Raw retired/departed equality is outside D5. |
| C16 | O20 existing ordinary departure assertions; N20 `ordinary_departing_holder_releases_a_cycle_in_the_same_batch`; U68 `a_program_without_a_window_saves_no_retired_field`. | G15 passes the complete delivered no-window population: 146 programs, 116 executed/30 skipped, exact dispatch/save/restored-save digests; 15 windowed fixtures produce 480 matching outcome rows. |
| C17 | K1 final-source release gate exercises normal restore, classified `limit/renewal_limit` refusal with unchanged save and empty archive at 65,536 held records, then accepted no-op collection and a later successful allocation retry. Source review confirms `history_room` remains pre-append. | Final explicit gate passed; capacity-only outcome difference remains a named semantic choice requiring owner acceptance. |
| C18 | N20 retained-chain idle test observes zero vertices/edges/candidates on unchanged events with metrics enabled. D6 identifies COW copying separately from these counters. | P62 completes five workloads/three repetitions with requested Rust heap, dispatch peaks, actual work-set counts, unrelated tiny renewal and failed release. E8 explains last-event save growth. G15 passes the registered C3 gate and M8 records retained allocation capacity after effects clear. Collector-only temporary bytes are not isolated; no universal bound follows. |
| C19 | F869 plus 857 reactive-only tests, strict Clippy/fmt, W56 and T329 observed. Strict NodeNext/type checks passed. | BPK package/actual Chrome checks and A2 scoped audit pass. WF7 passes the exact WebKit harness after repairing c1's wrongly encoded expected badge literal. Remote corrected-head CI remains pending; owner semantic/cost acceptance is still required. |
| C20 | Final source diff `f5ec829..c1fe15c` independently reviewed. Resolved graph-owner kind question; corrected rebuild ReliesOn coverage, pending-guard compaction and pre-symbol predicate-owner rebuild. Independent malformed archive and future ordinal regressions pass H13. No new actionable issue found in this pass. | Local artifact checks are complete as recorded in G15/BPK/A2; review is bounded by inspected consumers and cases, not a universal correctness proof. |

The executed actual-WASM cases are in
[`archive-history-runtime.test.mjs`](../../kit/test/archive-history-runtime.test.mjs):
`actual collector self/mutual cycle archives preserve both true reasons and independent live statuses`,
`actual simultaneous reading/withdrawal departures join nested payload references across records`,
and `CLI, authoring and explicit serve archive queries carry collector history without server retention`.
W56 records execution of all three on the identified clean collector WASM; BPK
adds installed-package execution in Node and Chrome.

## Decisions still gated after a passing draft

### Measured costs for the owner's contract review

P62 and E8 make several tradeoffs concrete. These are Windows single-host
measurements with counting-allocator/counter overhead, not portable latency
guarantees or a universal boundedness proof.

| Property | Observed result | Review consequence |
| --- | --- | --- |
| Isolated self/mutual growth | At 3,000 cycles, zero retired dynamic records. Native profiled saves are 965/1,177 bytes respectively, versus 619,712/1,238,671 baseline bytes. | The draft removes these unreachable groups under its conservative contract. It does not establish that every program is bounded. |
| Required reachable chain | At 3,000 cycles, 5,998 retired dynamic records remain; both versions save exactly 1,238,709 bytes with equal save hashes. Requested retained Rust heap is 13,413,126 candidate versus 6,004,653 baseline bytes. | Exact true reasons survive; indexes add substantial live-memory cost where collection cannot help. |
| Tiny unrelated renewal beside that chain | Across three trials, candidate median 6.052–7.949ms; baseline 4.019–6.739ms. Candidate visits zero region vertices/edges but peaks at 7,652,584 additional requested heap bytes, versus 4,865,371 baseline. | Region counters alone understate whole-event work. Existing ordinary departure plus transaction/index copying still scale with retained state; the timing ranges are descriptive. |
| Synchronous final release of mutual groups | The three 3,000-cycle final-release samples are 89.876, 97.889 and 115.684ms; each collects 5,998 records, visits 5,998 vertices / 11,996 edges, peaks at 14,996 work-set entries and 31,128,319 additional requested heap bytes. | Same-event atomic collection can be expensive. Three samples do not estimate a stable p99. Whole-dispatch peak bounds but does not isolate collector temporary allocations. |
| Immediate post-release serialization | E8's actual-WASM mutual 3,000 release has zero retired dynamic records but a 298,403-byte save containing 5,998 departure effects. After the next accepted no-op, it saves 711 bytes and zero effects. The native profiled variant saves 298,583 immediately because it adds probe source/state. | Last-event effects remain observable/saveable until the next event. A small retired set does not imply an immediately small save; clearing effects early would be a separate semantics change. |

Draining the host archive is independent of these live/save observations. E8's
mutual 3,000 handover contains 5,998 records and 5,998 provenance nodes totaling
3,183,055 serialized bytes. That exact historical storage still grows if retained
by a host. M8 shows that after the next accepted no-op, zero retired dynamic
records/effects can still retain **2,669,587 requested Rust heap bytes** for the
3,000 mutual case, versus **43,018** after restoring the identical logical save
and dropping the original. Self measures **1,353,448** versus **40,071**. All
eight runs return to zero residual requested session bytes after final drop.
Source review identifies retained vector capacities, but the probe does not
isolate every container's share. This is resident allocation cost, not retained
logical records or an automatic-restore recommendation. The native injected
no-op name produces a 713-byte mutual save versus E8's 711-byte WASM save;
distinct source hashes make these different workloads, not a compatibility
mismatch. Saved bytes alone do not establish released allocation capacity.

### Remaining approval boundary

The owner still reviews the candidate/transfer-veto boundary, conservative
over-retention, synchronous worst-case cost, archived withdrawal rendering API,
same-source branch consistency boundary and capacity-only compatibility
exception. Any save-schema or validator change needs a separately identified
decision. Passing this checklist does not automatically authorize merge,
release, new series windows or a claim that all windowed histories are bounded.
