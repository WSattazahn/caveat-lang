# Probe run 1: Fable (Claude family), not counted

- **Received:** the owner posted it in the project chat at 2026-10-10 23:26:21 UTC (message `cmsg_015odJFpwogpoW5yAoJxQmJwVsKmifFkp3C5Fpim9WsPkr`), headed "fable response".
- **Model:** Fable, as named by the owner. Fable is a Claude model, so it belongs to the same family as the round's authors.
- **Prompt:** `probe/prompt.txt`, registered at stage 1 (SHA256 `55e771f8…`). The owner sent it from `/mnt/project-files/glowcap/round8/probe-prompt.txt`, which has the same hash.
- **Status: not counted.** The protocol requires "a model from a family other than the authors'" ("The learnability probe"; amendment 2). This run produces no guide or packet items. The reply is kept so the record has it. Any diff against it is information only and never feeds the probe's numbers.

The reply follows exactly as posted. The heading word and the owner's label have been moved into this header.

---

1. The Caveat program

```caveat
program glowcap

-- ---------------------------------------------------------------- vocabulary
kind mushroom_kind = glowcap | duskcap

caveat tasted_in_dark  "tasted while the slime was not glowing"
caveat taste_faded     "more than 60 s of play have passed since this taste"
caveat secondhand      "seen happening to another creature, not experienced"

const GLOW_SECONDS    = 30
const HEAVY_SECONDS   = 20
const HEAVY_CAP       = 30
const FADE_SECONDS    = 60
const REGROW_SECONDS  = 45
const MAX_LIVES       = 64
const TWICE_BITTEN    = 2

-- ------------------------------------------------------------ ordinary state
clock play                       -- advanced only by `tick`; all timers hang off it
value glow_timer  : seconds = 0
value heavy_timer : seconds = 0

-- --------------------------------------------------------- epistemic objects
-- A mushroom is a renewable name: its current occurrence is one "life".
-- Evidence observed about a life is named `<verb>_<id>` for life 1 and
-- `<verb>_<id>_<n>` for life n > 1.
entity mushroom renewable lives MAX_LIVES {
  consumed_by : evidence?       -- what ate the current life, if anything
  taste       : evidence?       -- the taste of the current life, if any
}
mushrooms = { cave: mushroom, pool: mushroom, ruin: mushroom, grove: mushroom }

claim safe "Glowing mushrooms give you light."

decision trust "absorb glowing mushrooms without tasting"

-- supporting observations seen since the most recent contradiction while
-- trust is reopened (CR7); evidence references, not a confidence score
value recovery_support : evidence list = []

-- ------------------------------------------------------------ pure functions
fn present(m)        = m.consumed_by == none
fn known(m)          = m.taste != none
fn twice_bitten()    = count(safe.opposition) >= TWICE_BITTEN
fn stance_name(s)    = match s { none -> "none", supported -> "probably_safe",
                                 opposed -> "probably_unsafe", contested -> "uncertain" }
fn note_for(n)       = if n == 1 then "Based on one observation."
                       else "Based on " ~ text(n) ~ " observations."
fn ceil_seconds(t)   = if t > 0 then ceil(t) else 0

fn can_absorb(m) =
  present(m) and
  (if known(m) then m.taste.kind == glowcap else not twice_bitten())

fn can_taste(m) = present(m) and not known(m)

fn tasted_label(m) =
  let base = if m.taste.kind == glowcap then "glowcap" else "duskcap" in
  if   m.taste has taste_faded    then "Probably a " ~ base ~ " (taste has faded)"
  elif m.taste has tasted_in_dark then "Probably a " ~ base ~ " (tasted in the dark)"
  elif m.taste.kind == glowcap    then "Glowcap"
  else                                 "Duskcap — avoid"

fn unknown_label() =
  if twice_bitten() then "Too risky — taste first"
  else match stance(safe) {
    none      -> "Glowing mushroom"
    supported -> "Probably a glowcap"
    opposed   -> "Probably a duskcap"
    contested -> "Could be a duskcap — taste first"
  }

-- which evidence a mushroom's own view cites (CR2, CR4 applied)
fn unknown_because() =
  if twice_bitten() then safe.opposition
  else match stance(safe) {
    none      -> []
    supported -> safe.support
    opposed   -> safe.opposition
    contested -> safe.opposition          -- CR4: cite the counterexample only
  }

fn because_for(m) =
  if not present(m) then []
  elif known(m)     then [m.taste]
  else                   unknown_because()

-- --------------------------------------------------------- shared procedures
-- Called inside every learning event. The call freezes `ev` and `k` with
-- their caveats; the decision's basis inherits them automatically.
proc learn(ev : evidence, k : mushroom_kind) {
  if k == glowcap {
    ev supports safe
    match status(trust) {
      none      -> if stance(safe) == supported {
                     commit trust on safe.support        -- basis frozen here
                   }
      committed -> skip
      reopened  -> {
        recovery_support <- recovery_support ++ [ev]
        if count(recovery_support) == 2 {
          revise trust on recovery_support                -- new revision, new basis
          recovery_support <- []
        }
      }
    }
  } else {
    ev opposes safe
    recovery_support <- []
    if status(trust) != none {
      reopen trust because ev          -- appends to reopenedBy of the current revision
    }
  }
}

proc consume(m : mushroom, ev : evidence) {
  m.consumed_by <- ev
  if m.lives_remaining > 0 {
    after REGROW_SECONDS on play { renew m }   -- bound to this life; new life is blank
  }
}

-- ------------------------------------------------------------------- events
-- An event is one transaction: any `reject` or failed `require` publishes
-- nothing (no values, graph edges, cues, occurrence ids, or schedules).

event absorb(id : mushroom_id, k : mushroom_kind) {
  let m = mushrooms[id]
  require present(m)                               "mushroom consumed"
  require m.taste == none or m.taste.kind == k     "kind contradicts a previous taste"
  require can_absorb(m)                            "cannot absorb this mushroom"

  let ev = observe absorb(m) as k                  -- fresh occurrence absorb_<id>[_n]
  consume(m, ev)
  if k == glowcap {
    glow_timer <- GLOW_SECONDS
  } else {
    heavy_timer <- if heavy_timer > 0
                   then min(heavy_timer + HEAVY_SECONDS, HEAVY_CAP)
                   else HEAVY_SECONDS
  }
  learn(ev, k)
}

event taste(id : mushroom_id, k : mushroom_kind) {
  let m = mushrooms[id]
  require present(m)           "mushroom consumed"
  require m.taste == none      "mushroom already tasted"

  let ev = observe taste(m) as k                   -- taste_<id>[_n]
  if glow_timer <= 0 { qualify ev with tasted_in_dark }
  m.taste <- ev
  after FADE_SECONDS on play { qualify ev with taste_faded }   -- bound to this occurrence
  learn(ev, k)
}

event witness(id : mushroom_id, k : mushroom_kind) {
  let m = mushrooms[id]
  require present(m)                               "mushroom consumed"
  require m.taste == none or m.taste.kind == k     "kind contradicts a previous taste"

  let ev = observe witness(m) as k with secondhand -- witness_<id>[_n]
  consume(m, ev)                                   -- timers untouched
  learn(ev, k)
}

event tick(dt : number) {
  require dt >= 0 and dt <= 0.1  "dt out of range"
  advance play by dt             -- fires due fades and regrowths inside this transaction
  glow_timer  <- max(0, glow_timer  - dt)
  heavy_timer <- max(0, heavy_timer - dt)
}

-- ------------------------------------------------------------- presentation
-- Bindings project state; they cannot write to the graph. `because` lists
-- grounds (a subset of lineage) and `caveats` is derived from them.

bind view.slime = {
  glowing:      glow_timer  > 0,
  heavy:        heavy_timer > 0,
  heavySeconds: ceil_seconds(heavy_timer),
}

bind view.belief =
  let s = stance(safe) in {
    state:          stance_name(s),
    text:           match s { none      -> "",
                              supported -> "Glowing mushrooms give you light.",
                              opposed   -> "Glowing mushrooms make you heavy.",
                              contested -> "Not every glowing mushroom is safe. Taste before absorbing." },
    note:           match s { supported -> note_for(count(safe.support)),
                              opposed   -> note_for(count(safe.opposition)),
                              _         -> "" },
    supportedBy:    ids(safe.support),
    contradictedBy: ids(safe.opposition),
    caveats:        caveats_of(safe.support ++ safe.opposition),
  }

bind view.decision = {
  state:      text(status(trust)),              -- "none" | "committed" | "reopened"
  basis:      ids(trust.basis),                 -- current revision, frozen at commit
  reopenedBy: ids(trust.reopened_by),           -- current revision only
  caveats:    trust.retained_caveats,           -- as carried at commit; later fades don't change it
  history:    history(trust) map (h -> {
                change:  text(h.change),        -- "committed" | "reopened"
                because: ids(h.because),
              }),
}

bind each (id, m) in mushrooms -> view.mushrooms[id] =
  let because = because_for(m) in {
    present:   present(m),
    label:     if not present(m) then ""
               elif known(m)     then tasted_label(m)
               else                   unknown_label(),
    canAbsorb: can_absorb(m),
    canTaste:  can_taste(m),
    because:   ids(because),
    caveats:   caveats_of(because),
    why: {
      absorb: explain(
        if not present(m)                      then ("Already eaten",      [m.consumed_by])
        elif known(m) and m.taste.kind == duskcap then ("Known duskcap",   [m.taste])
        elif not known(m) and twice_bitten()   then ("Too risky untasted", safe.opposition)
        else                                        ("",                   [])),
      taste: explain(
        if not present(m) then ("Already eaten",   [m.consumed_by])
        elif known(m)     then ("Already tasted",  [m.taste])
        else                   ("",                [])),
    },
  }

-- turns (reason, evidence list) into { reason, because, caveats }
fn explain(pair) = { reason: pair.0, because: ids(pair.1), caveats: caveats_of(pair.1) }

```

