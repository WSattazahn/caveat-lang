import Caveat.Model

namespace Caveat.Late

/-- A commitment's frozen `using` basis; the value is absent when the commit had no `using`. -/
structure Basis where
  value : Option Int
  provenance : Provenance
  deriving Repr, DecidableEq

/-- What a decision was made on: its basis and its grounds. -/
structure Commitment where
  basis : Basis
  grounds : Provenance
  deriving Repr, DecidableEq

/-- Current values, recorded commitments, observed evidence and the last step's
qualify effects as (evidence, caveat). How commitments are created is not modeled;
this module is about what a later qualification does to ones that exist. -/
structure Ledger where
  values : List (String × Tracked)
  commitments : List (String × Commitment)
  observations : List String
  effects : List (String × String)

/-- `[when GUARD] qualify EVIDENCE with CAVEAT`. `qualifiers` are the caveats that
qualify `caveat` in the graph; they arrive with it, as `qualified` inherits them.
`guard` is the evaluated rule guard, `Tracked.plain 1` when there is none. -/
structure Qualification where
  evidence : String
  caveat : String
  qualifiers : List String
  guard : Tracked

def Qualification.added (q : Qualification) : Provenance := ⟨[], q.caveat :: q.qualifiers⟩

/-- One current value. A value that read the evidence gains the caveat (and the
guard) in its lineage; it gains the caveat in its grounds only if its grounds
include the evidence. Grounds stay within lineage by construction. -/
def qualifyValue (value : Tracked) (evidence : String) (added guard : Provenance) : Tracked :=
  if evidence ∈ value.grounds.evidence then
    ⟨value.value, (value.lineage.union added).union guard, value.grounds.union added,
      Provenance.included_trans
        (Provenance.included_union value.grounded (Provenance.included_refl added))
        (Provenance.included_left _ guard)⟩
  else if evidence ∈ value.lineage.evidence then
    ⟨value.value, (value.lineage.union added).union guard, value.grounds,
      Provenance.included_trans value.grounded
        (Provenance.included_trans (Provenance.included_left value.lineage added)
          (Provenance.included_left _ guard))⟩
  else value

def qualifyValues (values : List (String × Tracked)) (q : Qualification) :
    List (String × Tracked) :=
  values.map fun (name, value) => (name, qualifyValue value q.evidence q.added q.guard.lineage)

/-- Late Qualification 0.1: a skipped rule changes nothing; unobserved evidence
refuses the event; otherwise every current value is qualified and nothing else is. -/
def qualifyStep (before : Ledger) (q : Qualification) : Outcome Ledger :=
  if q.guard.value = 0 then .accepted { before with effects := [] }
  else if q.evidence ∈ before.observations then
    .accepted { before with
      values := qualifyValues before.values q
      effects := [(q.evidence, q.caveat)] }
  else .rejected .unobservedEvidence

def runQualifications : Ledger → List Qualification → Outcome Ledger
  | before, [] => .accepted before
  | before, q :: remaining =>
      match qualifyStep before q with
      | .accepted after => runQualifications after remaining
      | .rejected reason => .rejected reason
      | .fatal reason => .fatal reason

theorem late_qualification_preserves_commitments (before after : Ledger) (q : Qualification)
    (h : qualifyStep before q = .accepted after) :
    after.commitments = before.commitments := by
  unfold qualifyStep at h
  split at h
  · cases h; rfl
  · split at h
    · cases h; rfl
    · cases h

/-- Each recorded commitment keeps exactly the basis and grounds it was made on. -/
theorem late_qualification_preserves_basis_and_grounds (before after : Ledger)
    (q : Qualification) (name : String) (h : qualifyStep before q = .accepted after) :
    (after.commitments.lookup name).map Commitment.basis =
        (before.commitments.lookup name).map Commitment.basis ∧
    (after.commitments.lookup name).map Commitment.grounds =
        (before.commitments.lookup name).map Commitment.grounds := by
  rw [late_qualification_preserves_commitments before after q h]
  exact ⟨rfl, rfl⟩

/-- Whatever the outcome, the session that continues has the same commitments. -/
theorem late_qualification_session_preserves_commitments (before resumed : Ledger)
    (q : Qualification) (h : resumeSession before (qualifyStep before q) = some resumed) :
    resumed.commitments = before.commitments := by
  cases hs : qualifyStep before q with
  | accepted after =>
      rw [hs] at h
      cases h
      exact late_qualification_preserves_commitments before resumed q hs
  | rejected reason =>
      rw [hs] at h
      cases h; rfl
  | fatal reason =>
      rw [hs] at h
      cases h

