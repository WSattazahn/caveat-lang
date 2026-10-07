# Withdrawal reachability — proposal for review

Status (2026-10-07): design only. The owner requested a proposal, not a runtime
change. No collector, new withdrawal rejection, pin weakening, or revised save
validation is implemented here. The current departure and exact-reason rules
remain authoritative until an amendment is approved. The C3 registered gate
and exact own-ground requirements remain unchanged.

## Measured problem and limits

`withdraw a because a` is valid source. A retired subject pins itself as its
withdrawal's reason. Two subjects can similarly pin one another. Reference
counts alone retain these groups even after no later language operation can
address them from outside the group. The subject must not become an independent
root merely because its withdrawal has a reason or because the reason has a
withdrawal. That would restate the cycle rather than establish reachability.

The accompanying `withdrawal-retention.mjs` observes the existing runtime. It
drains one session after every event, retains another session's entire archive,
and compares outcomes, views, saves and the final ordered archive. The small
`draftReachability` diagnostic covers the included fixtures only; it never
deletes a record and is not a proof of complete language reachability.

Current Windows WASM SHA-256:
`daab2c71023777a3a9e5dc72f7882c653147c4acb445470368cecd76ed30bc59`.
The build records revision `7f551e5774d61ec9f88e0d603e5b01196597b00c`, dirty,
compiled. `withdrawal-retention-results.json` records source/save digests and
checkpoints 10, 30, 60, 300 and 1,000.

| Fixture | Retired pins at 10 / 1,000 cycles | Live save bytes at 10 / 1,000 | Draft classification at 1,000 |
| --- | --- | --- | --- |
| self | 9 / 999 | 2,313 / 199,459 | 999 unrooted withdrawn retired records |
| mutual | 18 / 1,998 | 4,367 / 398,657 | 1,998 unrooted withdrawn retired records |
| fixed-reason acyclic | 0 / 0 | 777 / 803 | already departs; 999 transferred records |
| reachable chain | 19 / 1,999 | 4,399 / 398,695 | no eligible record; every link remains reachable |

All these sessions have zero pending archive items after each drain. Growth
is live retained state, not the handover buffer. The chain starts with an
observed `b`, then alternately renews `a`/`b`, withdrawing each new subject
because of the other current occurrence. The latest `b` reaches the latest
`a`, which reaches the preceding `b`, and so on. Removing isolated cycles
does **not** establish a universal bound based only on window sizes. A
reachable chain may still grow until another applicable runtime limit. These
measurements are save sizes, not resident heap or proposed collector timings.

## Proposed retention rule

For the isolated fixtures, this is a source-specific unreachability argument:
neither program declares a state, reading, decision, journal, scheduled action,
permission, binding or qualification-bearing guard that could retain an older
occurrence. The self fixture's independent roots are the declared `a` and current
`a`; the mutual fixture has declared/current `a` and `b`. Each withdrawal points
only inside its same-cycle group. After renewal, an old group is outside every
window and no root edge enters it. Source selectors can reach only the current
occurrences. The paired own-ground, permission and later-history fixtures add
precisely the independent paths that invalidate that argument; those older
groups must remain. Mere absence from the direct pin census is never enough.

For a transaction's prospective final state, define independent retention
roots `R` and required dependency edges `E`. Compute the least fixed point
`L = R union successors_E(L)`. A group can leave only if every member is
retired, departable under the existing declaration rules, and outside `L`.
An edge from a withdrawal subject to its concrete reason belongs to `E`.
Neither endpoint is put into `R` solely because that edge exists. A reachable
subject retains the true reason and its dependencies transitively, including
any withdrawal of that reason. No reason, status, caveat or own citation is
rewritten to make the group collectible.

This is a conservative proposal: when completeness of a read path is uncertain,
retain its exact dependencies and report the limitation. Optimizing those roots
away requires a separate semantic argument. A graph algorithm or an analogy to
another language's collector is not that argument.

## Required roots and later reads

The implementation review must enumerate these categories and every consumer.
Existing non-withdrawal pin counts alone are insufficient.

