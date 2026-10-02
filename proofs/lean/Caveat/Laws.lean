import Caveat.Model

namespace Caveat

theorem union_evidence_membership (a b : Provenance) (x : String) :
    x ∈ (a.union b).evidence ↔ x ∈ a.evidence ∨ x ∈ b.evidence :=
  List.mem_append

theorem union_caveat_membership (a b : Provenance) (x : String) :
    x ∈ (a.union b).caveats ↔ x ∈ a.caveats ∨ x ∈ b.caveats :=
  List.mem_append

theorem union_associative (a b c : Provenance) :
    (a.union b).union c = a.union (b.union c) := by
  cases a; cases b; cases c
  simp [Provenance.union, List.append_assoc]

theorem union_evidence_idempotent (a : Provenance) (x : String) :
    x ∈ (a.union a).evidence ↔ x ∈ a.evidence := by
  simp [Provenance.union]

theorem union_caveat_idempotent (a : Provenance) (x : String) :
    x ∈ (a.union a).caveats ↔ x ∈ a.caveats := by
  simp [Provenance.union]

theorem eager_combination_exact_lineage (n : Int) (a b : Tracked) :
    (Tracked.combine n a b).lineage = a.lineage.union b.lineage := rfl

theorem eager_combination_exact_grounds (n : Int) (a b : Tracked) :
    (Tracked.combine n a b).grounds = a.grounds.union b.grounds := rfl

theorem combine_preserves_grounded (n : Int) (a b : Tracked) :
    (Tracked.combine n a b).grounds.Included (Tracked.combine n a b).lineage :=
  (Tracked.combine n a b).grounded

theorem ignored_argument_retains_lineage (a : Tracked) (n : Int) :
    (a.ignore n).lineage = a.lineage := rfl

theorem ignored_argument_retains_grounds (a : Tracked) (n : Int) :
    (a.ignore n).grounds = a.grounds := rfl

theorem control_retains_grounds (value guard : Tracked) :
    (value.control guard).grounds = value.grounds := rfl

theorem control_adds_lineage (value guard : Tracked) :
    (value.control guard).lineage = value.lineage.union guard.lineage := rfl

theorem true_branch_exact_dependencies (condition : Tracked) (yes no : Unit → Tracked)
    (h : condition.value ≠ 0) :
    condition.select yes no =
      Tracked.combine (yes ()).value condition (yes ()) := by
  simp [Tracked.select, h]

theorem false_branch_exact_dependencies (condition : Tracked) (yes no : Unit → Tracked)
    (h : condition.value = 0) :
    condition.select yes no =
      Tracked.combine (no ()).value condition (no ()) := by
  simp [Tracked.select, h]

theorem successful_write_exact_dependencies (previous guard : Tracked)
    (body : Unit → Tracked) (h : guard.value ≠ 0) :
    let result := previous.guardedWrite guard body
    result.value = (body ()).value ∧
    result.lineage = (body ()).lineage.union guard.lineage ∧
    result.grounds = (body ()).grounds := by
  simp [Tracked.guardedWrite, Tracked.control, h]

theorem skipped_write_value_unchanged (previous guard : Tracked) (body : Unit → Tracked)
    (h : guard.value = 0) :
    (previous.guardedWrite guard body).value = previous.value := by
  simp [Tracked.guardedWrite, Tracked.control, h]

theorem skipped_write_grounds_unchanged (previous guard : Tracked) (body : Unit → Tracked)
    (h : guard.value = 0) :
    (previous.guardedWrite guard body).grounds = previous.grounds := by
  simp [Tracked.guardedWrite, Tracked.control, h]

theorem skipped_write_adds_control_lineage (previous guard : Tracked)
    (body : Unit → Tracked) (h : guard.value = 0) :
    (previous.guardedWrite guard body).lineage = previous.lineage.union guard.lineage := by
  simp [Tracked.guardedWrite, Tracked.control, h]

theorem accepted_citation_subset (value : Tracked) (grounds : Provenance)
    (result : Tracked) (h : cite value grounds = .accepted result) :
    grounds.Included value.lineage := by
  unfold cite at h
  split at h
  · assumption
  · cases h

theorem invalid_citation_rejected (value : Tracked) (grounds : Provenance)
    (h : ¬ grounds.Included value.lineage) :
    cite value grounds = .rejected .ungroundedCitation := by
  simp [cite, h]