theorem late_qualifications_preserve_commitments (before after : Ledger)
    (qs : List Qualification) (h : runQualifications before qs = .accepted after) :
    after.commitments = before.commitments := by
  induction qs generalizing before with
  | nil =>
      simp [runQualifications] at h
      rw [h]
  | cons q remaining ih =>
      simp only [runQualifications] at h
      split at h
      · rename_i middle hm
        rw [ih middle h, late_qualification_preserves_commitments before middle q hm]
      · cases h
      · cases h

theorem unobserved_late_qualification_rejected (before : Ledger) (q : Qualification)
    (hg : q.guard.value ≠ 0) (ho : q.evidence ∉ before.observations) :
    qualifyStep before q = .rejected .unobservedEvidence := by
  simp [qualifyStep, hg, ho]

theorem unobserved_late_qualification_classification {State : Type} :
    (Outcome.rejected (State := State) .unobservedEvidence).origin = some "evaluation" ∧
    (Outcome.rejected (State := State) .unobservedEvidence).code = some "unobserved_evidence" :=
  ⟨rfl, rfl⟩

theorem skipped_late_qualification_changes_no_value (before : Ledger) (q : Qualification)
    (hg : q.guard.value = 0) :
    qualifyStep before q = .accepted { before with effects := [] } := by
  simp [qualifyStep, hg]

/-- The contrast: a current value that read the evidence does gain the caveat. -/
theorem late_qualification_reaches_current_lineage (value : Tracked) (q : Qualification)
    (h : q.evidence ∈ value.lineage.evidence) :
    (qualifyValue value q.evidence q.added q.guard.lineage).value = value.value ∧
    q.caveat ∈ (qualifyValue value q.evidence q.added q.guard.lineage).lineage.caveats ∧
    q.guard.lineage.Included (qualifyValue value q.evidence q.added q.guard.lineage).lineage := by
  have guard : q.guard.lineage.Included ((value.lineage.union q.added).union q.guard.lineage) :=
    ⟨fun _ hx => List.mem_append_right _ hx, fun _ hx => List.mem_append_right _ hx⟩
  by_cases hg : q.evidence ∈ value.grounds.evidence
  · simp only [qualifyValue, hg, ite_true]
    exact ⟨trivial, by simp [Provenance.union, Qualification.added], guard⟩
  · simp only [qualifyValue, hg, h, ite_true, ite_false]
    exact ⟨trivial, by simp [Provenance.union, Qualification.added], guard⟩

theorem late_qualification_reaches_current_grounds (value : Tracked) (q : Qualification)
    (h : q.evidence ∈ value.grounds.evidence) :
    q.caveat ∈ (qualifyValue value q.evidence q.added q.guard.lineage).grounds.caveats := by
  simp [qualifyValue, h, Provenance.union, Qualification.added]

/-- The guard joins lineage only, as for any effect. -/
theorem late_qualification_guard_not_in_grounds (value : Tracked) (q : Qualification) :
    (qualifyValue value q.evidence q.added q.guard.lineage).grounds = value.grounds ∨
    (qualifyValue value q.evidence q.added q.guard.lineage).grounds =
      value.grounds.union q.added := by
  unfold qualifyValue
  split
  · exact Or.inr rfl
  · split
    · exact Or.inl rfl
    · exact Or.inl rfl

theorem late_qualification_ignores_unrelated_value (value : Tracked) (q : Qualification)
    (h : q.evidence ∉ value.lineage.evidence) :
    qualifyValue value q.evidence q.added q.guard.lineage = value := by
  have hg : q.evidence ∉ value.grounds.evidence := fun hx => h (value.grounded.1 hx)
  simp [qualifyValue, h, hg]

/-- A nonempty witness: the value and the decision both rest on `ea`; only the value learns. -/
def witness : Ledger :=
  ⟨[("a", sample.qualify "ea" ["ca"])],
    [("plan", ⟨⟨some 7, ⟨["ea"], ["ca"]⟩⟩, ⟨["ea"], ["ca"]⟩⟩)], ["ea"], []⟩

def learned : Qualification := ⟨"ea", "late", ["meta"], Tracked.plain 1⟩

def witnessView : Outcome Ledger →
    Option (Option (List String) × Option (List String) × Option Commitment × List (String × String))
  | .accepted after => some ((after.values.lookup "a").map (·.lineage.caveats),
      (after.values.lookup "a").map (·.grounds.caveats), after.commitments.lookup "plan", after.effects)
  | _ => none

theorem nonempty_late_qualification_spares_commitment :
    witnessView (qualifyStep witness learned) =
      some (some ["calibration", "ca", "late", "meta"], some ["calibration", "ca", "late", "meta"],
        some ⟨⟨some 7, ⟨["ea"], ["ca"]⟩⟩, ⟨["ea"], ["ca"]⟩⟩, [("ea", "late")]) := by decide

end Caveat.Late
