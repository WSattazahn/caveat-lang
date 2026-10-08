# Withdrawal collector — reviewable draft contract

Status: **implementation authorized for review; semantic acceptance pending**,
2026-10-07. The owner explicitly authorized work on
`codex/withdrawal-collector-draft`, created from
`f5ec8294efe2be24705f234ef75e5f5459aa5e89`. PR #174 and its baseline remain
separate. This grant permits a reviewable implementation and its evidence; it
does not approve merge, release, a universal boundedness claim, or every
provisional choice below. Earlier pasted external review text was feedback,
not the implementation grant.

The [source inventory](../../experiments/departure-gate/WITHDRAWAL-ROOT-INVENTORY.md)
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
that C01–C20 are accepted. Native tests below ran against the evolving dirty
draft based on `f5ec829`; a frozen source identity and final rerun remain pending.
The original failed attempt remains in its log. Current collector WASM,
package/browser, baseline comparison and performance results are not yet recorded
here. No baseline result is substituted for a collector result.

Receipt identifiers used below:

- **N19:** `cargo test --locked --manifest-path runtime/Cargo.toml --features
  collector-metrics --test withdrawal_collection`, observed **19/19 passed** in
  `test-results/collector-draft-collection-tests.log`. Tests are in
  [`withdrawal_collection.rs`](../../runtime/tests/withdrawal_collection.rs).
  The log identifies a debug native binary; its final source hash is pending.
- **O20:** The departure portion of
  `test-results/collector-draft-focused.log` observed **20/20 passed** in
  [`departure.rs`](../../runtime/tests/departure.rs). The same log also preserves
  the earlier collector attempt with **9/11 passed and two failures**: an incorrect
  `hud` assertion path and an invalid fixture attempting permission with an
  already-withdrawn grant. N19 records the corrected tests; this mixed log is
  not an all-green gate or final source receipt.
- **U68:** `test-results/collector-draft-consumers.log` observed **68/68 passed**
  across `decision_journal`, `grounds`, `late_qualification`, `permission`,
  `qualification_dependencies`, `reactive_procedures`, `windows` and `withdrawal`.
  Only named assertions below contribute to each row. Exact invocation and
  final source identity still need to be attached.
- **H13:** `node --test C:/Dev/caveat-lang/kit/test/archive-history.test.mjs
  C:/Dev/caveat-lang/kit/test/archive.test.mjs`, independently observed **13/13
  passed** with Node `v24.11.1`. Private receipts under
  `C:/Dev/GPT_SandBox_Web/.cache/caveat-continuation/` are
  `collector-review-archive-helper.log` and
  `collector-review-archive-helper-receipt.json`; the latter records the exact
  command, UTC times and unchanged before/after SHA-256 hashes for both helper
  implementations and tests. These are synthetic host-helper tests, not WASM
  collector execution.

