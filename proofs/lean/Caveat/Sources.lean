import Caveat.Model

namespace Caveat.Sources

/-! # Caveat Sources 0.1

spec/caveat-caveat-sources-0.1.md, "The invariant": for every current value,
its grounds and every stored observation or predicate qualification, a caveat
is carried exactly when it has a kept route, or a route through an occurrence
whose relation is live.

This module keeps every route explicitly and updates them event by event, the
way an implementation that pushes caveats to what holds them would. The
theorems prove that those updates never drift from the invariant: after any
sequence of events, refused ones included, each carrier's routes are exactly
its kept routes plus one route through each listed occurrence for each of that
occurrence's live relations. D012 to D020 were each a carrier that drifted. -/

/-- An occurrence of evidence: the evidence's name and which occurrence. -/
abbrev Occ := String × Nat

/-- How a caveat arrived: from a record (`kept`) or through an occurrence's relation. -/
inductive Route where
  | kept
  | through (occurrence : Occ)
  deriving Repr, DecidableEq

/-- What a value, its grounds or a stored qualification rests on, with the route
of each caveat it carries. A caveat may arrive by several routes. -/
structure Carrier where
  evidence : List Occ := []
  routes : List (String × Route) := []
  deriving Repr, DecidableEq

def Carrier.join (a b : Carrier) : Carrier := ⟨a.evidence ++ b.evidence, a.routes ++ b.routes⟩

/-- The caveat names a carrier shows. -/
def Carrier.caveats (k : Carrier) : List String := k.routes.map Prod.fst

/-- `C qualifies O`, declared or added by `qualify`; a take-back marks it. -/
structure Relation where
  occurrence : Occ
  caveat : String
  marked : Bool
  deriving Repr, DecidableEq

/-- What holds a carrier: a state's lineage, its grounds, the stored observation
an occurrence's guard read, or a stored predicate qualification. -/
inductive Cell where
  | lineage (state : String)
  | grounds (state : String)
  | observation (occurrence : Occ)
  | predicate (key : String)
  deriving Repr, DecidableEq

/-- A decision's frozen basis: the evidence it lists and the caveats it was made with. -/
structure Basis where
  evidence : List Occ
  kept : List String
  deriving Repr, DecidableEq

structure Ledger where
  cells : List (Cell × Carrier) := []
  relations : List Relation := []
  held : List Occ := []
  current : List (String × Occ) := []
  bases : List (String × Basis) := []
  next : Nat := 0
  deriving Repr, DecidableEq

/-- A relation is live while its occurrence is held and it is not taken back. -/
def Live (s : Ledger) (o : Occ) (c : String) : Prop :=
  o ∈ s.held ∧ ⟨o, c, false⟩ ∈ s.relations

instance (s : Ledger) (o : Occ) (c : String) : Decidable (Live s o c) :=
  inferInstanceAs (Decidable (_ ∧ _))

/-- The spec's invariant for one carrier, as routes: a route through `o` is
present exactly when `o` is listed and its relation is live. Kept routes are free. -/
def Matches (s : Ledger) (k : Carrier) : Prop :=
  ∀ c o, (c, Route.through o) ∈ k.routes ↔ o ∈ k.evidence ∧ Live s o c

def Consistent (s : Ledger) : Prop := ∀ p ∈ s.cells, Matches s p.2

/-! ## Reading -/

/-- The live routes through each listed occurrence. -/
def liveRoutes (s : Ledger) (evidence : List Occ) : List (String × Route) :=
  evidence.flatMap fun o =>
    if o ∈ s.held then
      s.relations.filterMap fun r =>
        if r.occurrence = o ∧ r.marked = false then some (r.caveat, Route.through o) else none
    else []

def cell (s : Ledger) (k : Cell) : Carrier := (s.cells.lookup k).getD {}

/-- Where a rule reads from: a state, evidence (its current occurrence and the
observation its guard stored), a decision's basis, or a stored predicate. -/
inductive Source where
  | state (name : String)
  | evidence (name : String)
  | basis (decision : String)
  | predicate (key : String)
  deriving Repr, DecidableEq

def occurrenceCarrier (s : Ledger) (o : Occ) : Carrier :=
  (Carrier.mk [o] (liveRoutes s [o])).join (cell s (.observation o))

/-- A record's own caveats are kept; evidence a record lists is read live. -/
def basisCarrier (s : Ledger) (b : Basis) : Carrier :=
  ⟨b.evidence, b.kept.map (·, Route.kept) ++ liveRoutes s b.evidence⟩

