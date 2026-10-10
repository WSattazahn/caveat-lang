# CAVEAT Withdrawal Collection 0.1 — when an unreachable withdrawal group leaves

Status: specification of behavior that shipped in 0.1.0-rc.16. This document
makes the rule a supported contract for 1.0 ([Path to 1.0](../docs/releases/path-to-1.0.md),
blocker B1). It replaces nothing: it restates, as specification, the
experimental contract the owner accepted on 2026-10-08 UTC in
[the collector draft](../docs/design/withdrawal-collector-draft.md), which keeps
its history and evidence. Where the two differ, this file decides.

It extends [Departure 0.1](caveat-departure-0.1.md) and relies on
[Withdrawal 0.1](caveat-withdrawal-0.1.md),
[Windows 0.1](caveat-windows-0.1.md) and
[Lineage Compaction 0.1](caveat-lineage-compaction-0.1.md). It applies only to
programs with at least one window. A program without a window is unaffected.

## The problem

Departure removes a retired record once nothing pins it. A standing withdrawal
pins its reason ([Departure 0.1](caveat-departure-0.1.md), "When a record
departs"). So a record that withdraws itself, or two records that withdraw each
other, pin each other forever, even when nothing current can ever read them
again. Their saves grow with play (rc.16 measured 1.24 MB at 3,000 mutual
cycles without collection).

Collection removes such a group in the event that makes it unreachable. It
never removes a record that something current still needs.

## The rule

At the end of an accepted event, ordinary departure computes its set `D0` as
[Departure 0.1](caveat-departure-0.1.md) specifies, including its cascade over
pins that departure itself releases. Collection then adds a set `C`:

1. **Who can be in `C`.** Only retired, departable records of a windowed
   history in a withdrawal dependency region. A declaration, a current
   occurrence, a live window member, a record of a history without a window or
   a decision revision is never added by collection.
2. **What roots a record.** Every pin of ordinary departure keeps its exact
   scope: state own grounds, own grounds of in-force commitments, `because`
   and `permitted_by` of journal entries that have not retired, scheduled
   targets, and the grants of every held permission record. A record with any
   of these pins is not in `C`. Inherited provenance is not promoted to an own
   ground.
3. **Withdrawals are edges, not roots.** A withdrawal is an edge from its
   subject to its concrete reason. A subject that is required keeps its reason,
   any withdrawal of that reason, and what those require, transitively. A path
   into a group from a retained record outside it keeps the whole group.
   Edges inside a group, including a self-edge, do not keep it.
4. **Conservative vetoes.** A record referenced by a retained computational
   holder (state lineage or grounds, a commitment's basis or grounds, a stream
   or series selection's qualifications, a pending guard or qualifier owned by a
   declaration) is not added to `C` while that holder holds the reference. The
   veto applies only to membership in `C`. It is not a pin, and it changes
   neither `D0` nor what ordinary departure may compact. When the holder is
   replaced or cleared, the veto goes with it.
5. **Record-owned metadata.** A candidate's own reading provenance,
   qualifiers and withdrawal reason are its outgoing edges. They do not root it.
6. **Unknown paths keep records.** Where no rule above proves that a group is
   unreachable, the group stays. Collection may over-retain. It never
   under-retains.
7. **One batch.** `D0` and `C` depart as one batch in the global history and
   number order of [Departure 0.1](caveat-departure-0.1.md). The order of
   internal work does not show in effects, the archive or reports.

## Transaction

Collection runs at the departure boundary, before bindings are recomputed. If
any later step of the event fails, the event is refused and everything is
restored exactly: values, graph, qualifications, pins, indexes and queues,
selections, numbering, decision status and the undrained archive. A refusal is
never turned into acceptance with lost data.

A group proved collectible is collected in that event. Collection is never
deferred to a later event and never refuses an event of its own.

## Restore

Restore rebuilds collection's indexes once from the validated save. It departs
nothing and publishes no archive items on load. The restored session starts
with an empty pending archive. A group that became collectible in an older
save, including one written by rc.15 without collection, departs on the next
accepted event. A refused event leaves the save and pending archive unchanged.
Restore checks that a save is possible for the source. It does not
authenticate history.

## Archive

Each collected record enters the archive as every departed record does
([Departure 0.1](caveat-departure-0.1.md), "The archive"), with its original
withdrawal (evidence, `because`, sequence, event), its reading data, its
qualifications and its removed relations. Every departure, ordinary or
collected, also emits a source-scoped record leaf naming the source, history,
occurrence and departure sequence. A leaf identifies a record. It does not
authenticate the record's content or tell two runs of the same source apart.

`explain` and `dependents`, given an archive, report historical withdrawals in
a separate section:

```text
archive: {
  scope: "provided records and their referenced closure",
  authenticated: false,
  records: [{record, status: "complete" | "unavailable",
             dependencies, unresolved, entry?}],
  unresolved, withdrawals, reasonForWithdrawals?, relations
}
```

`complete` means the supplied record and its referenced closure join
consistently. It does not mean every departed record of the session was
supplied, and it does not authenticate the supplied content. Missing or
conflicting data makes a record, and every record that depends on it,
`unavailable`. Without an archive the live sections are unchanged.

## What changes for a program with windows

- **Snapshots, saves and views.** Collected records leave the retained
  snapshot, `retired`, and saves, and appear as `depart` effects, markers and
  archive entries. These outputs are not byte-identical to a runtime without
  collection, and this profile does not promise that they are.
- **Outcomes.** Executable values, binding values and caveats, true reasons
  and own grounds, current decision status, permission results, and the
  accepted or refused classification with origin and code are the same as
  without collection, with one exception below.
- **Capacity.** Collection can free held records, so a later event can fit
  under the 65,536 held-record limit (`MAX_WINDOWED_RECORDS`) where a runtime
  without collection refuses it with its held-history limit refusal. The limit
  check stays where it is, before the append. Collection does not run early to
  rescue an event already refused there, and it changes no other refusal.

Departure 0.1's invariant reads accordingly: ordinary departure preserves
outcomes; this capacity difference is the one outcome difference collection
adds.

## Cost

Collection is incremental. Indexes map each holder to the records it
references, and the mutation sites that change a reference mark the affected
region. Restore rebuilds the indexes once; no event rebuilds them from the
whole session. Work in one event is proportional to the affected region plus
index and copy costs, and the region can be the whole retained graph when one
root is released. There is no constant-time or bounded-memory promise.
rc.16 measured 55–60 ms for a synchronous release of 3,000 mutual cycles. Hosts
set and test their own budgets.

Windows plus collection do not bound a save. A reachable reason chain is
required and is kept, so it grows with play.

## Conformance

The rules above are checked by these tests at the revision this file merges
at, and by the departure gate that the runtime workflow runs on every change
(`node experiments/departure-gate/run.mjs`: no-window saves and outcomes against
published rc.15, windowed outcomes over 32 seeds, C3, restore of the published
rc.15 C3 save).

| Rule | Tests (`runtime/tests/`) |
| --- | --- |
| Self and mutual groups collect; both directions archived | `withdrawal_collection.rs`: `isolated_self_and_mutual_cycles_depart_with_both_directions_archived` |
| Required chains stay | `reachable_chain_survives_and_unchanged_events_do_not_trace_it` |
| Two roots, release one then the last | `a_mutual_group_waits_for_two_roots_and_archives_atomically_after_retry`, `final_independent_root_release_collects_and_failures_roll_back` |
| Ordinary departure releases a group in the same batch | `ordinary_departing_holder_releases_a_cycle_in_the_same_batch` |
| Pins: scheduled targets, guards, grants, journal | `pending_target_pins_survive_until_the_last_scheduled_application`, `pending_guard_lineage_and_its_transfer_are_conservative_roots`, `permission_grants_and_historical_commitment_bases_keep_exact_reasons`, `journal_retirement_releases_only_its_pin_while_other_owners_retain` |
| Vetoes affect only `C` and clear with their holder | `transferred_skipped_guard_lineage_vetoes_only_additional_collection`, `procedure_arguments_and_skipped_procedure_guards_keep_their_actual_holders`, `stream_and_series_selection_holders_veto_until_successful_overwrite`, `declaration_qualifiers_are_roots_and_successful_predicate_clear_releases` |
| Record-owned metadata does not root | `record_owned_qualification_inside_a_cycle_is_not_an_external_root` |
| Status and citations unchanged | `reopened_commitment_keeps_reason_graph_and_current_status`, `explicit_citations_keep_exact_old_membership_and_refuse_current_alias_substitution`, `later_readable_history_preserves_the_reason_until_the_holder_departs` |
| Transaction rollback | `failed_due_event_keeps_pending_target_and_guard_indexes` |
| Restore collects nothing until an accepted event | `authentic_older_saves_rebuild_without_collecting_until_an_accepted_event`, `a_skipped_uncreated_commitment_is_a_real_owner_on_restore_and_clear` |
| Capacity difference | `withdrawal_capacity.rs`: `capacity_refusal_is_preappend_and_atomic_then_prior_collection_frees_room` |

## Not in this profile

- Series windows. Decision revisions never depart by collection.
- Removing conservative vetoes. Narrowing a veto needs its own argument and
  paired test.
- Any bound on save size, live memory or event time.
- Authentication of saves, archives or evidence.

## Changes

- 0.1 (1.0 cycle): the rc.16 collector contract made specification. The rule
  and its observable differences are unchanged from what rc.16 shipped.
