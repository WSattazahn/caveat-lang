# Design note: graded belief

Status: a design note. The owner chose P1 and P2 for after 1.4.0 (see
"Decisions" at the end). Nothing here is implemented yet, and no specification changes with
it. Each proposal below is additive (new syntax or a new tool) and would need
its own specification PR and corpus sweep under the
[compatibility commitment](../COMPATIBILITY.md). Receipts are `path: Lx–Ly`
at `main` `af61d2e` unless marked. The programs were checked with
`check --strict` and run with `explain` on the runtime built from that
commit, and again on `41171fe`, after the D024 fix described below.

## The question

Caveat answers "what does this rest on, and what is known to be wrong with
it". People who work with uncertainty ask a second question: "how sure?" A
robot acts on a probability that the door is open. A risk model gives a
patient a five-year risk score. A guardrail model estimates the chance that
an agent's action causes harm. Can Caveat carry "how sure", and where does
that stop?

The [probabilities note](../PROBABILITIES.md) answers what works today: a
score from another tool is sent as a reading of named evidence, carries that
evidence's caveats into a decision's grounds, and can be withdrawn. This note
takes the next step. It runs three test cases against the runtime, lists
where they hurt, and proposes the smallest additions that would help.

## What stays fixed

Graded belief must not change what Caveat already means:

- **The graph is not a confidence score.** "The graph is not a confidence
  score, and this must not become the first place one creeps in"
  (`spec/caveat-borrowed-uncertainty-0.1.md: L60–62`). `observed`,
  `committed` and `reopened` "do not turn the graph into a single confidence
  score" (`docs/CAVEAT_ESSENCE.md: L17`).
- **A caveat's consequence is a level, not a likelihood.** It says how much a
  caveat matters. Nothing below turns it into a number.
- **The runtime does not combine grades.** If a program averages two scores,
  the arithmetic is the program's, and the result rests on both readings.
  Caveat does not propagate a number through lineage, as provenance
  semirings do.
- **Withdrawal removes nothing and lowers nothing.** It marks what rests on
  the evidence. A program that wants a lower score computes one.
