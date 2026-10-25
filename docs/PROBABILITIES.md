# Caveat alongside probabilities

Caveat tracks what a value rests on. It does not say how likely the value is.
Several readers have asked how the two fit together. This note answers with
what works today, without changing what any program means. It is a design
note, not a specification.

## What Caveat already does with a number

A score from another tool is a number like any other: a classifier's
probability, a strength and confidence from Probabilistic Logic Networks
([related work](RELATED_WORK.md#probabilistic-logic-networks-in-opencog-hyperon)),
or a posterior from a graphical model. Caveat can carry it today:

- **As an event payload.** A host sends the score as a bounded parameter, for
  example `event score p min 0 max 1;`.
- **As evidence, with its own caveats.** Sampled into a reading stream, the
  score becomes a reading: an observation of evidence with a name, an order
  and a source. Caveats declared on that evidence, such as "the model is not
  calibrated for this population", travel with every value computed from it.
- **In a decision's grounds.** A decision made on the score keeps the reading
  it was made on, and that record does not change later.
- **Withdrawable.** If the score turns out to be wrong, a program withdraws
  that reading. Everything that rests on it is marked, and a decision made on
  it can reopen through an authored policy.

For example (checked on 2026-10-25 with the runtime on main;
`caveat-lang explain` shows the grounds, the caveats and the withdrawal):

```caveat
claim fraud;
evidence model_score from "the fraud model's score for this transaction";
evidence analyst from "an analyst reviewed the transaction";
caveat uncalibrated consequence material;
uncalibrated qualifies model_score;
readings scores from model_score limit 8;
decisions hold limit 4;
state confidence = 0 min 0 max 1;
event score p min 0 max 1;
event miscalibrated;
on score sample scores = p supports fraud;
on score set confidence = latest(scores);
on score when latest(scores) >= 0.9 and not committed(hold)
    commit hold because enough using latest(scores);
on miscalibrated reveal analyst supports fraud;
on miscalibrated withdraw latest(scores) because analyst;
on miscalibrated when committed(hold) and not reopened(hold) and rests_on_withdrawn(hold)
    reopen hold because analyst;
bind panel.confidence = confidence;
```

After scores of 0.4 and then 0.95 and a `miscalibrated` event, the decision
`hold@1` is based on the second reading, carries `uncalibrated`, and shows
that reading as withdrawn and the decision as reopened. The 0.95 is still
there: withdrawal removes nothing.

So the two answer different questions. The score says how likely fraud is.
Caveat says which score a decision used, what was known to be wrong with it
at the time, and what happened when the score was taken back.

## Where the number and the reasons meet

- **The program states the threshold.** `latest(scores) >= 0.9` is ordinary
  arithmetic in the program's own rule. Caveat records that the decision read
  the score, not why 0.9 is the right threshold.
- **Lineage is not a probability.** A value's lineage lists every piece of
  evidence its evaluation read, including evidence a guard only looked at. It
  answers "what could this have depended on", which is not "how much".
- **Combining scores is the program's job.** If a program averages two
  models' scores, the result rests on both readings, and Caveat records that.
  The averaging is whatever arithmetic the program wrote.
- **Caveat consequences are not likelihoods.** A caveat's consequence is one
  of five declared levels, from `negligible` to `catastrophic`. It says how
  much the caveat matters, not how likely it is to apply.

## Out of scope

None of the following is planned. Each would change what the language
means, so each would need its own specification and the owner's decision:

- Caveat computing probabilities itself, or propagating a score through
  lineage (as provenance semirings do; see
  [related work](RELATED_WORK.md#datalog-provenance-provenance-semirings)).
- A built-in confidence on evidence or caveats that the runtime combines.
- Lowering a score when evidence is withdrawn. Withdrawal marks what rests on
  the evidence; a program that wants a lower score computes it.
- Treating a decision as stronger or weaker by a number. A decision is in
  force or reopened, by the program's policy.

## What a host can do now

1. Send each score as a reading of named evidence, not as a bare state, so
   decisions keep which score they used.
2. Declare what is known to be wrong with the scoring tool as caveats on that
   evidence, so every value computed from it carries them.
3. Write the threshold in the rule, so a reviewer sees it beside the grounds.
4. Withdraw a score that was wrong, and write the reopening policy for the
   decisions made on it.