2. The adapter

```js
import { loadProgram } from '@caveat/runtime';
import source from './glowcap.caveat?raw';

const program = loadProgram(source);

export function createPolicy(saved) {
  const session = saved === undefined
    ? program.start()
    : program.resume(saved);           // runtime snapshot: values, graph, clock, pending schedules

  return {
    dispatch(event) {
      const { type, ...payload } = event;
      // Runtime rejections throw; a rejected transaction publishes nothing,
      // so view() is unchanged by construction.
      session.dispatch(type, payload);
    },
    view() {
      // Bindings already produce the exact shape; copy so callers can't poke the runtime.
      return structuredClone(session.view());
    },
    save() {
      return session.snapshot();       // plain JSON value
    },
  };
}

```

This is barely an adapter — it is the wrapper I would expect to need to turn `dispatch`/`view`/`save` into the harness's names and to make the view a detached copy. Every piece of policy, text and `because` lives in the program.
3. Assumptions and invented constructs
Top-level declarations

* `program`, `const`, `fn`, `proc`, `event`, `bind` — I assumed the usual split the description implies: pure functions (`fn`) carry dependencies but have no effects; effect procedures (`proc`) freeze their arguments and share the caller's transaction (Reactive 0.7); events are the host-facing transactions.
* `kind mushroom_kind = glowcap | duskcap` — an enum. Unknown `kind` in a payload is rejected by typing; likewise `mushroom_id` is assumed to be the key type of `mushrooms`, so an unknown id is a typed rejection.
* `value x : T = …` — an ordinary deterministic value (timers, the recovery list). `<-` is assignment inside a transaction.
* `clock play` and `advance play by dt` — I assumed the runtime has authored clocks and that `after N on <clock> { … }` schedules a block that fires during the `advance` that crosses the deadline, inside the same transaction. "Counted in ticks since" maps onto this directly.
* `caveat name "text"` — declares a caveat; `qualify ev with c` attaches it to one occurrence; `ev has c` tests it.