| Row | Specific observed evidence | Remaining evidence / scope limit |
| --- | --- | --- |
| C01 | N19 `isolated_self_and_mutual_cycles_depart_with_both_directions_archived`: 16 cycles each, exact archived self/mutual reasons, one leaf per record and only declaration retirement remaining. | Long runs and actual WASM/host reports pending; this is finite native coverage. |
| C02 | N19 `reachable_chain_survives_and_unchanged_events_do_not_trace_it`; `permission_grants_and_historical_commitment_bases_keep_exact_reasons`; self/mutual own-ground release tests. | Long retained-chain memory/latency and paired baseline comparisons pending. |
| C03 | N19 `final_independent_root_release_collects_and_failures_roll_back` and `a_mutual_group_waits_for_two_roots_and_archives_atomically_after_retry` retain after first release, collect after final accepted release. | Final frozen-build receipt pending. |
| C04 | The same N19 self/mutual tests cover explicit refusal and final-binding failure; mutual case preserves a preexisting undrained batch through retry. `failed_due_event_keeps_pending_target_and_guard_indexes` covers a failed due event. | Classified baseline outcome comparison and final measured large-region failed release pending. |
| C05 | N19 `pending_target_pins_survive_until_the_last_scheduled_application`, `pending_guard_lineage_and_its_transfer_are_conservative_roots`, and `failed_due_event_keeps_pending_target_and_guard_indexes`; O20 `each_scheduled_target_pins_until_its_own_qualification_applies`. | Final frozen-build receipt pending; no claim about every schedule. |
| C06 | N19 `journal_retirement_releases_only_its_pin_while_other_owners_retain`; U68 `the_journal_records_each_decision_change_in_order`, `the_journal_keeps_its_newest_entries`; O20 archive ordering assertions. | Under D2, commit/reopen references remain in a held basis/graph and grants in held permission records. A cycle's final *journal-only* root is therefore not manufactured as a fixture; retirement must preserve these other roots. Broader paired baseline journal population pending. |
| C07 | N19 `permission_grants_and_historical_commitment_bases_keep_exact_reasons`, `reopened_commitment_keeps_reason_graph_and_current_status`; O20 `permission_grants_remain_pinned_after_a_revision_is_superseded`; U68 `permission_withdrawn_asks_about_the_grant_recorded_on_the_current_revision`, `a_missing_withdrawn_or_mismatched_grant_is_refused_and_the_session_goes_on`. | Collector-specific explicit-citation checks and paired baseline classified outcomes pending. |
| C08 | N19 `transferred_skipped_guard_lineage_vetoes_only_additional_collection`, `procedure_arguments_and_skipped_procedure_guards_keep_their_actual_holders`, `stream_and_series_selection_holders_veto_until_successful_overwrite`; series case correctly retains transferred basis after selection reset. | Final frozen-build receipt pending. The test does not falsely demand clearing a still-held basis. |
| C09 | N19 `record_owned_qualification_inside_a_cycle_is_not_an_external_root`, `declaration_qualifiers_are_roots_and_successful_predicate_clear_releases`, `reopened_commitment_keeps_reason_graph_and_current_status`; U68 `everything_built_on_the_evidence_gains_the_caveat` and `qualifying_is_idempotent_reported_and_atomic`. Debug events audit collector indexes against rebuilt maps. | Mutation inventory review complements these finite paths; universal path coverage is not claimed. |
| C10 | N19 `later_readable_history_preserves_the_reason_until_the_holder_departs` checks later withdrawn predicate, reason retention and exact archived reading provenance; `explicit_citations_keep_exact_old_membership_and_refuse_current_alias_substitution` accepts the held old member, refuses current-alias substitution with `evaluation/ungrounded_citation`, preserves save on refusal and collects after clear. U68 `reading_withdrawn_history_again_keeps_the_withdrawal`. | Final frozen-build and paired baseline receipt pending; finite citation paths only. |
| C11 | N19 `authentic_older_saves_rebuild_without_collecting_until_an_accepted_event` uses captured `f5ec829` **and published rc.15** self/mutual saves, preserves save/no archive on load and refused event, then collects on accepted no-op. Other N19 cases restore current saves. O20 forged marker/inherited tests; U68 forged retirement/permission/withdrawal tests. | Final collector source identity pending. Each old fixture retains its own runtime identity; the versions are not conflated. |
| C12 | N19 `isolated_self_and_mutual_cycles_depart_with_both_directions_archived` compares per-event drain against late drain for identical save and concatenated archive. Mutual retry test preserves preexisting archive. O20 `the_archive_hands_each_departed_record_over_once` and `transfer_metadata_is_transactional_and_read_queries_do_not_publish_it`. | Actual collector WASM drain/restore reports and longer drain schedules pending. |
| C13 | N19 `ordinary_departing_holder_releases_a_cycle_in_the_same_batch`, conditional-qualification archive assertion and exact self/mutual reasons; O20 `archive_roots_reconstruct_post_departure_copies_replacements_and_overlap`; H13 `archive history joins mutual withdrawals, once-stored relations and nested payload markers without changing live reports`. | Real collector WASM nested closure, CLI/authoring/serve and rendered historical reports pending. |
| C14 | H13 missing/conflicting/source mismatch cases, malformed-object regressions, future occurrence check including `journal`-named stream, duplicate/cycle/copy determinism, and `consistent payload edits and omitted unrelated records cannot prove authenticity or global completeness`. | Actual runtime-produced archive counterparts and final host integration receipt pending; authentication/exhaustiveness remains explicitly unsupported. |
| C15 | N19 later-read and reopened-commitment tests assert binding/status preservation and snapshot equality after restore. Debug `run_event` invokes `check_incremental_bindings` against full evaluation, including these success/failure paths. H13 compares historical-report current fields with no-archive current fields. | Paired baseline snapshot projection and current WASM consumer evidence pending; raw retired/departed representation equality is intentionally outside D5. |
| C16 | O20 existing ordinary departure assertions; N19 `ordinary_departing_holder_releases_a_cycle_in_the_same_batch`; U68 `a_program_without_a_window_saves_no_retired_field`. | Full registered no-window byte comparison and ordinary baseline population on final collector build pending. A single no-window test is not the corpus gate. |
| C17 | Source review confirms `history_room` remains pre-append; D5 specifies the proposed difference. | Actual full-capacity accepted/refused/rollback boundary test pending. Source review alone does not discharge this row. |
| C18 | N19 retained-chain idle test observes zero vertices/edges/candidates on unchanged events with metrics enabled. D6 identifies COW copying separately from these counters. | Five workloads at 60/300/1000/3000, three repetitions; actual requested Rust live heap/dispatch peak; collector temporary-memory upper bound versus work-set counts; unrelated tiny renewal, failed release and unchanged registered C3 pending. No measured cost claimed yet. |
| C19 | H13 host helper tests observed. Kit owner reports strict NodeNext/type checks, awaiting persisted command/build receipt. | Actual collector WASM tests, full native/kit/package gates and browser evidence pending. |
| C20 | Independent source review resolved graph-owner kind question; rebuild ReliesOn coverage and pending-guard compaction index issues were corrected. Independent malformed archive and future ordinal regressions pass H13. | Final frozen diff/source review and receipt identity pending; review is bounded by inspected consumers and cases. |

The planned actual-WASM cases are in
[`archive-history-runtime.test.mjs`](../../kit/test/archive-history-runtime.test.mjs):
`actual collector self/mutual cycle archives preserve both true reasons and independent live statuses`,
`actual simultaneous reading/withdrawal departures join nested payload references across records`,
and `CLI, authoring and explicit serve archive queries carry collector history without server retention`.
Their existence is not execution evidence.

## Decisions still gated after a passing draft

The owner still reviews the candidate/transfer-veto boundary, conservative
over-retention, synchronous worst-case cost, archived withdrawal rendering API,
same-source branch consistency boundary and capacity-only compatibility
exception. Any save-schema or validator change needs a separately identified
decision. Passing this checklist does not automatically authorize merge,
release, new series windows or a claim that all windowed histories are bounded.