def Source.read (s : Ledger) (grounds : Bool) : Source → Carrier
  | .state name => cell s (if grounds then .grounds name else .lineage name)
  | .evidence name => match s.current.lookup name with
      | some o => occurrenceCarrier s o
      | none => {}
  | .basis d => match s.bases.lookup d with
      | some b => basisCarrier s b
      | none => {}
  | .predicate key => cell s (.predicate key)

def read (s : Ledger) (grounds : Bool) (sources : List Source) : Carrier :=
  sources.foldr (fun source acc => (source.read s grounds).join acc) {}

/-! ## Events -/

/-- The events of the model. A rule whose guard is false is no event. Guards are
given as the sources they read. -/
inductive Event where
  /-- `[when GUARD] set STATE = BODY`: the guard joins the lineage only. -/
  | set (state : String) (body guard : List Source)
  /-- `[when GUARD] reveal`/`renew EVIDENCE`: a new occurrence with its declared
  relations; its guard's read is stored as the occurrence's observation. -/
  | observe (evidence : String) (declared : List String) (guard : List Source)
  /-- A guard of `commit`, `reopen`, `withdraw` or `examine` stores what it read. -/
  | predicate (key : String) (guard : List Source)
  /-- `[when GUARD] qualify EVIDENCE with CAVEAT`. -/
  | qualify (evidence caveat : String) (guard : List Source)
  /-- `unqualify EVIDENCE with CAVEAT`. -/
  | unqualify (evidence caveat : String)
  /-- `commit DECISION … using BASIS`: the basis is frozen as a record. -/
  | commit (decision : String) (basis : List Source)
  /-- An occurrence leaves its window. -/
  | depart (occurrence : Occ)
  deriving Repr, DecidableEq

def mapCells (f : Cell → Carrier → Carrier) (cells : List (Cell × Carrier)) :
    List (Cell × Carrier) :=
  cells.map fun p => (p.1, f p.1 p.2)

/-- Store a carrier in a cell, replacing what it held. -/
def store (cells : List (Cell × Carrier)) (k : Cell) (v : Carrier) : List (Cell × Carrier) :=
  (k, v) :: cells.filter (·.1 ≠ k)

/-- An occurrence just observed: each carrier that lists it gains its live routes. -/
def addLive (s : Ledger) (o : Occ) (k : Carrier) : Carrier :=
  if o ∈ k.evidence then { k with routes := k.routes ++ liveRoutes s [o] } else k

/-- A `qualify`: a carrier that lists the occurrence gains the route through it. -/
def addRoute (o : Occ) (c : String) (k : Carrier) : Carrier :=
  if o ∈ k.evidence then { k with routes := (c, Route.through o) :: k.routes } else k

/-- Whether a take-back of `evidence` with `caveat` removes this route. -/
def takenBack (held : List Occ) (evidence caveat : String) : String × Route → Bool
  | (c, .through o) => decide (c = caveat ∧ o.1 = evidence ∧ o ∈ held)
  | (_, .kept) => false

def takeBack (held : List Occ) (evidence caveat : String) (k : Carrier) : Carrier :=
  { k with routes := k.routes.filter fun r => !takenBack held evidence caveat r }

def markRelation (held : List Occ) (evidence caveat : String) (r : Relation) : Relation :=
  if r.occurrence.1 = evidence ∧ r.caveat = caveat ∧ r.occurrence ∈ held then
    { r with marked := true }
  else r

/-- A departed occurrence's routes become kept: the carrier keeps what its live
relations gave it (D018). -/
def keepRoute (o : Occ) : String × Route → String × Route
  | (c, .through p) => if p = o then (c, .kept) else (c, .through p)
  | (c, .kept) => (c, .kept)

def depart (o : Occ) (k : Carrier) : Carrier := { k with routes := k.routes.map (keepRoute o) }

def observeStep (s : Ledger) (e : String) (declared : List String) (guard : List Source) : Ledger :=
  let o : Occ := (e, s.next)
  let stored : Ledger := { s with cells := store s.cells (.observation o) (read s false guard) }
  let after : Ledger := { stored with
    held := o :: s.held
    relations := declared.map (fun c => ⟨o, c, false⟩) ++ s.relations
    current := (e, o) :: s.current
    next := s.next + 1 }
  { after with cells := mapCells (fun _ k => addLive after o k) after.cells }

