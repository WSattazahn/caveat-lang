import Caveat.Late

namespace Caveat.Reopen

/-- A recorded commitment: its frozen basis and grounds, the caveats it retains, and
the evidence that has reopened it, in the order the reopenings happened. It is
reopened, `open` in the runtime's view, when that list is nonempty. -/
structure Commitment where
  basis : Late.Basis
  grounds : Provenance
  retained : List String
  reopenedBy : List String
  deriving Repr, DecidableEq

/-- A decision journal entry, as far as this module needs it. -/
structure Entry where
  commitment : String
  change : String
  because : List String
  caveats : List String
  deriving Repr, DecidableEq

/-- Recorded commitments, observed evidence, the decision journal and the last
step's reopen effects as (commitment, evidence). How commitments are created is
not modeled; this module is about what a later reopening does to them. -/
structure Ledger where
  commitments : List (String × Commitment)
  observations : List String
  journal : List Entry
  effects : List (String × String)
  deriving Repr, DecidableEq

/-- `[when GUARD] reopen COMMITMENT because EVIDENCE`. `caveats` are the evidence's
caveats at that moment, which the graph supplies. `guard` is whether the rule's
guard held. -/
structure Reopening where
  commitment : String
  evidence : String
  caveats : List String
  guard : Bool
  deriving Repr, DecidableEq

def Reopening.entry (r : Reopening) : Entry := ⟨r.commitment, "reopened", [r.evidence], r.caveats⟩

/-- Add a cause to the named commitment and to nothing else. -/
def addCause (commitments : List (String × Commitment)) (r : Reopening) :
    List (String × Commitment) :=
  commitments.map fun (name, c) =>
    if name = r.commitment then (name, { c with reopenedBy := c.reopenedBy ++ [r.evidence] })
    else (name, c)

/-- One reopening. A skipped rule changes nothing. An uncommitted target refuses the
event, then unobserved evidence does. Evidence that already reopened the commitment
adds nothing. Otherwise the cause is recorded on the commitment and in the journal,
and nothing else about any commitment changes. -/
def reopenStep (before : Ledger) (r : Reopening) : Outcome Ledger :=
  if !r.guard then .accepted { before with effects := [] }
  else match before.commitments.lookup r.commitment with
    | none => .rejected .notCommitted
    | some c =>
        if r.evidence ∉ before.observations then .rejected .unobservedEvidence
        else if r.evidence ∈ c.reopenedBy then .accepted { before with effects := [] }
        else .accepted { before with
          commitments := addCause before.commitments r
          journal := before.journal ++ [r.entry]
          effects := [(r.commitment, r.evidence)] }

def runReopenings : Ledger → List Reopening → Outcome Ledger
  | before, [] => .accepted before
  | before, r :: remaining =>
      match reopenStep before r with
      | .accepted after => runReopenings after remaining
      | .rejected reason => .rejected reason
      | .fatal reason => .fatal reason

/-- What a commitment was made on and what it retains, without its reopenings. -/
def Commitment.record (c : Commitment) : Late.Basis × Provenance × List String :=
  (c.basis, c.grounds, c.retained)

def records (commitments : List (String × Commitment)) :
    List (String × (Late.Basis × Provenance × List String)) :=
  commitments.map fun (name, c) => (name, c.record)

theorem add_cause_preserves_records (commitments : List (String × Commitment)) (r : Reopening) :
    records (addCause commitments r) = records commitments := by
  simp only [records, addCause, List.map_map]
  apply List.map_congr_left
  intro entry _
  obtain ⟨name, c⟩ := entry
  by_cases h : name = r.commitment <;> simp [h, Commitment.record]

theorem add_cause_preserves_names (commitments : List (String × Commitment)) (r : Reopening) :
    (addCause commitments r).map Prod.fst = commitments.map Prod.fst := by
  simp only [addCause, List.map_map]
  apply List.map_congr_left
  intro entry _
  obtain ⟨name, c⟩ := entry
  by_cases h : name = r.commitment <;> simp [h]

/-- Every commitment keeps its basis, grounds and retained caveats through an accepted
reopening, the reopened one included. -/
theorem reopen_preserves_records (before after : Ledger) (r : Reopening)
    (h : reopenStep before r = .accepted after) :
    records after.commitments = records before.commitments := by
  unfold reopenStep at h
  split at h
  · cases h; rfl
  · split at h
    · cases h
    · split at h
      · cases h
      · split at h
        · cases h; rfl
        · cases h; exact add_cause_preserves_records _ _

