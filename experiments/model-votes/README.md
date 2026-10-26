# Decisions that rest on several models' votes

Several independent models vote on each case, and a case is approved when at
least two of the three agree. The idea comes from Jeff Hawkins' "thousand
brains" account of perception, in which many small models vote and the answer
is their consensus. This experiment asks what an agent's record should say
when its decisions are made that way and one of the models later turns out to
be unreliable.

[`votes.cav`](votes.cav) answers with `withdraw every` from
[Untrusted Sources 0.1](../../spec/caveat-untrusted-sources-0.1.md) and
`only_if` from [Only If 0.1](../../spec/caveat-only-if-0.1.md), both 1.4
drafts:

- Each decision is grounded on the votes that agreed, and only on those. Its
  `using` adds one `only_if(MODEL_AGREES, VOTE)` term per model. The condition
  chose the vote without being a ground, so a dissenting vote is in the
  lineage only, and `explain` lists it under "could also have been influenced
  by".
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

## What it needed

The first version of this program, before Only If 0.1, needed one rule per
set of agreeing models: four for two of three, and sixteen for three of five.
That was because a value computed with `if(...)` is grounded on everything its
condition read, so `using if(m1_yes, latest(m1_vote), 0) + ...` grounded the
decision on the dissenting votes too. Only a rule's `when` condition stayed
out of the grounds. `only_if` gives an expression the same choice, which makes
it one term per model.