def qualifyCells (s : Ledger) (o : Occ) (c : String) (guard : Carrier) :
    List (Cell × Carrier) :=
  mapCells (fun name k =>
    match name with
    | .lineage _ => if o ∈ k.evidence then (addRoute o c k).join (addRoute o c guard) else k
    | _ => addRoute o c k) s.cells

def qualifyLedger (s : Ledger) (o : Occ) (c : String) (guard : Carrier) : Ledger :=
  { s with relations := ⟨o, c, false⟩ :: s.relations, cells := qualifyCells s o c guard }

def unqualifyLedger (s : Ledger) (e c : String) : Ledger :=
  { s with
    relations := s.relations.map (markRelation s.held e c)
    cells := mapCells (fun _ k => takeBack s.held e c k) s.cells }

def departLedger (s : Ledger) (o : Occ) : Ledger :=
  { s with held := s.held.filter (· ≠ o), cells := mapCells (fun _ k => depart o k) s.cells }

/-- One event. Only a `qualify` of evidence with no held current occurrence is
refused (`unobserved_evidence`); a refused event changes nothing. -/
def step (s : Ledger) : Event → Outcome Ledger
  | .set x body guard =>
      let lineage := (read s false body).join (read s false guard)
      let grounds := read s true body
      .accepted { s with cells := store (store s.cells (.grounds x) grounds) (.lineage x) lineage }
  | .observe e declared guard => .accepted (observeStep s e declared guard)
  | .predicate key guard =>
      .accepted { s with cells := store s.cells (.predicate key) (read s false guard) }
  | .qualify e c guard =>
      match s.current.lookup e with
      | some o =>
          if o ∈ s.held then
            .accepted (qualifyLedger s o c (read s false guard))
          else .rejected .unobservedEvidence
      | none => .rejected .unobservedEvidence
  | .unqualify e c => .accepted (unqualifyLedger s e c)
  | .commit d body =>
      let b := read s false body
      .accepted { s with bases := (d, ⟨b.evidence, b.caveats⟩) :: s.bases }
  | .depart o => .accepted (departLedger s o)

/-- The host contract: a refused event leaves the session as it was. -/
def run (s : Ledger) : List Event → Ledger
  | [] => s
  | event :: rest => run ((resumeSession s (step s event)).getD s) rest

/-! ## Reading preserves the invariant -/

theorem mem_liveRoutes (s : Ledger) (evidence : List Occ) (c : String) (r : Route) :
    (c, r) ∈ liveRoutes s evidence ↔ ∃ o, r = Route.through o ∧ o ∈ evidence ∧ Live s o c := by
  simp only [liveRoutes, List.mem_flatMap, Live]
  constructor
  · rintro ⟨o, ho, hm⟩
    split at hm
    · rename_i hheld
      obtain ⟨rel, hrel, hsome⟩ := List.mem_filterMap.mp hm
      split at hsome
      · rename_i hcond
        cases hsome
        obtain ⟨hocc, hmarked⟩ := hcond
        refine ⟨o, rfl, ho, hheld, ?_⟩
        have : rel = ⟨o, rel.caveat, false⟩ := by
          cases rel; simp_all
        rw [this] at hrel; exact hrel
      · cases hsome
    · simp at hm
  · rintro ⟨o, rfl, ho, hheld, hrel⟩
    refine ⟨o, ho, ?_⟩
    simp only [hheld, ↓reduceIte]
    exact List.mem_filterMap.mpr ⟨⟨o, c, false⟩, hrel, by simp⟩

theorem liveRoutes_matches (s : Ledger) (evidence : List Occ) :
    Matches s ⟨evidence, liveRoutes s evidence⟩ := by
  intro c o
  rw [mem_liveRoutes]
  constructor
  · rintro ⟨p, hp, he, hl⟩
    cases hp; exact ⟨he, hl⟩
  · rintro ⟨he, hl⟩
    exact ⟨o, rfl, he, hl⟩

theorem join_matches {s : Ledger} {a b : Carrier} (ha : Matches s a) (hb : Matches s b) :
    Matches s (a.join b) := by
  intro c o
  simp only [Carrier.join, List.mem_append]
  rw [ha c o, hb c o]
  constructor
  · rintro (⟨h1, h2⟩ | ⟨h1, h2⟩)
    · exact ⟨Or.inl h1, h2⟩
    · exact ⟨Or.inr h1, h2⟩
  · rintro ⟨h1 | h1, h2⟩
    · exact Or.inl ⟨h1, h2⟩
    · exact Or.inr ⟨h1, h2⟩