| Required retention | Source/consumer to preserve |
| --- | --- |
| All records still within a window; all records in non-windowed histories; current renewable occurrences; source-declared first occurrences | `reactive.rs`: `occurrence`, `withdrawal_target`, `history_read`, `latest`, `retire_oldest`; `caveat-windows-0.1.md` retirement rules |
| State own grounds, grounds of commitments in force, all required journal `because`/`permitted_by`, scheduled targets, every held permission grant | `reactive_departure.rs`: `for_each_pin`; `reactive.rs`: `grounded_citation`, `apply_due_qualifications`, `withdrawal_predicate`, `check_permission` |
| Exact dependencies in independently retained states, live readings, held decision bases/grounds, current selection/reopening/predicate qualifications and scheduled guards where a later operation may consult them | `reactive.rs`: `history_read`, `latest`, `with_current_withdrawals`, `qualify`, `qualify_core`, `predicate_tracked`, `predicate_grounds`, `retain_skipped_effect` |
| The real subject and reason behind observable withdrawal predicates | `withdrawn(E)`, `withdrawn(latest(S))`, `rests_on_withdrawn(D)`, `permission_withdrawn(D)` resolve exact occurrences and carry reasons into subsequent computations/citations |
| Data required by a still-held record's frozen explanation, permission, or pending action | Preserve exact own reasons; compact only where already allowed, with complete matching archive provenance. Do not infer reachability from marker ranges or holder counts. |

Retired records are excluded from `latest`, indexed history, counts and folds
of windowed histories; renewal names resolve to their newest occurrence.
Ordinary source cannot manufacture an `a@2` name through an `id` payload and
turn it into an evidence selector. These facts help demonstrate that the
isolated fixtures' older subjects have no independent later address. They are
not sufficient by themselves: reading a still-live *different* record can
carry that old subject in its frozen provenance.

`later-history-read.cav` demonstrates this distinction. A live reading freezes
`a@2` before it is withdrawn, then many renewals retire `a@2`. Reading that
live reading later calls `with_current_withdrawals` and adds the `withdrawn`
caveat. This subject must stay reachable even with no ordinary own-ground,
journal, scheduled or permission pin. A proposal that only traces those pin
categories would silently change later computation.

For the first implementation, treat exact dependencies of independently
retained computational containers conservatively as roots. For record-owned
metadata, follow its dependency edges only after its owner is reachable;
unreachable records' own metadata must not root the whole group. Frozen
bindings/cues and host snapshots are projections, not arbitrary host pointers
back into executable session records; the host may retain old snapshots
without retaining runtime nodes. Recomputed binding dependencies still need
coverage through their semantic inputs. Do not root stale output caches merely
because they have not yet been recomputed.

## Event, archive, restore and compatibility requirements

1. Form the complete departing set in the transaction; archive each exact
   record and its withdrawal `{evidence,because,sequence,event}` with all
   required relations/provenance. Simultaneous cycle departure must preserve
   both directed reason edges; no dependence on deletion order is allowed.
   Retain existing global history/occurrence ordering and deterministic
   content-addressed provenance. Host resolution of a complete matching archive
   must recover true relationships after copies, merges and replacement.
2. Compute bindings against the prospective state and publish only after all
   effects and bindings succeed. A policy refusal or late evaluation failure
   restores records, pins, graph, statuses, numbers and archive exactly.
   Reachability metadata must participate in the same copy-on-write rollback.
3. Drain frequency must not change state, decisions, own grounds, predicates,
   permission results, dispatch classifications, save bytes, or departure
   scheduling. Neither reachability nor execution consults the host archive.
4. Restore must preserve existing valid older saves, rebuild required roots
   and edges deterministically, depart nothing on load, and start with an empty
   archive. Older cyclic retention is not itself corruption. Any collection
   happens on a later accepted event; failed events cannot migrate the save.
   Review the current `restore_departures` assumption that every held retired
   record must still have a pin: an approved reachability rule needs a compatible
   validation/migration story without weakening unrelated forged-save checks.
