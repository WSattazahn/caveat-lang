import Caveat
import Lean

/- Exhaustive public Caveat theorem inventory, including compiler-generated declarations. -/
#print axioms Caveat.FatalReason.ofNat_ctorIdx
#print axioms Caveat.FatalReason.provenanceOverflow.sizeOf_spec
#print axioms Caveat.FatalReason.unclassified.sizeOf_spec
#print axioms Caveat.Outcome.accepted.inj
#print axioms Caveat.Outcome.accepted.injEq
#print axioms Caveat.Outcome.accepted.sizeOf_spec
#print axioms Caveat.Outcome.fatal.inj
#print axioms Caveat.Outcome.fatal.injEq
#print axioms Caveat.Outcome.fatal.sizeOf_spec
#print axioms Caveat.Outcome.rejected.inj
#print axioms Caveat.Outcome.rejected.injEq
#print axioms Caveat.Outcome.rejected.sizeOf_spec
#print axioms Caveat.Provenance.included_left
#print axioms Caveat.Provenance.included_refl
#print axioms Caveat.Provenance.included_trans
#print axioms Caveat.Provenance.included_union
#print axioms Caveat.Provenance.mk.inj
#print axioms Caveat.Provenance.mk.injEq
#print axioms Caveat.Provenance.mk.sizeOf_spec
#print axioms Caveat.Provenance.union.eq_1
#print axioms Caveat.RejectionReason.ofNat_ctorIdx
#print axioms Caveat.RejectionReason.policy.sizeOf_spec
#print axioms Caveat.RejectionReason.ungroundedCitation.sizeOf_spec
#print axioms Caveat.Tracked.combine._proof_1
#print axioms Caveat.Tracked.control._proof_1
#print axioms Caveat.Tracked.control.eq_1
#print axioms Caveat.Tracked.grounded
#print axioms Caveat.Tracked.guardedWrite.eq_1
#print axioms Caveat.Tracked.mk.congr_simp
#print axioms Caveat.Tracked.mk.inj
#print axioms Caveat.Tracked.mk.injEq
#print axioms Caveat.Tracked.mk.sizeOf_spec
#print axioms Caveat.Tracked.plain._proof_1
#print axioms Caveat.Tracked.qualify._proof_1
#print axioms Caveat.Tracked.select.eq_1
#print axioms Caveat.accepted_citation_preserves_value
#print axioms Caveat.accepted_citation_subset
#print axioms Caveat.accepted_citation_uses_exact_grounds
#print axioms Caveat.accepted_session_uses_after
#print axioms Caveat.citation_does_not_expand_lineage
#print axioms Caveat.citation_rejection_classification
#print axioms Caveat.cite.eq_1
#print axioms Caveat.combine_preserves_grounded
#print axioms Caveat.control_adds_lineage
#print axioms Caveat.control_retains_grounds
#print axioms Caveat.eager_combination_exact_grounds
#print axioms Caveat.eager_combination_exact_lineage
#print axioms Caveat.false_branch_exact_dependencies
#print axioms Caveat.fatal_session_unavailable
#print axioms Caveat.ignored_argument_retains_grounds
#print axioms Caveat.ignored_argument_retains_lineage
#print axioms Caveat.instDecidableEqFatalReason._proof_1
#print axioms Caveat.instDecidableEqFatalReason._proof_2
#print axioms Caveat.instDecidableEqProvenance.decEq._proof_1
#print axioms Caveat.instDecidableEqProvenance.decEq._proof_2
#print axioms Caveat.instDecidableEqProvenance.decEq._proof_3
#print axioms Caveat.instDecidableEqRejectionReason._proof_1
#print axioms Caveat.instDecidableEqRejectionReason._proof_2
#print axioms Caveat.invalid_citation_rejected
#print axioms Caveat.nonempty_citation_narrows_to_actual_basis
#print axioms Caveat.nonempty_foreign_caveat_rejected
#print axioms Caveat.nonempty_foreign_citation_rejected
#print axioms Caveat.nonempty_ignored_argument_retained
#print axioms Caveat.nonempty_lazy_branch_excludes_unselected
#print axioms Caveat.nonempty_skipped_guard_changes_lineage
#print axioms Caveat.nonempty_successful_guard_replaces_previous_dependencies
#print axioms Caveat.nonempty_zero_result_retains_both_inputs
#print axioms Caveat.provenance_overflow_code_unclassified
#print axioms Caveat.provenance_overflow_discards_session
#print axioms Caveat.provenance_overflow_is_fatal
#print axioms Caveat.rejected_session_uses_before
#print axioms Caveat.skipped_write_adds_control_lineage
#print axioms Caveat.skipped_write_grounds_unchanged
#print axioms Caveat.skipped_write_value_unchanged
#print axioms Caveat.successful_write_exact_dependencies
#print axioms Caveat.true_branch_exact_dependencies
#print axioms Caveat.union_associative
#print axioms Caveat.union_caveat_idempotent
#print axioms Caveat.union_caveat_membership
#print axioms Caveat.union_evidence_idempotent
#print axioms Caveat.union_evidence_membership

/- Enumerate and audit actual elaborated declarations, not source-text matches. -/
run_cmd do
  let env ← Lean.getEnv
  let declarations := env.constants.toList.filter fun (name, _) =>
    name.toString.startsWith "Caveat."
  let names := declarations.filterMap fun (name, info) =>
    if info.isTheorem then some name.toString else none
  for name in names.mergeSort do
    Lean.logInfo m!"CAVEAT_THEOREM {name}"
  for (name, info) in declarations do
    if info.isAxiom then
      throwError "CAVEAT forbids custom axiom {name}"
    let axioms ← Lean.collectAxioms name
    for axiomName in axioms do
      unless ["propext", "Classical.choice", "Quot.sound"].contains axiomName.toString do
        throwError "CAVEAT declaration {name} depends on forbidden axiom {axiomName}"
  Lean.logInfo m!"CAVEAT_DECLARATIONS_CHECKED {declarations.length}"
