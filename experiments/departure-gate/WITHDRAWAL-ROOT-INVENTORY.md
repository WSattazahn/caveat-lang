# Withdrawal collection: source inventory and narrower review boundary

Review checkpoint: source-consumer, editorial, local-link and independent
source review of baseline `f5ec829` are complete. The runtime reviewer confirmed
the pin categories and the incorporated `history_room` and archive-joining
boundaries. Those checks do not verify subsequent collector changes.

Status: **reviewable implementation draft authorized, 2026-10-07**. After the
design-only review, the owner explicitly authorized implementing a collector
draft on `codex/withdrawal-collector-draft`, starting at `f5ec829`; PR #174 is
preserved. The earlier pasted external review was feedback, not that grant.
Update, 2026-10-08 UTC: the owner accepted repaired `3d77aa` as the experimental
development baseline, including the explicit snapshot, historical-completeness
and capacity-only choices. Merge, publication, release and unrestricted-workload
claims remain unapproved. The separately authorized index-compaction task must
preserve this root/dependency boundary. Decisions and evidence are recorded in
[the collector contract](../../docs/design/withdrawal-collector-draft.md).
This inventory supplements [the earlier reachability proposal](WITHDRAWAL-REACHABILITY-DESIGN.md)
and narrows its sentence about making all retained computational containers'
exact dependencies roots.

Source inspected: `f5ec8294efe2be24705f234ef75e5f5459aa5e89`. The existing
`withdrawal-retention-results.json` is evidence from its recorded earlier dirty
build, not a measurement of this clean commit or an implemented collector. Its
`draftReachability` function is a fixture-only overapproximation; it is not the
root definition specified below. Source-line links below are pinned to the
inspected baseline so draft edits do not silently shift their meaning. Draft
implementation and verification must be recorded separately against their
actual revision/build; the earlier measurements are not collector results.

## Boundary: additional withdrawal collection, not new general pins