5. No-window programs remain byte-compatible. New reachability decisions may
   change departure/archival metadata for windowed programs, but must not
   change later source-observable values, caveats, statuses, genuine reasons,
   successful/refused event outcomes or refusal origin/code. Any runtime-limit
   effects from releasing storage require the same explicit treatment as
   existing departure, not a new rejection to avoid collector work.

## Paired regression obligations

| Included probe | Current observation / requirement for a future implementation |
| --- | --- |
| `self.cav`, `mutual.cav` | Current records remain. Demonstrate older groups have no independent root, then require complete transactional departure once the rule is implemented. |
| `self-own-ground.cav`, `mutual-own-ground.cav` | Two states pin `a@2`; removing one preserves reachability. Policy refusal and division-by-zero after attempting last release preserve it. Only an accepted last release changes draft eligibility. True own grounds must stay exact. |
| `permission-root.cav` | Held grant retains its self-withdrawal and reason. `permission_withdrawn(trust)` remains true; using the withdrawn current grant remains `policy/not_permitted`. |
| `later-history-read.cav` | A retained reading can still consult an old withdrawal; its later result must carry `withdrawn`. This is an independent read path beyond the six pin-count categories. |
| `reachable-chain.cav` | All older links remain reachable from current evidence. Collection must not sever the real reason chain; continued growth remains a reported finding. |
| `acyclic.cav` | Current departure already works. Every drained record retains its exact withdrawal relationship, and drain schedules yield identical ordered archives. |

The probe passed exact current-save restoration at every checkpoint, two
refusal cases for each own-ground fixture, and ordered archive/drain independence
for all eight programs over 1,000 cycles. It also restored 10-cycle self/mutual/
chain saves produced by the published rc.15 WASM and continued identically on
the current build. The rc.15 hash is
`29edda09e93f61381049b5bb614a77d74bf22873ca37fef40da014dbe2fed577`.
These are baseline observations and fixture-level assertions, not a passing
test of an unimplemented collector. Future coverage must add scheduled-target
release, in-force/superseded commitments, journal retirement, skipped guards,
qualification dependencies and complete archive resolution after collected
cycles. Existing targeted tests remain required; none may be weakened.

## Cost and review decisions

Reference counting of all withdrawal edges cannot distinguish an unreachable
cycle. A full trace of all records each event would violate the registered
incremental pin-check requirement. A proposed implementation should maintain
independent-root changes and dependency edges at their existing mutation sites,
and enqueue affected retired regions on retirement, root release, edge change
and restore. Trial reachability/SCC analysis of a candidate region would cost
`O(V_region + E_region)` with equivalent temporary storage; a region can be
the entire retained graph. Index maintenance also costs work proportional to
changed exact references, not merely one counter per event.

Do not claim constant dispatch, bounded candidate work, bounded history or
bounded heap from this sketch. Synchronously releasing one root can expose a
large old component. Deferring collection, rejecting an event to cap work, or
changing when departure occurs each changes a contract and needs review before
implementation. Record examined vertices/edges, maximum queued work, peak
temporary memory, retained live memory and dispatch percentiles at increasing
history sizes, with archive storage reported separately. Rerun registered C3
save/dispatch and outcome/digest sweeps after any implementation; the current
C3 measurement is neither replaced nor generalized by this proposal.

Approval sought in a later review: the conservative independent-root/edge
definition, completeness of its later-read inventory, same-event collection
cost policy, exact archive representation for simultaneous departures, and
older-save compatibility. The broader bound remains unresolved even if all
unreachable cycles are collected. No implementation is authorized by this file.

## Reproduce the current measurements

From the repository root, use an explicit reviewed runtime directory:

```sh
node experiments/departure-gate/withdrawal-retention.mjs --runtime=dist/pkg-reactive --cycles=1000 --baseline-runtime=/path/to/published-rc15/runtime --out=test-results/withdrawal-retention.json
```

The optional baseline path must contain the verified published rc.15 WASM.
The script reports the actual runtime identities and source/save digests.
It does not install packages, build runtime artifacts, or modify runtime source.