theorem empty_matches (s : Ledger) : Matches s {} := by
  intro c o
  simp

theorem mem_of_lookup {k : Cell} {v : Carrier} :
    ∀ {cells : List (Cell × Carrier)}, cells.lookup k = some v → (k, v) ∈ cells
  | [], h => by simp at h
  | (k', v') :: rest, h => by
      simp only [List.lookup] at h
      split at h
      · rename_i heq
        cases h
        have : k = k' := by simpa using heq
        subst this
        exact List.mem_cons_self
      · exact List.mem_cons_of_mem _ (mem_of_lookup h)

theorem cell_matches {s : Ledger} (h : Consistent s) (k : Cell) : Matches s (cell s k) := by
  unfold cell
  cases hl : s.cells.lookup k with
  | none => exact empty_matches s
  | some v =>
      have : (k, v) ∈ s.cells := mem_of_lookup hl
      exact h _ this

theorem basis_matches (s : Ledger) (b : Basis) : Matches s (basisCarrier s b) := by
  intro c o
  simp only [basisCarrier, List.mem_append, List.mem_map]
  constructor
  · rintro (⟨_, _, hx⟩ | h)
    · cases hx
    · exact (liveRoutes_matches s b.evidence c o).mp h
  · intro h
    exact Or.inr ((liveRoutes_matches s b.evidence c o).mpr h)

theorem source_matches {s : Ledger} (h : Consistent s) (grounds : Bool) (source : Source) :
    Matches s (source.read s grounds) := by
  cases source with
  | state name => exact cell_matches h _
  | evidence name =>
      simp only [Source.read]
      split
      · exact join_matches (liveRoutes_matches s _) (cell_matches h _)
      · exact empty_matches s
  | basis d =>
      simp only [Source.read]
      split
      · exact basis_matches s _
      · exact empty_matches s
  | predicate key => exact cell_matches h _

theorem read_matches {s : Ledger} (h : Consistent s) (grounds : Bool) (sources : List Source) :
    Matches s (read s grounds sources) := by
  induction sources with
  | nil => exact empty_matches s
  | cons source rest ih =>
      exact join_matches (source_matches h grounds source) ih

theorem store_consistent {s : Ledger} {cells : List (Cell × Carrier)} (k : Cell) (v : Carrier)
    (h : ∀ p ∈ cells, Matches s p.2) (hv : Matches s v) :
    ∀ p ∈ store cells k v, Matches s p.2 := by
  intro p hp
  simp only [store, List.mem_cons, List.mem_filter] at hp
  rcases hp with rfl | ⟨hp, _⟩
  · exact hv
  · exact h p hp

theorem mapCells_consistent {s : Ledger} {cells : List (Cell × Carrier)}
    {f : Cell → Carrier → Carrier} (h : ∀ p ∈ cells, Matches s (f p.1 p.2)) :
    ∀ p ∈ mapCells f cells, Matches s p.2 := by
  intro p hp
  simp only [mapCells, List.mem_map] at hp
  obtain ⟨q, hq, rfl⟩ := hp
  exact h q hq

/-! ## Each event preserves the invariant -/

theorem set_preserves (s : Ledger) (h : Consistent s) (x : String) (body guard : List Source) :
    ∀ after, step s (.set x body guard) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  exact store_consistent _ _ (store_consistent _ _ h (read_matches h true body))
    (join_matches (read_matches h false body) (read_matches h false guard))

theorem predicate_preserves (s : Ledger) (h : Consistent s) (key : String) (guard : List Source) :
    ∀ after, step s (.predicate key guard) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  exact store_consistent _ _ h (read_matches h false guard)

theorem commit_preserves (s : Ledger) (h : Consistent s) (d : String) (body : List Source) :
    ∀ after, step s (.commit d body) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  exact h