Ordinary departure already removes unpinned retired records and compacts their
names out of retained provenance. In particular, state lineage, inherited
grounds, historical commitment bases, reading provenance, selection and
predicate qualifications, scheduled guards and cue qualifications are handled
by [depart_unpinned](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L264), lines 296–480.
Their existence is **not** an existing requirement to retain every named
runtime record. `own_evidence()` excludes inherited evidence
([reactive_expr.rs:248](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_expr.rs#L248)). Making every
such name an unconditional pin would change ordinary departure and can retain
the very histories lineage compaction is intended to release.

The proposed boundary is therefore:

1. Preserve the current ordinary-departure rule, including its exact own-ground,
   journal, scheduled-target and permission pins and its allowed compaction.
   Let `D0` denote the complete departure set that that rule would produce for
   the event, including its existing withdrawal cascade.
2. Consider an **additional** set `C` of currently retained, retired,
   departable records in an affected withdrawal dependency region. Exclude all
   independently required records. A subject's withdrawal supplies a directed
   subject → concrete-reason edge, never an independent root merely because
   the withdrawal exists. Incoming reason edges from retained subjects outside
   `C` prevent collection; mutually internal edges do not prove retention.
3. Permit `C` only after accounting for every independently retained consumer
   that could still use a member or its metadata, including later language
   reads and transfers. For a known required path, retain its exact target and
   transitively required reasons/dependencies. For an unresolved path, veto
   **additional collection of the affected candidates**. Do not install a new
   global permanent pin or prevent a member of `D0` from departing.
4. These sets are a reasoning device, not a proposal to publish two batches or
   delete nodes before inspecting their relationships. Any implementation must
   plan the final `D0 union C`, archive exact data before deleting it, and keep
   the existing global history/occurrence ordering.

Thus a candidate-only veto may conservatively leave some withdrawal cycles
retained. It must disappear or be reconsidered when its actual owner/reference
disappears; it is not a lifetime pin. Proving a transfer path inert could relax
that veto in a separately reviewed refinement. This draft does not authorize
weakening real reasons or teaching language evaluation to read the host archive.

## Roots and edges the current code establishes

The table distinguishes current retention from proposed additional-collection
checks. A metadata field is owned by its record/container: metadata reachable
only from a candidate group must not root that group from outside itself.

| Class | Exact retained data / consumer | Consequence for the proposed collector |
| --- | --- | --- |
| Not eligible for ordinary departure | Live window suffixes; non-window histories; declared renewable first occurrence; current occurrences. `retire_oldest`, `live_start` and `departable`: [reactive.rs:5117](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5117), [reactive.rs:5190](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5190), [reactive_departure.rs:249](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L249). | Never collect them. Current retirement sites cover readings, renewals and windowed journal entries; this proposal adds no retirement of decision revisions or source declarations. |
| Existing independent pins: state | `StateCell.grounds.own_evidence()`: [for_each_pin:134](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L134). | Exact own grounds remain pins. Neither all state lineage nor all inherited grounds become pins. |
| Existing independent pins: decision | Own grounds of in-force commitments. Series revisions other than `current` are excluded: [for_each_pin:139](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L139). | Preserve current scope exactly. Retained superseded bases/grounds still need the later-read checks below; supersession alone does not prove all their metadata dead. |
| Existing independent pins: journal | Every `because` and `permitted_by` name of each **non-retired** journal entry: [for_each_pin:159](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L159). | This list is not filtered by `own_evidence`; preserve all its names and ordering. Retired journal entries do not continue to supply those pins. |
| Existing independent pins: pending action | `scheduled.evidence`: [for_each_pin:166](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L166). | Preserve target until execution. Its guard is separate transfer metadata, not another existing target pin. |
| Existing independent pins: permission | `commitment_permissions[*].grant`, for **every held permission record**, not just the current revision: [for_each_pin:169](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L169). | Preserve the held grant and required withdrawal reason closure. Do not release the grant merely because the decision is superseded. |
| Existing withdrawal pin, proposed conditional edge | Each `Withdrawal { evidence, because, sequence, event }` currently pins `because`: [for_each_pin:172](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L172), [Withdraw:4347](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4347). | Proposed edge is `evidence → because`. A retained subject keeps the exact reason; self/mutual internal edges alone cannot root a group. Preserve each original withdrawal record in its matching archive entry if the group eventually departs. |
| Required reason qualification | Withdrawal predicates call `qualify_core(because)` for grounds or `qualify(because)` for lineage: [withdrawal_predicate:5294](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5294). `qualify` also reads observation and `observed` predicate qualifications: [reactive.rs:3941](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3941). | A reachable reason requires its real caveats/relations and any independently needed qualification dependencies. Follow record-owned metadata only when its owner is retained/reachable; do not globally root all withdrawals' metadata. |

The five non-withdrawal pin categories above are the complete categories in
`for_each_pin`; they are not a complete future-observability proof. Likewise,
the general graph is not by itself the proposed retention graph. A `ReliesOn`
edge represents lineage, including lineage that ordinary departure may compact;
it must not automatically become a permanent runtime pin.

## Source consumers beyond the pin census

| Actual read path | What the code does | Additional-collection obligation / limit |
| --- | --- | --- |
| Source evidence selectors | `occurrence` resolves renewable aliases to newest occurrence; `withdrawal_target` resolves named/current or latest reading; `predicate_target` resolves evidence predicates: [4919](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4919), [4992](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4992), [5305](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5305). | No arbitrary host pointer or identifier-text-to-evidence lookup is exposed by these paths. This only removes a direct-address path; it does not prove a frozen dependency unreachable. |
| Indexed/count/latest history reads | `history_read` takes the live suffix, merges selected reading/decision provenance and selection qualifications, then `with_current_withdrawals`; `latest` does the same for current record: [2984](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L2984), [3077](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3077). | Exact evidence names can be looked up in the current withdrawal map. Protect candidate subjects reached by independently readable holders, including selection qualifications. `later-history-read.cav` proves a concrete case. Retired readings themselves are excluded from the history selection, but can be reached as required reasons or other retained references. |
| Dynamic withdrawal caveat | `with_current_withdrawals` checks `provenance.evidence` against `withdrawal_of` and adds `withdrawn`: [5239](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5239). | It does not consult marker ranges or archive references. A matching external archive cannot substitute for a retained subject in this executable path. A frozen caveat is also not a general substitute for the true withdrawal relation. |
| Withdrawal predicates | `withdrawn`, `withdrawn_latest`, `permission_withdrawn`, `rests_on_withdrawn`: [5252](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5252). The latter reads **all exact evidence** in the current commitment's grounds, not only own evidence. | Preserve targets, truth values, reasons and their qualified provenance. No direct pin census alone covers the inherited-ground part of `rests_on_withdrawn`. |
| Permission check and template sampling | `check_permission` tests current named/latest grant, consults withdrawal record and formats its sequence/reason in refusal: [5034](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5034). `Sample` checks the declared template withdrawal to avoid inheriting its `withdrawn` caveat: [4231](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4231). | Preserve acceptance/refusal code, origin and real reason. Template declarations are not collector candidates, but this lookup belongs in the inventory rather than being silently missed. |
| Direct state reads and explicit citations | `evaluate` copies state value lineage; `evaluate_grounds` copies grounds. `grounded_citation` checks exact evidence/caveat membership against lineage: [3688](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3688), [3727](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3727), [2898](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L2898). | State reads do not themselves query withdrawals. They can transfer exact names into a future sample, guard, decision or citation. Proving that a specific transfer is inert requires source-flow reasoning; absence of a current direct lookup is insufficient. Unresolved transfers veto affected additional candidates, not ordinary compaction. |
| Evidence/caveat and commitment predicates | `qualify_core` inspects evidence/claim qualification edges; `qualify` adds observation dependencies. `predicate_tracked` reads examination metadata, bases, reopening metadata, `Retains`, `ReliesOn`, incoming `Reopens`, and predicate qualifications; `predicate_grounds` reads grounds and real reopening reasons: [3982](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3982), [4015](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4015), [3753](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3753). | Preserve actual caveats, status and reasons. Some fields merely copy frozen provenance while `Reopens` reasons are qualified through current graph data. Do not flatten all these edge types into either permanent pins or discardable history. Unresolved later transfers remain candidate vetoes. |
| Reopen selectors | Named/latest reasons and `caveated(state,caveat)` select actual evidence; caveated selection uses state grounds, `observed` and `qualify_core`, then `qualify`: [Reopen:4727](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4727). | Retired evidence is already not `observed`; collection must preserve the selected live reasons and `empty_caveated_selection` behavior, without reinterpreting old evidence as its current alias. |
| Skipped effects, procedures, reopening triggers | `retain_skipped_effect` writes state lineage, stream/series selection qualifiers or per-target predicate qualifiers; calls recurse. Procedure argument lineage/grounds are frozen and guards propagated; triggers call the same guarded-effect path: [3113](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3113), [4166](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4166), [5004](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5004). | These are real dependency-transfer paths. No procedure-local frame survives the end-of-event collection boundary, but anything copied into a persistent holder does. Unsupported flow/owner classification vetoes the affected candidates. |
| Scheduled guards and late qualification | Scheduling stores `guard union delay.provenance`; due execution passes it to `apply_qualification`, which updates current state lineage/grounds containing the exact target: [4335](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4335), [4853](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4853), [4900](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4900). | Target is already pinned. Guard names are not all existing pins, but can transfer through a future state update. Guard transfer and target release must be reviewed separately. Frozen reading/commitment bases are intentionally not retroactively rewritten by late qualification. |
| Bindings, cues, host snapshots | Bindings evaluate source expressions and explicit citations; rules never read the shown binding caches. Cues/qualifications are cleared at event start: [2719](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L2719), [3446](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3446), [3489](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3489). | Source inputs to bindings matter; stale output caches and host-held snapshots are not semantic roots. Additional departures must invalidate the correct binding groups and preserve current values/caveats/status. Explanation representation may use existing exact archive mechanisms; never silently replace own grounds. |

### Graph export, status and archive consumers

There is no universal retired-record filter on the exported reactive graph.
[`snapshot`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5313) exports the held symbols,
withdrawals, `retired` map and graph relations; its relation `origin: "live"`
is not a test of window membership. [`view`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3601)
exports commitment records and relations through `commitment_records` and
`relation_records`. The [WASM wrapper](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/web.rs#L27) and
[kit session wrapper](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/kit/lib/session.mjs#L163) serialize/return these
outputs rather than filtering retirement. Retirement affects specific reads:
`observed` excludes retired evidence, and history reads use the live suffix.
The windows spec explicitly retains a retired record's graph relations until
departure ([Retirement](../../spec/caveat-windows-0.1.md#retirement)).

Current decision status has separate consumers. Runtime
[`predicate`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3815) tests commitment existence
and `NodeKind::Commitment.open`; qualifying the answer additionally traverses
the metadata described above. Kit [`explain`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/kit/lib/explain.mjs#L129)
uses the series' current revision plus the commitment's `open` field to show
`superseded`, `reopened` or `in force`; journal history is only the fallback
when an older snapshot lacks that commitment record. Removing a historical
reason edge must not reset the commitment's `open` bit or change a predicate's
true qualified reason. `Changes::between` includes graph, symbols, retirement
and the provenance holders used by binding invalidation
([reactive.rs:962](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L962)).

The archive currently provides two distinct things:

- An [`ArchiveEntry`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L590) retains
  the exact withdrawal `{evidence,because,sequence,event}`, removed relations
  and qualifications. Those payloads are available to a host retaining the
  complete matching archive. Simultaneous collector departures would still
  need tests proving both directions of a mutual relationship survive.
  An individual entry is not a closed dependency graph: a departing reading
  or qualification payload can name another departing record. Compaction and
  holder attribution skip those departing owners
  ([reactive_departure.rs:371](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L371),
  [420](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L420)). Their exact nested
  names remain in the payload, but historical traversal must join the relevant
  archive records. Removed graph relations are stored once under a departing
  endpoint ([496](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L496)); the existing
  typed relation families must be respected, and this alone is not evidence
  of a current defect. Withdrawal subject → reason is represented separately
  in `Withdrawal`, not as a `graph.edges` relation.
- [`archiveResolver`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/kit/lib/archive.mjs#L53) verifies/reconstructs
  marker membership and returns record **names**. Kit `explain` uses that to
  annotate lineage markers, but its evidence/decision withdrawal lists read
  `snapshot.withdrawals` ([explain.mjs:91](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/kit/lib/explain.mjs#L91)).
  [`dependents`](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/kit/lib/explain.mjs#L348) likewise builds its
  `withdrawals` and `reasonForWithdrawals` lists from the snapshot; passing
  `archive` does not presently rebuild those lists from archived withdrawals.

Consequently, the proposed collector cannot claim identical raw snapshots or
identical historical explanation lists: a retired record becoming departed is
an observable representation change, accompanied by departure effects and
archived data. It must preserve executable values, status, true reasons,
caveats, own grounds and dispatch classifications under the reviewed storage
contract. Whether existing host helpers must also render archived withdrawal
relationships, beyond exact membership, remains an explicit API/design question
for owner review. A reviewable helper implementation is now authorized under
the separate draft contract; its final API semantics are not approved merely
by that grant. Host-defined
programs that act on raw snapshot inventory are outside the source-language
unreachability proof; the migration contract must describe those representation
changes rather than claiming all possible hosts behave identically.

For self/mutual fixture pairs, no state, history-reading container, scheduled
action, permission or qualifying guard provides an incoming external dependency
to each retired group. For `self-own-ground`, `mutual-own-ground`,
`permission-root` and `later-history-read`, a concrete incoming path does exist.
The reachable-chain fixture has successive subject → reason paths all the way
back from a current record. It remains retained; collecting cycles does not
bound such reachable chains.

## Mutation and index-maintenance inventory

Any derived root/edge index needs create, replace, clear and owner-removal
updates, not just an extra counter in the withdrawal branch. This is the
baseline implementation-review checklist for the authorized draft.

| Mutation boundary | Current source sites and required accounting |
| --- | --- |
| Load/restore | Session initialization calls `rebuild_departure_index` ([reactive.rs:1965](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L1965)); restore validates graph, withdrawals, states, records, retirement, permissions and departures, then evaluates bindings ([reactive_save.rs:666](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_save.rs#L666)). Rebuild proposed derived indexes once from validated held state, without departing anything on load. |
| Root additions/removals | State replacement releases/adds own-ground pins ([4465](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4465)); commit releases superseded own grounds/adds current grounds and a held grant ([4628](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4628), [4684](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4684)); journal append pins and retirement releases its `because`/grant ([4719](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4719), [4843](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4843), [5215](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5215)); scheduling/due execution adds/releases target ([4342](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4342), [4911](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4911)). Preserve multiplicity; releasing one of two independent holders cannot make a group unreachable. |
| Withdrawal edge addition/removal | Successful first withdrawal appends exact record and adds reason pin ([4368](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4368)); ordinary departure releases outgoing reason before completing its cascade, then removes/archive-transfers withdrawal ([reactive_departure.rs:271](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L271), [550](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L550)). Proposed cycle removal must account for all internal/outside edges as one planned group. |
| Owner creation/current selection | Sample installs observed qualifications, current reading, frozen provenance and clears stream selection ([4261](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4261)); renewal creates occurrence and guard metadata ([4391](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4391)); commit installs grounds/basis/relations, current revision and clears series selection ([4640](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4640), [4677](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4677)). Retirement changes direct addressability and enqueues candidates ([5205](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5205)). |
| Metadata merges/replacements/clears | Skipped effects ([3113](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3113)); `clear_predicate_dependency` ([4140](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4140)); observe/reveal ([4485](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4485)); examine ([4529](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4529)); reopen ([4807](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4807)); qualification updates ([4853](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L4853)). Track the exact changed references and conditional owner, including removals after successful effects and additions from failed guards. |
| Ordinary compaction and owner destruction | All provenance holder families in `depart_unpinned` ([296](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L296)); graph edge/node removal ([483](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L483)); per-owner metadata and history removal ([524](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L524)). Retire stale proposed index edges together with the exact references or owners they describe. Archive-marker membership is not a surviving executable exact-name edge. |
| Archive settlement and drain | `settle_archive_provenance` removes transient archive DAG descendants from session provenance ([670](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L670)); `drain_archive` takes the handover buffer ([641](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_departure.rs#L641)). Neither retained archive nodes nor host drain frequency may root runtime records or alter collection decisions. |
| Transaction / output caches | All event effects, ordinary departure, binding evaluation and provenance settlement run in the transaction copy ([run_event:3489](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3489)); commit appends its archive only on success ([apply_classified:3459](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L3459)). Proposed indexes and work queues must be copied/rolled back with it. `Changes::between` and binding dependency invalidation require explicit regression coverage after additional deletion. |

## Same-event timing, costs and remaining decisions

The proposal is synchronous at the existing end-of-effects, pre-binding
departure boundary. Once the reviewed rule establishes an additional group as
collectible, collect it in that event's transaction. A later binding failure or
policy/evaluation refusal restores the entire earlier state, index, ordering,
numbers and undrained archive. Do not defer established collection to later
events, add a new refusal to avoid the work, or perform a whole-session trace
on every event. An unresolved proof veto means that the group was **not**
established collectible; it is not a time-budget deferral.

An incremental affected-region design needs incoming/outgoing dependency
information, changed-root bookkeeping and candidate updates at the sites
above. Trial reachability or SCC work can be `O(V_region + E_region)` with
comparable temporary storage; the affected region can be the **whole retained
graph** after one root release. Maintaining the index also costs the number of
changed references. These are algorithmic targets, not measured collector
performance, and do not imply that existing event operations outside collection
are constant time. A full rebuild during load/restore or a debug audit differs
from an unconditional full trace on every accepted event. C3 and the registered
incremental pin-check requirement remain mandatory.

The initial authorized draft uses copy-on-write `Arc<BTreeMap<...>>` indexes.
A small indexed mutation can copy a whole retained map and nested references
inside the transaction. The [draft cost contract](../../docs/design/withdrawal-collector-draft.md#d6-incremental-work-has-a-measured-nonconstant-worst-case)
therefore requires whole-event measurements, including a small affected region
beside a large unrelated retained region; traversal counters alone do not
measure that copying cost.

Concrete remaining review gaps:

- **Transfer completeness:** the inspected baseline has no consumer-sensitive
  dependency index distinguishing inert frozen provenance from a name that can
  reach a later sample, predicate, citation or qualifier. This addendum locates
  the paths but does not prove all program-specific flows. The conservative
  candidate veto is required until that proof/index is specified; removing it
  would exceed this design.
- **Conditional metadata ownership:** a source declaration's qualifier can be
  read independently; a retired candidate's qualifier may be read only through
  a reachable reason/reference. Procedure/guard, examination, reopening and
  predicate qualifier combinations need paired fixtures. Globally rooting
  every qualifier map would be incorrect as the proposed minimal boundary.
- **Graph/status and output equivalence:** source locations are now identified
  above; the universal-export-filter assumption is explicitly excluded. Reason
  qualification, reopening relations, binding invalidation and exact archive
  reconstruction still need paired before/after collector tests, including a
  failure during final binding evaluation. Historical withdrawal rendering from
  archives is an unresolved helper/API boundary, not a missing raw archive
  field. Earlier source review supplied no collector-test evidence; draft
  results belong in the separate acceptance record.
- **Restore validation:** current departure-aware saves require held retired
  non-declaration records to have an existing pin
  ([reactive_save.rs:819](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive_save.rs#L819)); older saves
  may enqueue unpinned records for their next accepted event. Cyclic reason pins
  are valid current saves. A future derived index must rebuild deterministically
  and accept those saves without collection on load. Do not assume a validator
  relaxation is needed, and do not relax forged-save checks preemptively; prove
  any necessary format/validation change with paired old/new saves first.
- **Archive closure:** simultaneously collected self/mutual relationships,
  record-owned qualifications, copied/merged/replaced holder traces and true
  reason identities must reconstruct from a complete matching archive. Existing
  membership resolution does not itself join cross-record nested provenance
  or render historical withdrawal relationships. Define batch-level joining
  and indexing (or separately review a schema/consumer change) and test those
  cross-record dependencies. Archive resolution cannot serve as a runtime
  lookup fallback. Drain schedules must yield the same language behavior,
  save and ordered archive.
- **Storage-limit outcomes:** `history_room` checks held-record capacity before
  adding a record ([reactive.rs:5142](https://github.com/WSattazahn/caveat-lang/blob/f5ec8294efe2be24705f234ef75e5f5459aa5e89/runtime/src/reactive.rs#L5142)).
  Additional departure may allow a later event that the retaining baseline
  would refuse at that limit. The comparison domain and intended treatment of
  that storage-limit difference need review; do not claim unrestricted outcome
  equality or invent a new refusal to cap collector work. Other dispatch
  classifications and rollback obligations remain unchanged.
- **Total retained-state bound:** reachable chains and conservative vetoed
  regions may keep growing. This proposal supplies no universal bound on live
  records, dispatch, heap or save size; window sizes alone do not supply one.

Owner review is still required for the final candidate boundary, completeness
of the proof/fixtures, conservative veto policy, synchronous worst-case cost
and any eventual save-format implications. Implementation of the reviewable
draft is authorized; merge and release remain gated on that review and the
separate acceptance record.