Evidence and claims

* `claim safe "…"` — a claim with two edge sets: `safe.support` and `safe.opposition`. `ev supports safe` / `ev opposes safe` add stance-bearing edges. `stance(safe)` returns `none | supported | opposed | contested` purely from edge presence, which I map onto the feature's four belief states. I assumed the runtime never collapses this into a score.
* `observe absorb(m) as k` — produces a fresh evidence occurrence, with the runtime naming it from the verb, the entity's id and its current life (`absorb_cave`, then `absorb_cave_2`). This is the biggest guess: I'm assuming occurrence naming can be authored via the entity's renewal state rather than being opaque.
* `observe … with secondhand` — a caveat declared on the evidence at observation time (the doc says a new occurrence "inherits only the caveats declared on the evidence"; `secondhand` is that kind).
* `ids(list)` turns evidence references into their string ids; `caveats_of(list)` is the union of caveats on those occurrences; `count(list)` is a count that keeps membership qualifications. I assumed the view's `caveats` is always derived from its `because`, never stored separately — except on the decision, where it is the retained set frozen at commit (CR5), which the description says the runtime does automatically.

Renewable entities (CR10)

* `entity mushroom renewable lives MAX_LIVES { … }` — I merged the description's "renewable name" with a small record. `renew m` gives a new, blank life (fields reset to `none`); `m.lives_remaining` reads the budget. Earlier occurrences and any `after` blocks bound to them keep working, which is exactly what CR10 wants: an old taste still fades, old evidence stays in the claim.
* I assumed a `renew` on a name with no lives left is unreachable because `consume` only schedules it when lives remain; the mushroom therefore stays consumed and ticks are never rejected.