theorem observe_preserves (s : Ledger) (h : Consistent s) (e : String) (declared : List String)
    (guard : List Source) :
    ∀ after, step s (.observe e declared guard) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  let o : Occ := (e, s.next)
  let stored : Ledger := { s with cells := store s.cells (.observation o) (read s false guard) }
  have hstored : Consistent stored :=
    store_consistent _ _ h (read_matches h false guard)
  let mid : Ledger := { stored with
    held := o :: s.held
    relations := declared.map (fun c => ⟨o, c, false⟩) ++ s.relations
    current := (e, o) :: s.current
    next := s.next + 1 }
  -- Only `o` gains live relations; every other occurrence is as before.
  have hother : ∀ p c, p ≠ o → (Live mid p c ↔ Live stored p c) := by
    intro p c hp
    simp only [Live, mid, stored, List.mem_cons, List.mem_append, List.mem_map]
    constructor
    · rintro ⟨hh | hh, hr | hr⟩
      · exact absurd hh hp
      · exact absurd hh hp
      · obtain ⟨_, _, hx⟩ := hr; cases hx; exact absurd rfl hp
      · exact ⟨hh, hr⟩
    · rintro ⟨hh, hr⟩
      exact ⟨Or.inr hh, Or.inr hr⟩
  have hgrow : ∀ p c, Live stored p c → Live mid p c := by
    rintro p c ⟨hh, hr⟩
    exact ⟨List.mem_cons_of_mem _ hh, List.mem_append_right _ hr⟩
  show ∀ p ∈ mapCells (fun _ k => addLive mid o k) mid.cells, Matches mid p.2
  apply mapCells_consistent
  intro p hp
  have hk : Matches stored p.2 := hstored p hp
  intro c q
  unfold addLive
  split
  · rename_i ho
    simp only [List.mem_append]
    rw [mem_liveRoutes]
    by_cases hq : q = o
    · subst hq
      constructor
      · rintro (hr | ⟨r, hr, hm, hl⟩)
        · obtain ⟨he, hl⟩ := (hk c _).mp hr
          exact ⟨he, hgrow _ _ hl⟩
        · cases hr; exact ⟨ho, hl⟩
      · rintro ⟨_, hl⟩
        exact Or.inr ⟨_, rfl, List.mem_singleton_self _, hl⟩
    · constructor
      · rintro (hr | ⟨r, hr, hm, hl⟩)
        · obtain ⟨he, hl⟩ := (hk c q).mp hr
          exact ⟨he, (hother q c hq).mpr hl⟩
        · cases hr; exact absurd (List.mem_singleton.mp hm) hq
      · rintro ⟨he, hl⟩
        exact Or.inl ((hk c q).mpr ⟨he, (hother q c hq).mp hl⟩)
  · rename_i ho
    by_cases hq : q = o
    · subst hq
      constructor
      · intro hr
        exact absurd ((hk c _).mp hr).1 ho
      · rintro ⟨he, _⟩
        exact absurd he ho
    · rw [hk c q, hother q c hq]

theorem addRoute_matches {s t : Ledger} {o : Occ} {c : String} (hheld : o ∈ s.held)
    (hlive : ∀ p d, Live t p d ↔ Live s p d ∨ (p = o ∧ d = c)) {k : Carrier}
    (hk : Matches s k) : Matches t (addRoute o c k) := by
  intro d p
  unfold addRoute
  split
  · rename_i ho
    simp only [List.mem_cons, Prod.mk.injEq, Route.through.injEq]
    rw [hk d p, hlive p d]
    constructor
    · rintro (⟨rfl, rfl⟩ | ⟨he, hl⟩)
      · exact ⟨ho, Or.inr ⟨rfl, rfl⟩⟩
      · exact ⟨he, Or.inl hl⟩
    · rintro ⟨he, hl | ⟨rfl, rfl⟩⟩
      · exact Or.inr ⟨he, hl⟩
      · exact Or.inl ⟨rfl, rfl⟩
  · rename_i ho
    rw [hk d p, hlive p d]
    constructor
    · rintro ⟨he, hl⟩; exact ⟨he, Or.inl hl⟩
    · rintro ⟨he, hl | ⟨rfl, _⟩⟩
      · exact ⟨he, hl⟩
      · exact absurd he ho

