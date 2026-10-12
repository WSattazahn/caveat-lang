# Sample 003: the presentation pass deferred to after 1.0

**Written after the fact.** The plan asked for 003 to be written live on the
next owner card that drops or defers something. The samples were paused from
2026-10-05 until the owner chose "Field test" at 15:28 UTC on 2026-10-12, so
this program was written at about 15:45 UTC that day from a decision made at
03:21 UTC. It is a retro-fit, like 001, not a live sample.

Run against the published `caveat-lang@0.1.0-rc.19` (doctor: wasm sha256
`3c894023…`), pinned by this directory's `package.json`. Skill text:
`skills/caveatist/SKILL.md` at main `21594cc`, sha256 `4b562d8e…`, identical in
its three copies.

## The decision

At 03:21:06 UTC the owner tapped "After 1.0" on the card "Run the game
presentation pass now or after 1.0?", two seconds after "Freeze now" on its
sibling card. They typed no words with either tap. The request behind both was
the owner's 02:48 UTC "so what comes next? work on v19?". The card's options and
reason are in `docs/releases/rc19-development.md` L83–92, "as the owner saw them
on the cards" (L69–70), and the answer is at L14:

- A, After 1.0 (recommended): the game stays as #184 left it. Cost: none now.
- B, Alongside rc.19: one more thread on `web/` and `game/`, with #179 brought
  up to main first (D1). Cost: about 5 days of one more thread.
- Why A: the game does not gate 1.0, the 1.0 promise does not cover it, and one
  thread keeps the cycle bounded. Under B the freeze would be unaffected,
  because `docs/COMPATIBILITY.md` (L70–72) lists `web/` and `game/` as not
  stable.

The posted card is not in the repository. Only the plan's record of it is,
which is why `why_a` and `owner_tap` carry `secondhand`.

## The program

`defer_pass` commits `using qualified(1, why_a) + qualified(1, game_unstable)`,
`permitted by owner_tap`, `retaining presentation_gaps, pr179_drift`. The two
caveats not pursued are carried by the evidence that shows them (`game_at_184`
and `pr179_behind`), not by the reason they were left. `one_oh_ships` reopens
the deferral, because "after 1.0" names its own revisit point and nothing
reopens by itself. Four scenarios pass. `explain.txt` and `dependents.txt` are
the checked-in outputs.

## Step 3: what changes if each caveat is true

- `presentation_gaps` (not pursued). The game looks unfinished: framing, NPC
  and prop consistency, placeholder art, dialogue and controls (D4). It would
  change whether outreach points at the game. Cost: low. COMPATIBILITY.md
  already says the game is not stable, and outreach waits for the owner's
  approval of target and message. Not pursued, for the reason in `why_a`.
- `pr179_drift` (not pursued). Draft #179 falls further behind main with each
  merge, so D1's cost grows. It would change the "about 5 days" estimate for
  the pass after 1.0, not the deferral. Cost: low. Not pursued, for the same
  reason.
- `secondhand` (inherited from `why_a` and the grant). The plan's record could
  differ from the posted card. That would change which reason this sample
  names, not the deferral. Cost: low. The rc.19 plan thread's 02:54 UTC draft
  could settle it if it ever matters.

## Where the reason went

The plan asked where the reason for a caveat not pursued goes: the caveat's
declaration, a separate `evidence`, a `bind`, or NOTES.md. All four were
tried. The program puts the reason in the deferral's own grounds, and this file
keeps the pairing.

| Place | Result on rc.19 | Receipt |
| --- | --- | --- |
| The caveat's declaration | does not load: `cannot parse statement: caveat gaps consequence low because "…"` | `probes/caveat-reason.txt` (source and output) |
| On `retaining` | does not load: `invalid reactive identifier k because e` | `probes/retaining-because.txt` (source and output) |
| A separate `evidence`, as the decision's grounds | works, and checked in. For a deferral, the decision is the choice not to pursue, so its grounds are the reason. Nothing pairs a reason with a retained caveat, so with two caveats left for two different reasons the pairing is lost. | `explain.txt` |
| One decision per deferred caveat | pairs each caveat with its own reason (`defer_pass` retains `presentation_gaps` on `why_a`, `defer_179` retains `pr179_drift` on `d1_only_for_b`), but splits one tap into two decisions | `probes/two-decisions.*` |
| A `bind` | displays prose. It cites the whole decision (`because game_unstable, owner_tap, why_a`), not the reason it names | `probes/bind-reason.*` |
| NOTES.md | holds the pairing and the "not pursued" status. The runtime knows neither | this file |

The `because` slot cannot hold it either. `commit … because REASON` takes
`enough`, `budget` or `deadline` (Reactive 0.1 L161), which are stopping rules.
`budget` is valid only after a declared ledger is exhausted, so "out of scope
for this release" is written `because enough`.

## Findings

**D006: grammar gap. A caveat chosen not to pursue has no place for its reason
or its status.**
- Needed: skill step 3, "Record caveats you chose not to pursue as such, with
  the reason" (`skills/caveatist/SKILL.md` L23).
- Wrote: `retaining presentation_gaps, pr179_drift` and the reason as grounds.
  A reason on the declaration or on `retaining` does not load (table above).
- Concerns: Permission 0.1 L27 (`retaining C, …` takes names only) and
  Reactive 0.3 L140.
- In the runtime's record, a caveat retained because it was not pursued looks
  the same as one retained while it waits (001's `ci_pending`) and as one the
  grounds brought in (D007). The workarounds are one decision per deferred
  caveat, or a reason kept only in prose.
- If fixed: authors and reviewers could ask "what did we choose not to look at,
  and why?" of the record itself. That is the step-3 question the skill puts
  to every decision. It is a 1.1 addition at the earliest, since the soak
  admits no new syntax (`docs/releases/path-to-1.0.md` L101–105).

**D007: tooling, with a spec question. `retaining:` merges caveats the grounds
carried with caveats the author retained.**
- Needed: to tell `secondhand`, which `why_a` brought into the grounds, from
  `presentation_gaps`, which the author retained.
- On rc.19, `explain` prints `based on game_unstable, why_a` and then
  `retaining: presentation_gaps, pr179_drift, secondhand`. `dependents …
  secondhand` labels the decision `retaining: secondhand`, and its Claims
  section shows `why_a` is the carrier. The commit effect's `retained` and the
  journal's `caveats` both hold all three.
- Concerns: Reactive 0.3 L143–146 unions a basis's caveats with explicitly
  retained ones, so the runtime keeps no distinction. Dependents 0.1 L98–100
  labels any caveat in `retained` as `basis: "retained"`, while L21–26 give one
  reported basis by precedence. `kit/docs/GETTING_STARTED.md` L278–281 shows
  the same rendering for `forecast_is_old`, which its grounds carry and the
  source never retains. This
  is documented behavior, so I log it for after 1.0 and don't treat it as a
  soak fix. Whether L98–100 or L21–26 should win for a caveat both carried and
  retained is the owner's call.
- This sharpens D003's fix. On rc.15 the grounds line showed which evidence
  carried a caveat, and the problem was an authored caveat printed as if
  evidence carried it. On rc.19 it is the reverse: a carried caveat is printed
  as if the author retained it.
- If fixed (the commit record keeps authored `retaining` apart from inherited
  caveats, and the text shows each): a reader of any decision with caveated
  grounds can see which caveats were a choice.

## Known, hit again

- "There is no fallback" (AI_AUTHORING): 1. "After 1.0" needs an explicit
  `reopen defer_pass because release_1_0` rule.
