# Decisions that rest on several models' votes

Several independent models vote on each case, and a case is approved when at
least two of the three agree. The idea comes from Jeff Hawkins' "thousand
brains" account of perception, in which many small models vote and the answer
is their consensus. This experiment asks what an agent's record should say
when its decisions are made that way and one of the models later turns out to
be unreliable.

[`votes.cav`](votes.cav) answers with the multi-source pattern and
`withdraw every` from [Untrusted Sources 0.1](../../spec/caveat-untrusted-sources-0.1.md),
the 1.4 draft:

- Each decision is grounded on the votes that agreed, and only on those. A
  dissenting vote is not in the grounds. `explain` lists it under "could also
  have been influenced by", because the rule that chose the agreeing set read
  it.
- `distrust` withdraws every vote the model cast, on every case, in one step.
- A decision that the withdrawn vote tipped, one that falls below two agreeing
  votes without it, is reopened. A decision that still has two agreeing votes
  holds, and its record still says it rests on a withdrawn vote.
- Distrusting a model that dissented changes no decision.

```sh
npm run build
node kit/bin/caveat.mjs test experiments/model-votes/votes.scenarios.json
node kit/bin/caveat.mjs undo-list experiments/model-votes/votes.cav experiments/model-votes/demo.events.jsonl
```

With [`demo.events.jsonl`](demo.events.jsonl), case A is approved by m1 and
m2 over m3's dissent, and case B by all three. After m2 is distrusted, the
undo list reopens case A and keeps case B in force, each with the withdrawn
vote named. `kit/test/model-votes.test.mjs` runs the scenarios and checks that
undo list.

## What the program has to work around

- The grounds of a decision come from its `using` expression, and a value
  computed with `if(...)` is grounded on everything its condition read. So
  `using if(m1_yes, latest(m1_vote), 0) + ...` would ground the decision on
  the dissenting votes too. The program writes one rule per set of agreeing
  models instead, because a rule's condition does not ground the decision.
- That takes one rule for each quorum of agreeing models: four for two of
  three, and sixteen for three of five. The language has no way yet to ground a
  decision on "the members of a kind that satisfy a condition".