theorem qualify_preserves (s : Ledger) (h : Consistent s) (e c : String) (guard : List Source) :
    ∀ after, step s (.qualify e c guard) = .accepted after → Consistent after := by
  intro after hs
  simp only [step] at hs
  split at hs
  · rename_i o _
    split at hs
    · rename_i hheld
      simp only [Outcome.accepted.injEq] at hs
      subst hs
      have hlive : ∀ p d, Live (qualifyLedger s o c (read s false guard)) p d ↔
          Live s p d ∨ (p = o ∧ d = c) := by
        intro p d
        simp only [Live, qualifyLedger, List.mem_cons, Relation.mk.injEq, and_true]
        constructor
        · rintro ⟨hh, ⟨rfl, rfl⟩ | hr⟩
          · exact Or.inr ⟨rfl, rfl⟩
          · exact Or.inl ⟨hh, hr⟩
        · rintro (⟨hh, hr⟩ | ⟨rfl, rfl⟩)
          · exact ⟨hh, Or.inr hr⟩
          · exact ⟨hheld, Or.inl ⟨rfl, rfl⟩⟩
      have hg := read_matches h false guard
      intro p' hp'
      dsimp only [qualifyLedger, qualifyCells] at hp'
      refine mapCells_consistent ?_ p' hp'
      intro p hp
      have hk := h p hp
      match p.1 with
      | .lineage _ =>
          show Matches _ (if o ∈ p.2.evidence then _ else _)
          split
          · exact join_matches (addRoute_matches hheld hlive hk) (addRoute_matches hheld hlive hg)
          · rename_i ho
            have := addRoute_matches hheld hlive hk
            unfold addRoute at this
            simp only [ho, ↓reduceIte] at this
            exact this
      | .grounds _ => exact addRoute_matches hheld hlive hk
      | .observation _ => exact addRoute_matches hheld hlive hk
      | .predicate _ => exact addRoute_matches hheld hlive hk
    · cases hs
  · cases hs

theorem unqualify_preserves (s : Ledger) (h : Consistent s) (e c : String) :
    ∀ after, step s (.unqualify e c) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  have hlive : ∀ p d, Live (unqualifyLedger s e c) p d ↔
      Live s p d ∧ ¬ (d = c ∧ p.1 = e) := by
    intro p d
    simp only [Live, unqualifyLedger, List.mem_map]
    constructor
    · rintro ⟨hh, r, hr, hm⟩
      unfold markRelation at hm
      split at hm
      · cases hm
      · rename_i hcond
        subst hm
        refine ⟨⟨hh, hr⟩, ?_⟩
        rintro ⟨rfl, rfl⟩
        exact hcond ⟨rfl, rfl, hh⟩
    · rintro ⟨⟨hh, hr⟩, hn⟩
      refine ⟨hh, ⟨p, d, false⟩, hr, ?_⟩
      unfold markRelation
      split
      · rename_i hc
        exact absurd ⟨hc.2.1, hc.1⟩ hn
      · rfl
  intro p' hp'
  dsimp only [unqualifyLedger] at hp'
  refine mapCells_consistent ?_ p' hp'
  intro p hp
  have hk := h p hp
  intro d q
  simp only [takeBack, List.mem_filter, takenBack]
  rw [hk d q, hlive q d]
  constructor
  · rintro ⟨⟨he, hl⟩, hn⟩
    refine ⟨he, hl, ?_⟩
    rintro ⟨rfl, rfl⟩
    simp [hl.1] at hn
  · rintro ⟨he, hl, hn⟩
    refine ⟨⟨he, hl⟩, ?_⟩
    simp only [Bool.not_eq_eq_eq_not, Bool.not_true, decide_eq_false_iff_not]
    rintro ⟨h1, h2, _⟩
    exact hn ⟨h1, h2⟩

theorem depart_preserves (s : Ledger) (h : Consistent s) (o : Occ) :
    ∀ after, step s (.depart o) = .accepted after → Consistent after := by
  intro after hs
  simp only [step, Outcome.accepted.injEq] at hs
  subst hs
  intro p' hp'
  dsimp only [departLedger] at hp'
  refine mapCells_consistent ?_ p' hp'
  intro p hp
  have hk := h p hp
  intro d q
  simp only [depart, departLedger, List.mem_map, Live, List.mem_filter]
  constructor
  · rintro ⟨⟨d', r⟩, hr, hx⟩
    cases r with
    | kept => simp [keepRoute] at hx
    | through p' =>
        simp only [keepRoute] at hx
        split at hx
        · cases hx
        · rename_i hne
          simp only [Prod.mk.injEq, Route.through.injEq] at hx
          obtain ⟨rfl, rfl⟩ := hx
          obtain ⟨he, hh, hrel⟩ := (hk d' p').mp hr
          exact ⟨he, ⟨hh, by simpa using hne⟩, hrel⟩
  · rintro ⟨he, ⟨hh, hne⟩, hrel⟩
    refine ⟨(d, Route.through q), (hk d q).mpr ⟨he, hh, hrel⟩, ?_⟩
    simp only [keepRoute]
    split
    · rename_i heq
      exact absurd heq (by simpa using hne)
    · rfl

