---
name: Break the withdrawal claim
about: A case where Caveat reopens too much, too little, or changes the past after a withdrawal
title: "[Break report] "
labels: break-report
---

<!--
Thank you for trying to break it. The claim and its five rules are at
https://wsattazahn.github.io/caveat-lang/break.html.

Please read "What is outside the claim" there first. If you think one of the
rules itself is wrong, that's welcome too: choose "I disagree with a rule"
below and show the case where it causes harm.

Use synthetic examples only: no credentials, private data or private source.
-->

## Which rule

- [ ] R1 Nothing recorded changes
- [ ] R2 Everything computed from it is flagged
- [ ] R3 "Rests on" means grounds, exactly
- [ ] R4 Nothing reopens by itself
- [ ] R5 A refused event withdraws nothing

## What kind of report

- [ ] Reopens too much (flagged or reopened something that didn't rest on the evidence)
- [ ] Reopens too little (missed something that did)
- [ ] Changed the past (a recorded basis, observation, archive or journal entry changed)
- [ ] A refused event left a withdrawal behind
- [ ] I disagree with a rule as written

## Version

- `caveat-lang` version (`npx --no-install caveat-lang --version`), or the
  version the page showed:
- Where you ran it (the page, CLI, Node library, `caveat serve`, Python):

## The program

```caveat
(the smallest .cav program that shows it)
```

## The events

```jsonl
(one event per line)
```

## What you expected, and what happened

- Expected (which decision or value, and why you think it rests or doesn't
  rest on the withdrawn evidence):
- Actual (paste `npx --no-install caveat-lang explain PROGRAM EVENTS`, or the
  checker's red line):

## Anything else

Related issues, what you already tried, or why the rule causes harm in a real
task.