theorem records_lookup (commitments : List (String × Commitment)) (name : String) :
    (records commitments).lookup name = (commitments.lookup name).map Commitment.record := by
  induction commitments with
  | nil => rfl
  | cons head tail ih =>
      obtain ⟨key, c⟩ := head
      by_cases h : name = key
      · subst h; simp [records, List.lookup]
      · have hne : (name == key) = false := by simp [h]
        simp only [records, List.map_cons, List.lookup, hne] at ih ⊢
        exact ih

/-- The reopened commitment, by name, keeps exactly the caveats it retained, and its basis and grounds. -/
theorem reopen_retains_caveats (before after : Ledger) (r : Reopening) (name : String)
    (h : reopenStep before r = .accepted after) :
    (after.commitments.lookup name).map Commitment.retained =
        (before.commitments.lookup name).map Commitment.retained ∧
    (after.commitments.lookup name).map Commitment.basis =
        (before.commitments.lookup name).map Commitment.basis ∧
    (after.commitments.lookup name).map Commitment.grounds =
        (before.commitments.lookup name).map Commitment.grounds := by
  have hr := congrArg (fun l => (List.lookup name l).map
    (fun (x : Late.Basis × Provenance × List String) => (x.2.2, x.1, x.2.1)))
    (reopen_preserves_records before after r h)
  simp only [records_lookup, Option.map_map] at hr
  have h1 := congrArg (Option.map (fun x : List String × Late.Basis × Provenance => x.1)) hr
  have h2 := congrArg (Option.map (fun x : List String × Late.Basis × Provenance => x.2.1)) hr
  have h3 := congrArg (Option.map (fun x : List String × Late.Basis × Provenance => x.2.2)) hr
  simp only [Option.map_map] at h1 h2 h3
  exact ⟨h1, h2, h3⟩

/-- Whatever the outcome, the session that continues keeps every commitment's record. -/
theorem reopen_session_preserves_records (before resumed : Ledger) (r : Reopening)
    (h : resumeSession before (reopenStep before r) = some resumed) :
    records resumed.commitments = records before.commitments := by
  cases hs : reopenStep before r with
  | accepted after =>
      rw [hs] at h
      cases h
      exact reopen_preserves_records before resumed r hs
  | rejected reason =>
      rw [hs] at h
      cases h; rfl
  | fatal reason =>
      rw [hs] at h
      cases h

theorem reopenings_preserve_records (before after : Ledger) (rs : List Reopening)
    (h : runReopenings before rs = .accepted after) :
    records after.commitments = records before.commitments := by
  induction rs generalizing before with
  | nil =>
      simp [runReopenings] at h
      rw [h]
  | cons r remaining ih =>
      simp only [runReopenings] at h
      split at h
      · rename_i middle hm
        rw [ih middle h, reopen_preserves_records before middle r hm]
      · cases h
      · cases h

/-- A new cause is recorded: the journal gains exactly the reopening entry, naming the
evidence and its caveats, and the step reports the reopen. -/
theorem reopen_records_cause (before : Ledger) (r : Reopening) (c : Commitment)
    (hg : r.guard = true) (hc : before.commitments.lookup r.commitment = some c)
    (ho : r.evidence ∈ before.observations) (hn : r.evidence ∉ c.reopenedBy) :
    reopenStep before r = .accepted { before with
      commitments := addCause before.commitments r
      journal := before.journal ++ [⟨r.commitment, "reopened", [r.evidence], r.caveats⟩]
      effects := [(r.commitment, r.evidence)] } := by
  simp [reopenStep, hg, hc, ho, hn, Reopening.entry]