theorem step_preserves (s : Ledger) (h : Consistent s) (event : Event) :
    ∀ after, step s event = .accepted after → Consistent after := by
  cases event with
  | set x body guard => exact set_preserves s h x body guard
  | observe e declared guard => exact observe_preserves s h e declared guard
  | predicate key guard => exact predicate_preserves s h key guard
  | qualify e c guard => exact qualify_preserves s h e c guard
  | unqualify e c => exact unqualify_preserves s h e c
  | commit d body => exact commit_preserves s h d body
  | depart o => exact depart_preserves s h o

/-! ## Any history keeps the invariant -/

theorem initial_consistent : Consistent {} := by
  intro p hp
  simp at hp

/-- A refused event leaves the session as it was; an accepted one keeps the invariant. -/
theorem resumed_consistent (s : Ledger) (h : Consistent s) (event : Event) :
    Consistent ((resumeSession s (step s event)).getD s) := by
  cases hs : step s event with
  | accepted after => exact step_preserves s h event after hs
  | rejected reason => exact h
  | fatal reason => exact h

theorem run_preserves (s : Ledger) (h : Consistent s) (events : List Event) :
    Consistent (run s events) := by
  induction events generalizing s with
  | nil => exact h
  | cons event rest ih => exact ih _ (resumed_consistent s h event)

/-- Caveat Sources 0.1, "The invariant", for the model: after any history,
for every current value, its grounds and every stored observation or predicate
qualification, a caveat is carried exactly when it has a kept route, or a route
through a listed occurrence whose relation is live. -/
theorem caveat_sources_invariant (events : List Event) :
    ∀ p ∈ (run {} events).cells, ∀ c,
      c ∈ p.2.caveats ↔
        (c, Route.kept) ∈ p.2.routes ∨ ∃ o ∈ p.2.evidence, Live (run {} events) o c := by
  intro p hp c
  have hm := run_preserves {} initial_consistent events p hp
  simp only [Carrier.caveats, List.mem_map]
  constructor
  · rintro ⟨⟨c', r⟩, hr, rfl⟩
    cases r with
    | kept => exact Or.inl hr
    | through o => exact Or.inr ⟨o, ((hm _ o).mp hr).1, ((hm _ o).mp hr).2⟩
  · rintro (hk | ⟨o, he, hl⟩)
    · exact ⟨_, hk, rfl⟩
    · exact ⟨_, (hm c o).mpr ⟨he, hl⟩, rfl⟩

/-! ## What each event may change -/

theorem unobserved_qualification_rejected (s : Ledger) (e c : String) (guard : List Source)
    (h : ∀ o, s.current.lookup e = some o → o ∉ s.held) :
    step s (.qualify e c guard) = .rejected .unobservedEvidence := by
  simp only [step]
  split
  · rename_i o ho
    simp [h o ho]
  · rfl

theorem take_back_never_refuses (s : Ledger) (e c : String) :
    step s (.unqualify e c) = .accepted (unqualifyLedger s e c) := rfl

/-- A take-back never removes a kept route: a record's own caveats stay. -/
theorem take_back_keeps_kept_routes (held : List Occ) (e c d : String) (k : Carrier)
    (h : (d, Route.kept) ∈ k.routes) : (d, Route.kept) ∈ (takeBack held e c k).routes := by
  simp only [takeBack, List.mem_filter]
  exact ⟨h, rfl⟩

/-- A take-back removes only routes of its own caveat. -/
theorem take_back_removes_only_its_caveat (held : List Occ) (e c : String) (k : Carrier)
    (d : String) (r : Route) (hd : d ≠ c) (h : (d, r) ∈ k.routes) :
    (d, r) ∈ (takeBack held e c k).routes := by
  simp only [takeBack, List.mem_filter]
  refine ⟨h, ?_⟩
  cases r with
  | kept => rfl
  | through o => simp [takenBack, hd]

/-- A `qualify` only adds: every route each cell had, it still has. -/
theorem qualification_only_adds (s : Ledger) (o : Occ) (c : String) (guard : Carrier) :
    ∀ p ∈ s.cells, ∃ q ∈ qualifyCells s o c guard, q.1 = p.1 ∧ p.2.routes ⊆ q.2.routes := by
  intro p hp
  refine ⟨_, List.mem_map_of_mem hp, rfl, ?_⟩
  intro r hr
  have hadd : r ∈ (addRoute o c p.2).routes := by
    unfold addRoute
    split
    · exact List.mem_cons_of_mem _ hr
    · exact hr
  show r ∈ (match p.1 with
    | .lineage _ => if o ∈ p.2.evidence then (addRoute o c p.2).join (addRoute o c guard) else p.2
    | _ => addRoute o c p.2).routes
  split
  · split
    · exact List.mem_append_left _ hadd
    · exact hr
  all_goals exact hadd