- **Dependence, not cause.** Lineage lists what an evaluation read, not how
  much each piece moved the result (see the probabilities note, "Dependence,
  not cause").

Within those lines, a grade is an ordinary number with a source. The work is
in recording it well: which grade a decision used, what threshold it was held
to, and what was known to be wrong with the tool that produced it.

## Three test cases

### 1. A robot acting on a belief (Kaelbling)

Planning under partial observability chooses actions from a belief, a
probability over the states the robot might be in, and an action can be one
that gathers information rather than one that makes progress (Kaelbling,
Littman and Cassandra, "Planning and acting in partially observable
stochastic domains", *Artificial Intelligence* 101, 1998). The robot's filter
keeps the belief. Caveat keeps which belief each step acted on:

```caveat
claim door_open;
evidence estimator from "the robot's state estimator: probability the door ahead is open";
caveat single_camera consequence material;
single_camera qualifies estimator;
readings beliefs from estimator limit 32;
decisions plan limit 32 reopened by beliefs;
event belief p min 0 max 1;
on belief sample beliefs = p supports door_open;
on belief when not committed(plan) or reopened(plan)
    commit plan because enough
    using if(latest(beliefs) >= 0.9, 2, if(latest(beliefs) >= 0.3, 1, 0));
```

The plan is 2 (go through), 1 (look again) or 0 (replan). After beliefs of
0.55 and then 0.95, `explain` shows `plan@1 = 1` based on `beliefs@1`,
reopened by `beliefs@2`, and `plan@2 = 2` based on `beliefs@2`, each
carrying `single_camera`. This works today.

**Where it hurts.** The two thresholds, 0.9 and 0.3, are in the rule but not
in the record. The record says the plan read 0.55 and chose 1. It does not
say "between 0.3 and 0.9, so look". A reviewer reconstructs that from the
source, which the record binds by its SHA-256 but does not quote.

### 2. A medical risk flag (Barzilay)

Image-based risk models give a patient a five-year risk score, and such a
model has to be checked on each population it is used for, because a score
calibrated at one hospital may not hold at another (Yala et al., "Toward
robust mammography-based models for breast cancer risk", *Science
Translational Medicine* 13, 2021). Both facts are caveats in Caveat's sense:
a score is a flag, not a diagnosis, and it may be calibrated elsewhere.

```caveat
claim high_risk;
evidence risk_model from "an image-based five-year risk model's score for this patient";
evidence biopsy from "the patient's follow-up result";
evidence scorer from "squared error of a resolved risk score";
renewable risk_model limit 64;
caveat calibrated_elsewhere consequence material;
caveat flags_not_diagnoses consequence material;
flags_not_diagnoses qualifies risk_model;
readings scores from risk_model limit 64;
readings followups from biopsy limit 64;
readings errors from scorer limit 64;
decisions referral limit 64 reopened by scores;
event score p min 0 max 1, other_site min 0 max 1;
event followup positive min 0 max 1;
on score renew risk_model;
on score reveal risk_model;
on score when other_site >= 1 qualify risk_model with calibrated_elsewhere;
on score sample scores = qualified(p, risk_model) supports high_risk;
on score when not committed(referral) or reopened(referral)
    commit referral because enough using if(latest(scores) >= 0.2, 1, 0);
on followup sample followups = positive;
on followup resolve referral = positive because latest(followups);
on followup sample errors = (latest(scores) - outcome(referral)) * (latest(scores) - outcome(referral));
```

A score of 0.31 from another site refers the patient, and the referral
carries both caveats. The follow-up resolves it as 0, through
[Outcomes 0.1](../../spec/caveat-outcomes-0.1.md), and records a squared
error of 0.0961 that rests on the score it judges and the follow-up that
resolved it. That pairing of forecast and outcome is what calibration needs
(`spec/caveat-outcomes-0.1.md: L186–207`). A second patient's score of 0.12
from this site gives `referral@2 = 0`, based on a score carrying only
`flags_not_diagnoses`.

**Where it hurts.** To keep "calibrated elsewhere" on one patient's score
and off the next, the program renews the model's evidence for every score,
reveals it, and wraps the number in `qualified(p, risk_model)`. Three rules
do the work of one idea: this caveat is about this reading. It also showed
the gap below: before 1.4.0 the reading `scores@1` itself did not carry
`calibrated_elsewhere`; only the `qualified(…)` wrapper put it in the
grounds.

### 3. A guardrail's risk estimate (Bengio)

One proposal for keeping a capable agent in check is a second model that
does not act: it estimates how likely an action is to cause harm, and the
action is blocked when the estimate is too high (Bengio et al.,
"Superintelligent Agents Pose Catastrophic Risks: Can Scientist AI Offer a
Safer Path?", arXiv:2502.15657, 2025, a preprint). The project's private
guardrail demo builds this on 1.3.0 with three buckets, `low`, `high` and
`unknown`, each its own evidence. It names two rough spots this note should
answer: the real output is a probability and the threshold that made the
bucket "appears nowhere in the record", and a call "cannot be judged twice"
because caveats attach to evidence, not to individual readings.

With a probability and one assessment per occurrence, it works today:

```caveat
claim action_is_safe;
evidence guard from "the guardrail model's estimated probability of harm for the proposed action";
evidence person from "a person confirmed this action";
renewable guard limit 64;
caveat guard_flags_risk consequence high;
readings risks from guard limit 64;
decisions delete_records limit 1 irreversible;
event assess p min 0 max 1;
event run;
event confirm;
on assess renew guard;
on assess reveal guard;
on assess when p >= 0.2 qualify guard with guard_flags_risk;
on assess sample risks = qualified(p, guard);
on run when not committed(delete_records)
    commit delete_records because enough using latest(risks);
on confirm reveal person supports action_is_safe;
on confirm when not committed(delete_records)
    commit delete_records because enough using latest(risks) retaining guard_flags_risk;
```

An estimate of 0.35 qualifies that assessment with `guard_flags_risk`
(`high`), so `run` is refused as `evaluation/caveated_irreversible`
(`spec/caveat-foreclosure-0.1.md: L104–112`) and `confirm` commits with the
risk retained on purpose. Each `assess` renews the guard, so a call can be
judged again: after estimates of 0.05 and then 0.35, only `guard@3`, the
second assessment, carries the flag.

**Where it hurts.** The threshold 0.2 is again only in the source. And the
version a reader would write first, `sample risks = p;` with the `qualify`
before it, was not protected before 1.4.0: `run` committed, because the
reading did not carry the flag. That is the gap below.

## A gap the examples found: readings from renewable evidence (D024)

Up to 1.3.0, a reading stream declared `from` renewable evidence took its
caveats from the declared evidence, not from the current occurrence. A sample
linked the "template qualifications" (`spec/caveat-reactive-0.5.md:
L60–66`), and the template is the declared evidence, while
[Renewal 0.1](../../spec/caveat-renewal-0.1.md)'s list of forms that follow
the current occurrence did not include `readings … from`
(`spec/caveat-renewal-0.1.md: L37–45`). So a caveat that `qualify` added to
`guard@2` did not reach a reading sampled after it. That drops a doubt, which
matters most for an irreversible step.

The bug hunt took it as finding D024 and fixed it for 1.4.0 as an add-only
change (#351, merged as `41171fe`): a reading from renewable evidence now
also inherits the caveats of the occurrence current when it is taken
(`spec/caveat-renewal-0.1.md`, Changes, 2026-10-25). Readings only gain
caveats: a late caveat on the first occurrence still reaches every later
reading, as before. On `41171fe` the guardrail program written with
`sample risks = p;` refuses `run`, and the screening program's `scores@1`
carries `calibrated_elsewhere`. On 1.3.0, `qualified(p, EVIDENCE)`, as in
the examples above, names the current occurrence and is safe either way.

## Proposals

In order of how much they help the three cases. Each is opt-in: a program
that does not use it loads, runs, saves and explains byte for byte as
before.

### P1. A caveat on one reading

```text
on EVENT [when CONDITION] sample STREAM = EXPRESSION [supports CLAIM | opposes CLAIM] with CAVEAT [, CAVEAT …] [when CONDITION];
```

For example `on assess sample risks = p with guard_flags_risk when p >= 0.2;`.
The caveats qualify that reading's own identity, `risks@N`, and nothing else:
every value computed from `risks@N` carries them, as a declared
qualification's caveats are carried, and no other reading or occurrence does.
This is the guardrail demo's "per-reading caveat (a `high` sample in a stream
qualified, a `low` one not)", and it replaces the renew, reveal and
`qualified(…)` trio in the screening and guardrail programs with one clause.

Design points for the spec PR:

- The trailing `when` reads the same values as the sample's expression, and
  its lineage joins the reading's, as a sampling guard's does.
- It records nothing new in the snapshot: a reading's `caveats` list exists
  already. A save holds the qualification as it holds any other; restore
  accepts it only where a `sample … with` rule of the source could have
  produced it (the [A3 boundary](../RESTORE_TRUST_BOUNDARY.md)).
- It needs no new refusal code. An undeclared caveat is a load-time error,
  as for `qualify`.
- `unqualify` and take-back reach it as they reach any qualification of that
  reading. Withdrawal works as it does today. (The spec narrowed this: a
  take-back caveat cannot be named in `with`, so a doubt that must be taken
  back stays a late `qualify`.)

Specified and implemented as
[Reading Caveats 0.1](../../spec/caveat-reading-caveats-0.1.md), a 1.5 draft.

### P2. A threshold the record names

```text
threshold NAME = NUMBER;
```

A threshold is a named number in the source, read wherever a number is.
Unlike a [define](../../spec/caveat-define-0.1.md), which is inlined and
leaves no trace (`spec/caveat-define-0.1.md: L22–30`), a threshold read by an
evaluation is listed in that value's record, with its value: `explain` would
print `plan@1 = 1 ... thresholds: look_bar = 0.3, go_bar = 0.9`, and
`caveat-explain/0.1` would gain an optional `thresholds` field, left out when
a revision read none. This answers "the threshold was 0.5 and the estimate
0.62" from the record, for all three cases.

A threshold is not state: it cannot change during a session, so it adds no
lineage. Changing it is a source change, which the record already tells
apart by the source's SHA-256.

A later tool could then answer the guardrail demo's other half ("a later
change of threshold cannot find the calls it would have flipped"): replay the
same events with one threshold changed and list the decisions whose value
differs. That is a counterfactual about the program, not the world, and it
needs only replay and P2's record.

### P3. A calibration summary over outcomes (later, tooling only)

Outcomes 0.1 already pairs a decision's score with what happened, and a
program can fold the squared errors itself. A CLI summary, for example
`caveat calibration program.cav events.jsonl --stream scores --series referral`,
could print the Brier score and a reliability table by score band for the
revisions that have outcomes, each row linked to the revisions it counts. It
computes nothing a program could not, so it can wait until a program asks
for it.

### What is not proposed

- A built-in confidence on evidence, caveats or claims, or a grade the
  runtime combines.
- Graded `supports` or `opposes`. A stance stays a direction; the grade is
  the reading's value.
- A decision that is "stronger" or "weaker" by a number. It is in force or
  reopened, by the program's policy.
- Holding a full belief distribution in Caveat. The robot's filter, the risk
  model and the guardrail keep their own state; Caveat records the number a
  decision used and what was known to be wrong with it.

## Decisions

The owner chose on 2026-10-25:

1. **P1, a caveat on one reading:** yes, for the first 1.x minor after
   1.4.0, as its own specification PR with a corpus sweep.
2. **P2, named thresholds in the record:** yes, in the same minor as P1,
   also as its own specification PR.
3. **P3** waits for a program that needs it.

Both stay opt-in: a program that uses neither loads, runs, saves and
explains byte for byte as before. The renewable-evidence gap is already
fixed for 1.4.0 (D024, above). P1 is
[Reading Caveats 0.1](../../spec/caveat-reading-caveats-0.1.md) and P2 is
[Thresholds 0.1](../../spec/caveat-thresholds-0.1.md). P2 lists the
thresholds in a commitment's record only; states, bindings and readings keep
the number, as before.