theorem add_cause_lookup (commitments : List (String × Commitment)) (r : Reopening)
    (c : Commitment) (hc : commitments.lookup r.commitment = some c) :
    (addCause commitments r).lookup r.commitment =
      some { c with reopenedBy := c.reopenedBy ++ [r.evidence] } := by
  induction commitments with
  | nil => simp [List.lookup] at hc
  | cons head tail ih =>
      obtain ⟨key, d⟩ := head
      by_cases h : r.commitment = key
      · subst h
        simp [List.lookup] at hc
        subst hc
        simp [addCause]
      · have hne : (r.commitment == key) = false := by simp [h]
        have hne' : ¬key = r.commitment := fun e => h e.symm
        simp only [List.lookup, hne] at hc
        simp only [addCause, List.map_cons, hne', ite_false, List.lookup, hne] at ih ⊢
        exact ih hc

/-- The reopened commitment names its new cause after its earlier ones, so it reads as reopened. -/
theorem reopen_marks_commitment (before : Ledger) (r : Reopening) (c : Commitment)
    (hg : r.guard = true) (hc : before.commitments.lookup r.commitment = some c)
    (ho : r.evidence ∈ before.observations) (hn : r.evidence ∉ c.reopenedBy) :
    ∃ after, reopenStep before r = .accepted after ∧
      (after.commitments.lookup r.commitment).map Commitment.reopenedBy =
        some (c.reopenedBy ++ [r.evidence]) := by
  refine ⟨_, reopen_records_cause before r c hg hc ho hn, ?_⟩
  simp [add_cause_lookup before.commitments r c hc]

/-- An accepted reopening only appends: earlier journal entries and earlier causes stay. -/
theorem reopen_keeps_history (before after : Ledger) (r : Reopening)
    (h : reopenStep before r = .accepted after) :
    ∃ added, after.journal = before.journal ++ added := by
  unfold reopenStep at h
  split at h
  · cases h; exact ⟨[], by simp⟩
  · split at h
    · cases h
    · split at h
      · cases h
      · split at h
        · cases h; exact ⟨[], by simp⟩
        · cases h; exact ⟨_, rfl⟩

theorem repeated_cause_changes_nothing (before : Ledger) (r : Reopening) (c : Commitment)
    (hg : r.guard = true) (hc : before.commitments.lookup r.commitment = some c)
    (ho : r.evidence ∈ before.observations) (hr : r.evidence ∈ c.reopenedBy) :
    reopenStep before r = .accepted { before with effects := [] } := by
  simp [reopenStep, hg, hc, ho, hr]

theorem skipped_reopening_changes_nothing (before : Ledger) (r : Reopening)
    (hg : r.guard = false) :
    reopenStep before r = .accepted { before with effects := [] } := by
  simp [reopenStep, hg]

theorem uncommitted_reopening_rejected (before : Ledger) (r : Reopening)
    (hg : r.guard = true) (hc : before.commitments.lookup r.commitment = none) :
    reopenStep before r = .rejected .notCommitted := by
  simp [reopenStep, hg, hc]

theorem unobserved_reopening_rejected (before : Ledger) (r : Reopening) (c : Commitment)
    (hg : r.guard = true) (hc : before.commitments.lookup r.commitment = some c)
    (ho : r.evidence ∉ before.observations) :
    reopenStep before r = .rejected .unobservedEvidence := by
  simp [reopenStep, hg, hc, ho]

theorem uncommitted_reopening_classification {State : Type} :
    (Outcome.rejected (State := State) .notCommitted).origin = some "evaluation" ∧
    (Outcome.rejected (State := State) .notCommitted).code = some "not_committed" :=
  ⟨rfl, rfl⟩

/-- A nonempty witness: `plan` retains `late` and rests on `ea`; `eb` reopens it. -/
def witness : Ledger :=
  ⟨[("plan", ⟨⟨some 3, ⟨["ea"], ["ca", "late"]⟩⟩, ⟨["ea"], ["ca", "late"]⟩, ["late", "ca"], []⟩),
    ("hold", ⟨⟨none, ⟨[], []⟩⟩, ⟨[], []⟩, [], []⟩)],
    ["ea", "eb"], [⟨"plan", "committed", ["ea"], ["ca", "late"]⟩], []⟩

def doubt : Reopening := ⟨"plan", "eb", ["cb"], true⟩

def witnessView : Outcome Ledger → Option Ledger
  | .accepted after => some after
  | _ => none

theorem nonempty_reopening_retains_and_records :
    witnessView (reopenStep witness doubt) = some
      ⟨[("plan", ⟨⟨some 3, ⟨["ea"], ["ca", "late"]⟩⟩, ⟨["ea"], ["ca", "late"]⟩, ["late", "ca"], ["eb"]⟩),
        ("hold", ⟨⟨none, ⟨[], []⟩⟩, ⟨[], []⟩, [], []⟩)],
        ["ea", "eb"],
        [⟨"plan", "committed", ["ea"], ["ca", "late"]⟩, ⟨"plan", "reopened", ["eb"], ["cb"]⟩],
        [("plan", "eb")]⟩ := by decide

end Caveat.Reopen