theorem citation_does_not_expand_lineage (value : Tracked) (grounds : Provenance)
    (result : Tracked) (h : cite value grounds = .accepted result) :
    result.lineage = value.lineage := by
  unfold cite at h
  split at h
  · cases h; rfl
  · cases h

theorem accepted_citation_uses_exact_grounds (value : Tracked) (grounds : Provenance)
    (result : Tracked) (h : cite value grounds = .accepted result) :
    result.grounds = grounds := by
  unfold cite at h
  split at h
  · cases h; rfl
  · cases h

theorem accepted_citation_preserves_value (value : Tracked) (grounds : Provenance)
    (result : Tracked) (h : cite value grounds = .accepted result) :
    result.value = value.value := by
  unfold cite at h
  split at h
  · cases h; rfl
  · cases h

theorem accepted_session_uses_after {State : Type} (before after : State) :
    resumeSession before (.accepted after) = some after := rfl

theorem rejected_session_uses_before {State : Type} (before : State) (reason : RejectionReason) :
    resumeSession before (.rejected reason) = some before := rfl

theorem fatal_session_unavailable {State : Type} (before : State) (reason : FatalReason) :
    resumeSession before (.fatal reason) = none := rfl

theorem provenance_overflow_is_fatal {State : Type} :
    @provenanceOverflow State = .fatal .provenanceOverflow := rfl

theorem provenance_overflow_code_unclassified {State : Type} :
    (@provenanceOverflow State).code = some "unclassified" := rfl

theorem provenance_overflow_discards_session {State : Type} (before : State) :
    resumeSession before provenanceOverflow = none := rfl

theorem citation_rejection_classification {State : Type} :
    (Outcome.rejected (State := State) .ungroundedCitation).origin = some "evaluation" ∧
    (Outcome.rejected (State := State) .ungroundedCitation).code = some "ungrounded_citation" :=
  ⟨rfl, rfl⟩

theorem nonempty_ignored_argument_retained :
    (sample.ignore 0).value = 0 ∧
    (sample.ignore 0).lineage.evidence = ["reading"] ∧
    (sample.ignore 0).grounds.caveats = ["calibration"] := by decide

theorem nonempty_successful_guard_replaces_previous_dependencies :
    let result := unselected.guardedWrite goGuard (fun _ => sample)
    result.value = 7 ∧
    result.lineage.evidence = ["reading", "gate"] ∧
    result.lineage.caveats = ["calibration", "uncertain"] ∧
    result.grounds.evidence = ["reading"] ∧
    result.grounds.caveats = ["calibration"] ∧
    "unused" ∉ result.lineage.evidence ∧
    "irrelevant" ∉ result.lineage.caveats ∧
    "gate" ∉ result.grounds.evidence ∧
    "uncertain" ∉ result.grounds.caveats := by decide

theorem nonempty_skipped_guard_changes_lineage :
    let result := sample.guardedWrite stopGuard (fun _ => unselected)
    result.value = 7 ∧
    result.grounds.evidence = ["reading"] ∧
    result.grounds.caveats = ["calibration"] ∧
    result.lineage.evidence = ["reading", "gate"] ∧
    result.lineage.caveats = ["calibration", "uncertain"] := by decide

theorem nonempty_lazy_branch_excludes_unselected :
    let result := goGuard.select (fun _ => sample) (fun _ => unselected)
    result.value = 7 ∧
    result.lineage.evidence = ["gate", "reading"] ∧
    result.grounds.caveats = ["uncertain", "calibration"] ∧
    "unused" ∉ result.lineage.evidence := by decide

theorem nonempty_citation_narrows_to_actual_basis :
    citedGrounds (cite (sample.control goGuard) sample.grounds) = some sample.grounds := by
  decide

theorem nonempty_foreign_citation_rejected :
    (cite sample unselected.grounds).code = some "ungrounded_citation" := by decide


theorem nonempty_zero_result_retains_both_inputs :
    let result := Tracked.combine 0 sample unselected
    result.value = 0 ∧
    result.lineage.evidence = ["reading", "unused"] ∧
    result.grounds.caveats = ["calibration", "irrelevant"] := by decide

theorem nonempty_foreign_caveat_rejected :
    (cite sample ⟨["reading"], ["invented"]⟩).code = some "ungrounded_citation" := by decide

end Caveat