/-- Departure changes no caveat a carrier shows: its routes through the departed
occurrence become kept (D018). -/
theorem departure_keeps_caveats (o : Occ) (k : Carrier) : (depart o k).caveats = k.caveats := by
  simp only [depart, Carrier.caveats, List.map_map]
  congr 1
  funext r
  rcases r with ⟨c, _ | p⟩
  · rfl
  · simp only [Function.comp, keepRoute]
    split <;> rfl

/-- Only `commit` changes the recorded bases, and it adds one. -/
theorem bases_frozen (s after : Ledger) (event : Event)
    (hc : ∀ d body, event ≠ .commit d body) (h : step s event = .accepted after) :
    after.bases = s.bases := by
  cases event with
  | set => simp only [step, Outcome.accepted.injEq] at h; subst h; rfl
  | observe => simp only [step, Outcome.accepted.injEq] at h; subst h; rfl
  | predicate => simp only [step, Outcome.accepted.injEq] at h; subst h; rfl
  | qualify e c g =>
      simp only [step] at h
      split at h
      · split at h
        · simp only [Outcome.accepted.injEq] at h; subst h; rfl
        · cases h
      · cases h
  | unqualify => simp only [step, Outcome.accepted.injEq] at h; subst h; rfl
  | commit d body => exact absurd rfl (hc d body)
  | depart => simp only [step, Outcome.accepted.injEq] at h; subst h; rfl

theorem refused_event_changes_nothing (s : Ledger) (event : Event) (reason : RejectionReason)
    (h : step s event = .rejected reason) : run s [event] = s := by
  simp [run, h, resumeSession]

/-! ## Witnesses

Each finding's shape, run on the model. -/

def shows (s : Ledger) (k : Cell) : List String := (cell s k).caveats.eraseDups

/-- D012: a decision made while its evidence was flagged keeps the flag after
the take-back; a value read afterwards does not carry it. -/
def d012 : List Event :=
  [.observe "e" [] [], .set "v" [.evidence "e"] [], .qualify "e" "flaky" [],
    .commit "trust" [.state "v"], .unqualify "e" "flaky",
    .set "v" [.evidence "e"] [], .set "recalled" [] [.basis "trust"]]

theorem d012_decision_keeps_flag :
    shows (run {} d012) (.lineage "recalled") = ["flaky"] ∧
    shows (run {} d012) (.lineage "v") = [] := by decide

/-- D015: a later `qualify` reaches a value read through a basis, and a take-back
of that later doubt removes it again, since it is no part of the record. -/
def d015 : List Event :=
  [.observe "e" [] [], .set "v" [.evidence "e"] [], .commit "trust" [.state "v"],
    .set "recalled" [] [.basis "trust"], .qualify "e" "flaky" []]

theorem d015_late_qualification_reaches_basis_reader :
    shows (run {} d015) (.lineage "recalled") = ["flaky"] ∧
    shows (run {} (d015 ++ [.unqualify "e" "flaky"])) (.lineage "recalled") = [] := by decide

/-- D018: a value keeps a doubt whose occurrence departed; a later take-back
cannot reach it, and it is kept rather than unexplained. -/
def d018 : List Event :=
  [.observe "e" [] [], .qualify "e" "flaky" [], .set "v" [.evidence "e"] [],
    .depart ("e", 0), .unqualify "e" "flaky"]

theorem d018_departed_doubt_is_kept :
    (cell (run {} d018) (.lineage "v")).routes = [("flaky", .kept)] := by decide

/-- D020: a `qualify` after a guard stored a read of the evidence reaches that
stored record through the occurrence, so a take-back removes it from the record
and from a value read through it. -/
def d020 : List Event :=
  [.observe "e" [] [], .predicate "seen" [.evidence "e"], .qualify "e" "flaky" [],
    .set "ev" [.predicate "seen"] []]

theorem d020_stored_record_take_back :
    shows (run {} d020) (.lineage "ev") = ["flaky"] ∧
    shows (run {} (d020 ++ [.unqualify "e" "flaky", .set "ev" [.predicate "seen"] []]))
      (.lineage "ev") = [] ∧
    shows (run {} (d020 ++ [.unqualify "e" "flaky"])) (.predicate "seen") = [] := by decide

end Caveat.Sources
