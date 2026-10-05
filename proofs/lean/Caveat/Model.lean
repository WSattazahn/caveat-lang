import Std

namespace Caveat

/-- Lists represent finite sets extensionally; order and duplicates are immaterial to proofs. -/
structure Provenance where
  evidence : List String := []
  caveats : List String := []
  deriving Repr, BEq, DecidableEq

def Provenance.union (a b : Provenance) : Provenance :=
  ⟨a.evidence ++ b.evidence, a.caveats ++ b.caveats⟩

def Provenance.Included (a b : Provenance) : Prop :=
  a.evidence ⊆ b.evidence ∧ a.caveats ⊆ b.caveats

instance (a b : Provenance) : Decidable (a.Included b) :=
  inferInstanceAs (Decidable (a.evidence ⊆ b.evidence ∧ a.caveats ⊆ b.caveats))

theorem Provenance.included_refl (a : Provenance) : a.Included a :=
  ⟨fun _ h => h, fun _ h => h⟩

theorem Provenance.included_trans {a b c : Provenance}
    (hab : a.Included b) (hbc : b.Included c) : a.Included c :=
  ⟨fun _ hx => hbc.1 (hab.1 hx), fun _ hx => hbc.2 (hab.2 hx)⟩

theorem Provenance.included_left (a b : Provenance) : a.Included (a.union b) :=
  ⟨fun _ hx => List.mem_append_left _ hx, fun _ hx => List.mem_append_left _ hx⟩

theorem Provenance.included_union {a b c d : Provenance}
    (hab : a.Included b) (hcd : c.Included d) :
    (a.union c).Included (b.union d) := by
  constructor
  · intro x hx
    rcases List.mem_append.mp hx with h | h
    · exact List.mem_append_left _ (hab.1 h)
    · exact List.mem_append_right _ (hcd.1 h)
  · intro x hx
    rcases List.mem_append.mp hx with h | h
    · exact List.mem_append_left _ (hab.2 h)
    · exact List.mem_append_right _ (hcd.2 h)

/-- The invariant is carried by constructors, not trusted as an axiom. -/
structure Tracked where
  value : Int
  lineage : Provenance
  grounds : Provenance
  grounded : grounds.Included lineage

def Tracked.plain (n : Int) : Tracked :=
  ⟨n, {}, {}, Provenance.included_refl {}⟩

def Tracked.combine (n : Int) (a b : Tracked) : Tracked :=
  ⟨n, a.lineage.union b.lineage, a.grounds.union b.grounds,
    Provenance.included_union a.grounded b.grounded⟩

/-- A control dependency joins lineage only. -/
def Tracked.control (result guard : Tracked) : Tracked :=
  ⟨result.value, result.lineage.union guard.lineage, result.grounds,
    Provenance.included_trans result.grounded
      (Provenance.included_left result.lineage guard.lineage)⟩

def Tracked.qualify (result : Tracked) (name : String) (caveats : List String) : Tracked :=
  Tracked.combine result.value result
    ⟨result.value, ⟨[name], caveats⟩, ⟨[name], caveats⟩,
      Provenance.included_refl ⟨[name], caveats⟩⟩

/-- Eager function arguments retain both provenance channels even if numerically ignored. -/
def Tracked.ignore (arg : Tracked) (value : Int) : Tracked :=
  { arg with value }


/-- A lazy expression joins the condition and selected content in both channels. -/
def Tracked.select (condition : Tracked) (yes no : Unit → Tracked) : Tracked :=
  let selected := if condition.value != 0 then yes () else no ()
  Tracked.combine selected.value condition selected

/-- An external guard contributes only lineage, including when the write is skipped. -/
def Tracked.guardedWrite (previous guard : Tracked) (body : Unit → Tracked) : Tracked :=
  if guard.value != 0 then (body ()).control guard else previous.control guard

inductive RejectionReason where
  | policy
  | ungroundedCitation
  | unobservedEvidence
  | notCommitted
  deriving Repr, DecidableEq

inductive FatalReason where
  | provenanceOverflow
  | unclassified
  deriving Repr, DecidableEq

/-- The first slice describes outcome handling, not an event interpreter. -/
inductive Outcome (State : Type) where
  | accepted (after : State)
  | rejected (reason : RejectionReason)
  | fatal (reason : FatalReason)

def Outcome.code {State : Type} : Outcome State → Option String
  | .accepted _ => none
  | .rejected .policy => some "reject"
  | .rejected .ungroundedCitation => some "ungrounded_citation"
  | .rejected .unobservedEvidence => some "unobserved_evidence"
  | .rejected .notCommitted => some "not_committed"
  | .fatal _ => some "unclassified"

def Outcome.origin {State : Type} : Outcome State → Option String
  | .accepted _ => none
  | .rejected .policy => some "policy"
  | .rejected .ungroundedCitation => some "evaluation"
  | .rejected .unobservedEvidence => some "evaluation"
  | .rejected .notCommitted => some "evaluation"
  | .fatal _ => none

/-- This models the host contract: only a classified rejection retains the prior session. -/
def resumeSession {State : Type} (before : State) : Outcome State → Option State
  | .accepted after => some after
  | .rejected _ => some before
  | .fatal _ => none

/-- Capacity overflow is not a classified rejection in the current dispatch contract. -/
def provenanceOverflow {State : Type} : Outcome State :=
  .fatal .provenanceOverflow

/-- Citations narrow grounds without extending what the value actually read. -/
def cite (value : Tracked) (grounds : Provenance) : Outcome Tracked :=
  if h : grounds.Included value.lineage then
    .accepted ⟨value.value, value.lineage, grounds, h⟩
  else .rejected .ungroundedCitation

def citedGrounds : Outcome Tracked → Option Provenance
  | .accepted value => some value.grounds
  | .rejected _ => none
  | .fatal _ => none

/-- Nonempty witnesses used by the registered regression theorems. -/
def sample : Tracked := (Tracked.plain 7).qualify "reading" ["calibration"]
def stopGuard : Tracked := (Tracked.plain 0).qualify "gate" ["uncertain"]
def goGuard : Tracked := (Tracked.plain 1).qualify "gate" ["uncertain"]
def unselected : Tracked := (Tracked.plain 99).qualify "unused" ["irrelevant"]

end Caveat
