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

/-! ## Take-back

`unqualify EVIDENCE with CAVEAT` (spec/caveat-take-back-0.1.md). The model has
no graph, so which held occurrences the take-back marks and which current values
lose the caveat (the runtime's live and kept test) arrive as input, as a
qualification's qualifiers do. What is proved is the frozen-record half: the
commitments are untouched, and a current value keeps its number, loses at most
the take-back caveat, and is unchanged unless it holds a marked occurrence. -/

/-- A provenance without one caveat; its evidence is unchanged. -/
def _root_.Caveat.Provenance.without (p : Provenance) (caveat : String) : Provenance :=
  ⟨p.evidence, p.caveats.filter (· ≠ caveat)⟩

theorem _root_.Caveat.Provenance.without_included (p : Provenance) (caveat : String) :
    (p.without caveat).Included p :=
  ⟨fun _ hx => hx, fun _ hx => (List.mem_filter.mp hx).1⟩

theorem _root_.Caveat.Provenance.without_mono {a b : Provenance} (caveat : String) (h : a.Included b) :
    (a.without caveat).Included (b.without caveat) := by
  constructor
  · exact h.1
  · intro x hx
    have hx' := List.mem_filter.mp hx
    exact List.mem_filter.mpr ⟨h.2 hx'.1, hx'.2⟩

/-- `[when GUARD] unqualify EVIDENCE with CAVEAT`. `marked` are the held
occurrences whose live relation the take-back marks; `losing` the current
values for which the caveat is neither live nor kept. The guard joins no
lineage. -/
structure TakeBack where
  caveat : String
  marked : List String
  losing : List String
  guard : Tracked

/-- One current value: one that holds a marked occurrence and is losing drops
the caveat from its lineage and its grounds alike. -/
def takeBackValue (name : String) (value : Tracked) (t : TakeBack) : Tracked :=
  if name ∈ t.losing ∧ t.marked.any (· ∈ value.lineage.evidence) then
    ⟨value.value, value.lineage.without t.caveat, value.grounds.without t.caveat,
      Provenance.without_mono t.caveat value.grounded⟩
  else value

def takeBackValues (values : List (String × Tracked)) (t : TakeBack) :
    List (String × Tracked) :=
  values.map fun (name, value) => (name, takeBackValue name value t)

/-- Take-back 0.1: a skipped rule changes nothing, and nothing to take back is
not a failure. A take-back never refuses an event. -/
def takeBackStep (before : Ledger) (t : TakeBack) : Outcome Ledger :=
  if t.guard.value = 0 then .accepted { before with effects := [] }
  else .accepted { before with values := takeBackValues before.values t, effects := [] }

theorem take_back_preserves_commitments (before after : Ledger) (t : TakeBack)
    (h : takeBackStep before t = .accepted after) :
    after.commitments = before.commitments := by
  unfold takeBackStep at h
  split at h <;> (cases h; rfl)

/-- Each recorded commitment keeps exactly the basis and grounds it was made on. -/
theorem take_back_preserves_basis_and_grounds (before after : Ledger) (t : TakeBack)
    (name : String) (h : takeBackStep before t = .accepted after) :
    (after.commitments.lookup name).map Commitment.basis =
        (before.commitments.lookup name).map Commitment.basis ∧
    (after.commitments.lookup name).map Commitment.grounds =
        (before.commitments.lookup name).map Commitment.grounds := by
  rw [take_back_preserves_commitments before after t h]
  exact ⟨rfl, rfl⟩

/-- Observations, the record of what was seen, do not change. -/
theorem take_back_preserves_observations (before after : Ledger) (t : TakeBack)
    (h : takeBackStep before t = .accepted after) :
    after.observations = before.observations := by
  unfold takeBackStep at h
  split at h <;> (cases h; rfl)

theorem take_back_never_refuses (before : Ledger) (t : TakeBack) :
    ∃ after, takeBackStep before t = .accepted after := by
  unfold takeBackStep
  split
  · exact ⟨_, rfl⟩
  · exact ⟨_, rfl⟩

/-- A value keeps its number and its evidence. -/
theorem take_back_keeps_number (name : String) (value : Tracked) (t : TakeBack) :
    (takeBackValue name value t).value = value.value ∧
    (takeBackValue name value t).lineage.evidence = value.lineage.evidence ∧
    (takeBackValue name value t).grounds.evidence = value.grounds.evidence := by
  unfold takeBackValue
  split <;> exact ⟨rfl, rfl, rfl⟩

/-- A value loses at most the take-back caveat: nothing is added, and every
other caveat stays, in lineage and grounds alike. -/
theorem take_back_loses_at_most_the_caveat (name : String) (value : Tracked) (t : TakeBack) :
    (takeBackValue name value t).lineage.Included value.lineage ∧
    (takeBackValue name value t).grounds.Included value.grounds ∧
    (∀ c ∈ value.lineage.caveats, c ≠ t.caveat →
      c ∈ (takeBackValue name value t).lineage.caveats) ∧
    (∀ c ∈ value.grounds.caveats, c ≠ t.caveat →
      c ∈ (takeBackValue name value t).grounds.caveats) := by
  unfold takeBackValue
  split
  · refine ⟨Provenance.without_included _ _, Provenance.without_included _ _, ?_, ?_⟩
    · intro c hc hne
      exact List.mem_filter.mpr ⟨hc, by simpa using hne⟩
    · intro c hc hne
      exact List.mem_filter.mpr ⟨hc, by simpa using hne⟩
  · exact ⟨Provenance.included_refl _, Provenance.included_refl _,
      fun _ hc _ => hc, fun _ hc _ => hc⟩

/-- Grounds stay within lineage: carried by `Tracked`, restated for the step. -/
theorem take_back_grounds_within_lineage (name : String) (value : Tracked) (t : TakeBack) :
    (takeBackValue name value t).grounds.Included (takeBackValue name value t).lineage :=
  (takeBackValue name value t).grounded

/-- A value that holds no marked occurrence is unchanged. -/
theorem take_back_ignores_unrelated_value (name : String) (value : Tracked) (t : TakeBack)
    (h : ∀ occurrence ∈ t.marked, occurrence ∉ value.lineage.evidence) :
    takeBackValue name value t = value := by
  have hany : ¬ (t.marked.any (· ∈ value.lineage.evidence) = true) := by
    intro hx
    obtain ⟨occurrence, hm, he⟩ := List.any_eq_true.mp hx
    exact h occurrence hm (of_decide_eq_true he)
  simp [takeBackValue, hany]

/-- The round 8 sequence on the witness: doubted, then the doubt taken back.
The value loses it; the decision keeps what it was made on. -/
def doubt : Qualification := ⟨"ea", "doubted", [], Tracked.plain 1⟩

def undoubt : TakeBack := ⟨"doubted", ["ea"], ["a"], Tracked.plain 1⟩

theorem round8_take_back_spares_commitment :
    witnessView (match qualifyStep witness doubt with
      | .accepted after => takeBackStep after undoubt
      | other => other) =
      some (some ["calibration", "ca"], some ["calibration", "ca"],
        some ⟨⟨some 7, ⟨["ea"], ["ca"]⟩⟩, ⟨["ea"], ["ca"]⟩⟩, []) := by decide

end Caveat.Late
