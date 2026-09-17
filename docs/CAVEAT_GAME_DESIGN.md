# CAVEAT as a game and language

## Product thesis
CAVEAT is not a choice-script language with 3D decoration. Its distinctive mechanic is **acting under incomplete knowledge while preserving the reasons, caveats, and evidence behind a provisional commitment**.

A CAVEAT game should make the epistemic state and the physical world agree. If the program says `stairwell`, the player must encounter a stairwell. If a commitment is reopened, the world must make the new evidence and changed decision context perceptible. No action may be represented by an unrelated teleport.

## Core loop
1. **Situation** — establish a concrete goal and a legible physical environment.
2. **Uncertainty** — claims have evidence and caveats; the player does not automatically know everything.
3. **Attention** — investigating costs budget/time/attention. Investigation reveals evidence; it is not a free dialogue option.
4. **Provisional commitment** — the player acts when they judge they know enough, even though uncertainty can remain.
5. **World consequence** — the commitment causes a mechanically and spatially coherent action.
6. **New evidence** — action can expose information that was unavailable before commitment.
7. **Reopening** — material new evidence can reopen the earlier commitment rather than pretending the previous choice never happened.
8. **Revision** — continue, retreat, or choose an alternative; retained evidence/caveats remain visible in the reasoning state.

The interesting play is therefore not finding a hidden "correct answer." It is deciding **what deserves attention, when evidence is sufficient to act, what uncertainty to retain, and when new evidence justifies changing course**.

## Why use the CAVEAT language
The language should encode concepts that ordinary branching-dialogue code usually leaves implicit:

- claims and evidence are first-class data;
- `supports`, `opposes`, and `qualifies` preserve why information matters;
- caveats can carry consequence/attention rather than being flavor text;
- investigation has an explicit budget/cost;
- commitments retain unresolved evidence instead of collapsing the state to success/failure;
- later observations can `reopen` a commitment;
- revised choices operate on the accumulated epistemic state;
- the same program can drive CLI tests, browser UI, visualization, and a 3D world.

The benefit is strongest when **reasoning state causes world state**. A generic visual novel engine can branch; CAVEAT should model why a branch was reasonable, what remained uncertain, and why it was later revised.

## The Door: canonical scenario model

### Physical layout
The level is one continuous, understandable place:

**Starting office corridor** → **Door A (panic-bar fire door)** → **cross-corridor / protected route** → branches to:

- **forward continuation corridor**;
- **Stair Door B**, clearly signed, leading into a **protected stair enclosure** with landing, handrails, concrete steps and a descending flight;
- **back through Door A** to the starting corridor.

A stair choice must never share the same endpoint with a corridor choice. Doors are barriers: the camera cannot pass through a closed leaf. Door-mounted hardware inherits the leaf transform.

### Stage 1: pre-door investigation
Goal: leave the floor during an alarm.

Available investigations:
- **Inspect latch** — costs attention and reveals the maintenance/calibration problem.
- **Check camera** — costs attention and reveals the blind turn / incomplete coverage.

These alter knowledge, not player position.

### Stage 2: first commitment
- **Open / push panic bar** — physically depress/release hardware, Door A swings about its hinge, then the player crosses the threshold. Crossing begins only after sufficient opening clearance.
- **Wait** — player remains physically in place; time/state changes and new evidence may arrive. No teleport.
- **Reroute** — player turns away from Door A and travels toward an alternate route. If that route is not modeled yet, the action must not be offered in the 3D build.

### Reopening after Open
Opening Door A reveals `smoke_report` / route evidence unavailable from the original side. This can reopen the commitment. The important visual/gameplay beat is: **the player has already acted**. Reopening does not rewind Door A or reset the world.

### Stage 3: revised decision
- **Continue** — continue forward along the protected corridor. Door A remains behind the player in its resulting state.
- **Retreat** — turn approximately 180°, walk back to Door A, traverse it physically, and finish on the original side. If a closer is modeled, interaction must respect the current leaf state rather than teleport through it.
- **Use stairwell** — walk to Stair Door B, operate its hardware, open Door B, enter the stair enclosure, arrive on a landing with unmistakable stairs, then begin descending. This is a distinct route and destination.

## Outcome contract
Every player-facing action needs all of the following before it is considered implemented:

1. **semantic effect** — what happens to claims/evidence/commitments;
2. **physical precondition** — where the player/object must be;
3. **animation/action sequence** — what physically occurs;
4. **destination/world state** — where the player ends and what changed;
5. **feedback** — text/visuals describe what actually happened;
6. **branch-specific automated test**;
7. **branch-specific visual screenshot(s)** inspected by a human/model.

If any item is absent, the option is unfinished and should not be exposed as a working 3D choice.

## Invariants
- Never move a camera through a closed solid door.
- Never use position change alone to represent a named physical action such as climbing stairs, opening a door, retreating, or turning a corner.
- Never reuse one destination for semantically different routes unless the story explicitly says they converge and the traversal demonstrates that convergence.
- Never hide an abstract marker without replacing the physical thing it was standing in for.
- UI outcome text must be derived from the same action that drives world behavior.
- New evidence may change what is rational to do, but it does not retroactively erase prior physical actions.
- Epistemic state and world state persist together.

## QA matrix for The Door
Automated visual QA must cover at least:

| Route | Required captures/assertions |
|---|---|
| inspect latch | same position; latch/evidence feedback changes |
| inspect camera | same position; camera/evidence feedback changes |
| open | closed Door A → mid-swing → threshold crossing → beyond Door A |
| wait | no position jump; world/time feedback changes |
| reroute | turn/movement toward a modeled alternate route |
| open → continue | forward corridor traversal; no stair geometry claimed |
| open → retreat | turn back; Door A encountered/traversed physically; original side reached |
| open → stairwell | approach Stair Door B → Door B opens → stair landing → visible descending stairs |

A generic "canvas changed" assertion is insufficient. Each route must assert the expected action/destination and save its own visual evidence.

## Definition of done
The Door is a useful CAVEAT demonstration only when a new player can infer, without reading source code:

- where they are and what their goal is;
- what is known versus uncertain;
- why investigating costs something;
- that they can commit without perfect certainty;
- that action reveals new evidence;
- why a previous commitment reopened;
- how continue/retreat/stairwell differ physically and epistemically;
- what CAVEAT provides that a conventional branching script does not.
