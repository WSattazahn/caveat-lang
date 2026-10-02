import Caveat.Model

namespace Caveat.Bridge

/-- The executable comparison fragment has exactly these five state names. -/
inductive StateName where
  | a | b | g | x | y
  deriving Repr, BEq, DecidableEq

def stateNames : List StateName := [.a, .b, .g, .x, .y]

def StateName.text : StateName → String
  | .a => "a" | .b => "b" | .g => "g" | .x => "x" | .y => "y"

abbrev State := StateName → Tracked

def initial : State := fun _ => Tracked.plain 0

structure Seed where
  a : Int
  b : Int
  g : Int
  deriving Repr

def seeded (seed : Seed) : State
  | .a => (Tracked.plain seed.a).qualify "ea" ["ca"]
  | .b => (Tracked.plain seed.b).qualify "eb" ["cb"]
  | .g => (Tracked.plain seed.g).qualify "eg" ["cg"]
  | .x => Tracked.plain 0
  | .y => Tracked.plain 0

def write (before : State) (target : StateName) (value : Tracked) : State :=
  fun name => if name = target then value else before name

/-- An eager sum or one nonnested conditional between eager sums. -/
inductive Body where
  | sum (names : List StateName)
  | select (condition : StateName) (yes no : List StateName)
  deriving Repr

/-- Conservative numeric growth uses the larger branch, not both branches. -/
def Body.width : Body → Nat
  | .sum names => names.length
  | .select _ yes no => max yes.length no.length

structure Action where
  target : StateName
  body : Body
  guard : Option StateName
  citations : Option (List StateName)
  deriving Repr

/-- Body addition calls the same proved constructor as the core laws. -/
def readBody (state : State) (names : List StateName) : Tracked :=
  names.foldl (fun acc name =>
    let value := state name
    Tracked.combine (acc.value + value.value) acc value) (Tracked.plain 0)

/-- Expression conditions contribute to both channels; the unused branch is a thunk. -/
def evalBody (state : State) : Body → Tracked
  | .sum names => readBody state names
  | .select condition yes no =>
      Tracked.select (state condition)
        (fun _ => readBody state yes) (fun _ => readBody state no)

def readGuard (state : State) : Option StateName → Tracked
  | none => Tracked.plain 1
  | some name => state name

def readCitations (state : State) (names : List StateName) : Provenance :=
  names.foldl (fun acc name => acc.union (state name).grounds) {}

/-- A false guard avoids both the body thunk and every citation read. -/
def runAction (before : State) (action : Action) : Outcome State :=
  let guard := readGuard before action.guard
  let candidate := (before action.target).guardedWrite guard
    (fun _ => evalBody before action.body)
  if guard.value == 0 then .accepted (write before action.target candidate)
  else
    match action.citations with
    | none => .accepted (write before action.target candidate)
    | some names =>
        match cite candidate (readCitations before names) with
        | .accepted value => .accepted (write before action.target value)
        | .rejected reason => .rejected reason
        | .fatal reason => .fatal reason

def runActions : State → List Action → Outcome State
  | before, [] => .accepted before
  | before, action :: remaining =>
      match runAction before action with
      | .accepted after => runActions after remaining
      | .rejected reason => .rejected reason
      | .fatal reason => .fatal reason

/-- The entire step is the transaction boundary, including earlier successful actions. -/
def runStep (before : State) (actions : List Action) : Outcome State × Option State :=
  let outcome := runActions before actions
  (outcome, resumeSession before outcome)

theorem read_written_target (before : State) (target : StateName) (value : Tracked) :
    write before target value target = value := by
  simp [write]

theorem read_other_target_unchanged (before : State) (target name : StateName)
    (value : Tracked) (h : name ≠ target) :
    write before target value name = before name := by
  simp [write, h]

theorem empty_body_plain_zero (before : State) :
    readBody before [] = Tracked.plain 0 := rfl

theorem body_append_exact_combination (before : State) (names : List StateName)
    (name : StateName) :
    readBody before (names ++ [name]) =
      Tracked.combine ((readBody before names).value + (before name).value)
        (readBody before names) (before name) := by
  simp [readBody, List.foldl_append]

theorem sum_body_exact (before : State) (names : List StateName) :
    evalBody before (.sum names) = readBody before names := rfl

theorem true_body_exact_value (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value ≠ 0) :
    (evalBody before (.select condition yes no)).value = (readBody before yes).value := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

theorem false_body_exact_value (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value = 0) :
    (evalBody before (.select condition yes no)).value = (readBody before no).value := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

theorem true_body_exact_lineage (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value ≠ 0) :
    (evalBody before (.select condition yes no)).lineage =
      (before condition).lineage.union (readBody before yes).lineage := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

theorem false_body_exact_lineage (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value = 0) :
    (evalBody before (.select condition yes no)).lineage =
      (before condition).lineage.union (readBody before no).lineage := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

theorem true_body_exact_grounds (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value ≠ 0) :
    (evalBody before (.select condition yes no)).grounds =
      (before condition).grounds.union (readBody before yes).grounds := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

theorem false_body_exact_grounds (before : State) (condition : StateName)
    (yes no : List StateName) (h : (before condition).value = 0) :
    (evalBody before (.select condition yes no)).grounds =
      (before condition).grounds.union (readBody before no).grounds := by
  simp [evalBody, Tracked.select, Tracked.combine, h]