Decisions

* `decision trust "…"` with `status(trust)` ∈ `none | committed | reopened`, `trust.basis`, `trust.reopened_by`, `trust.retained_caveats`, and `history(trust)`.
* `commit trust on <evidence list>` — first commitment, basis frozen, caveats retained.
* `reopen trust because ev` — appends to `reopened_by` and the history. I assumed reopening an already-reopened decision is allowed and just appends (the beat says every duskcap after commit reopens it, and `reopenedBy` lists them in order).
* `revise trust on <evidence list>` — a new revision in the series, allowed only after an explicit reopen (per the description), with a fresh basis and empty `reopened_by`. I chose to record it in history as `change: 'committed'` because the feature's history vocabulary has only two words.
* `recovery_support` as a `value` of evidence references is a cheat I'm unsure about: the description says ordinary values and evidence are distinct, and it's unclear whether a list of evidence references is an "ordinary value". If not, the natural alternative is a fold over `safe.support` filtered to occurrences after the last opposition.

Events and rejection

* `require cond "message"` rejects the transaction; I assumed a rejection throws out of `dispatch` and publishes nothing, so "leave the view exactly as it was" is free.
* Argument order of rejection checks: for `absorb` I check taste contradiction before `can_absorb` so a tasted-duskcap `absorb(…, glowcap)` is reported as a contradiction rather than a refusal; the spec only says "rejected", so either message is fine.

Presentation

* `bind view.x = {…}` and `bind each (k, v) in map -> view.y[k] = {…}` — projection bindings that run after every committed event. I assumed they can read everything and write nothing, and that the `because` lists I emit are checked by the runtime against lineage (grounds ⊆ lineage). Nothing here uses `examine`, so no attention budget is spent by display.
* `match`, `if/elif/else`, `let … in`, `~` for string concatenation, `++` for list append, `text(n)` for number formatting, tuples with `.0`/`.1`. All invented; the description only promises formatting lives in Caveat.

Unsure how the language would express

* CR2/CR4 ordering: I put "too risky" above the belief rows and let CR4 only change the `uncertain` row's `because`; the feature doesn't say whether twice-bitten also changes `because` for known mushrooms (I left known ones alone, as CR2 says).
* Whether the runtime's fade scheduling survives a `renew` of the same name — the description says a scheduled thing is bound to the occurrence current when scheduled, so I relied on that rather than storing a per-taste timestamp.
* CR12 save size: I assumed `snapshot()` serialises the graph, values, clock and pending schedules — not the event log — so ten minutes of ticks add nothing, and ~13 lives per mushroom of evidence with caveats fits in 4 KB. If the snapshot included ticks as occurrences this would not hold.
* Whether `count(safe.opposition)` being a qualified number is a problem when compared with a constant inside `twice_bitten()` — I assumed the comparison just carries the qualification through, per "comparisons preserve those dependencies".