/-- Distinct evidence and caveats make dropping either evaluated dependency visible. -/
theorem nonempty_true_body_excludes_unselected :
    let result := evalBody (seeded ⟨7, 11, 1⟩) (.select .g [.a] [.b])
    result.value = 7 ∧
    result.lineage = ⟨["eg", "ea"], ["cg", "ca"]⟩ ∧
    result.grounds = ⟨["eg", "ea"], ["cg", "ca"]⟩ := by decide

theorem nonempty_false_body_excludes_unselected :
    let result := evalBody (seeded ⟨7, 11, 0⟩) (.select .g [.a] [.b])
    result.value = 11 ∧
    result.lineage = ⟨["eg", "eb"], ["cg", "cb"]⟩ ∧
    result.grounds = ⟨["eg", "eb"], ["cg", "cb"]⟩ := by decide

/-- g is an expression condition; b is only an external guard, even though also untaken. -/
theorem nonempty_expression_condition_distinct_from_statement_guard :
    let before := seeded ⟨7, 11, 1⟩
    let result := (before .x).guardedWrite (before .b)
      (fun _ => evalBody before (.select .g [.a] [.b]))
    result.value = 7 ∧
    result.lineage = ⟨["eg", "ea", "eb"], ["cg", "ca", "cb"]⟩ ∧
    result.grounds = ⟨["eg", "ea"], ["cg", "ca"]⟩ := by decide

theorem skipped_action_exact_state (before : State) (action : Action)
    (h : (readGuard before action.guard).value = 0) :
    runAction before action =
      .accepted (write before action.target
        ((before action.target).control (readGuard before action.guard))) := by
  simp [runAction, Tracked.guardedWrite, h]

theorem accepted_uncited_action_exact_state (before : State) (action : Action)
    (h : (readGuard before action.guard).value ≠ 0)
    (hc : action.citations = none) :
    runAction before action =
      .accepted (write before action.target
        ((evalBody before action.body).control (readGuard before action.guard))) := by
  simp [runAction, Tracked.guardedWrite, h, hc]

theorem accepted_cited_action_exact_state (before : State) (action : Action)
    (names : List StateName) (value : Tracked)
    (h : (readGuard before action.guard).value ≠ 0)
    (hc : action.citations = some names)
    (hv : cite ((evalBody before action.body).control (readGuard before action.guard))
      (readCitations before names) = .accepted value) :
    runAction before action = .accepted (write before action.target value) := by
  simp [runAction, Tracked.guardedWrite, h, hc, hv]

theorem invalid_cited_action_rejected (before : State) (action : Action)
    (names : List StateName)
    (h : (readGuard before action.guard).value ≠ 0)
    (hc : action.citations = some names)
    (invalid : ¬ (readCitations before names).Included
      ((evalBody before action.body).control (readGuard before action.guard)).lineage) :
    runAction before action = .rejected .ungroundedCitation := by
  simp [runAction, Tracked.guardedWrite, h, hc, cite, invalid]

theorem rejected_step_uses_before (before : State) (actions : List Action)
    (reason : RejectionReason) (h : runActions before actions = .rejected reason) :
    (runStep before actions).2 = some before := by
  simp [runStep, h, resumeSession]

theorem accepted_step_uses_after (before after : State) (actions : List Action)
    (h : runActions before actions = .accepted after) :
    (runStep before actions).2 = some after := by
  simp [runStep, h, resumeSession]

theorem accepted_head_uses_updated_state (before after : State) (action : Action)
    (remaining : List Action) (h : runAction before action = .accepted after) :
    runActions before (action :: remaining) = runActions after remaining := by
  simp [runActions, h]

theorem later_rejection_discards_successful_head (before after : State) (action : Action)
    (remaining : List Action) (reason : RejectionReason)
    (ha : runAction before action = .accepted after)
    (hr : runActions after remaining = .rejected reason) :
    (runStep before (action :: remaining)).2 = some before := by
  simp [runStep, runActions, ha, hr, resumeSession]


/-- Only the observations/effects admitted by this fixed neutral-reveal fixture. -/
structure Session where
  values : State
  observations : List String
  revealEffects : List String

def initialSession : Session := ⟨initial, [], []⟩

def seedSession (seed : Seed) : Session :=
  ⟨seeded seed, ["eg", "ea", "eb"], ["eg", "ea", "eb"]⟩

def runSessionStep (before : Session) (actions : List Action) :
    Outcome Session × Option Session :=
  let result := (runStep before.values actions).1
  let outcome := match result with
    | .accepted values => .accepted { before with values, revealEffects := [] }
    | .rejected reason => .rejected reason
    | .fatal reason => .fatal reason
  (outcome, resumeSession before outcome)

theorem rejected_session_step_preserves_all_modeled_fields
    (before : Session) (actions : List Action) (reason : RejectionReason)
    (h : runActions before.values actions = .rejected reason) :
    (runSessionStep before actions).2 = some before := by
  simp [runSessionStep, runStep, h, resumeSession]

theorem accepted_session_step_clears_effects
    (before : Session) (after : State) (actions : List Action)
    (h : runActions before.values actions = .accepted after) :
    (runSessionStep before actions).2 =
      some ⟨after, before.observations, []⟩ := by
  simp [runSessionStep, runStep, h, resumeSession]

theorem seed_observation_order (seed : Seed) :
    (seedSession seed).observations = ["eg", "ea", "eb"] := rfl

theorem seed_effect_order (seed : Seed) :
    (seedSession seed).revealEffects = ["eg", "ea", "eb"] := rfl

end Caveat.Bridge
