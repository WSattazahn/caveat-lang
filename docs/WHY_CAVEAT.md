# Why Caveat

Most programs know things without knowing *how* they know them. A label says
"probably safe", and nothing in the program can tell you which observations
made it say so, whether those observations still hold, or what the program
believed yesterday when it made a decision. When someone asks "why?", a
developer adds bookkeeping: an array of reasons, a flag for doubt, a copy of
the state at decision time. The bookkeeping drifts from the logic it
describes, and nothing checks it.

Caveat makes that knowledge part of the language. The examples below come from
the [glowcap benchmark](../experiments/glowcap/RESULTS.md), where the same game
beat was built in TypeScript and in Caveat and changed eight times. The
TypeScript is the benchmark's own code, not a straw man.

## 1. Explanations that can't lie

A mushroom that has not been tasted gets a label from the slime's belief, and
the label should cite the evidence behind it.

**TypeScript.** The reasons are a list you keep in step with the logic by hand:

```ts
const guess: Record<BeliefState, [string, string[]]> = {
  none: ['Glowing mushroom', []],
  probably_safe: ['Probably a glowcap', supportedBy],
  probably_unsafe: ['Probably a duskcap', contradictedBy],
  uncertain: ['Could be a duskcap — taste first', contradictedBy],
};
```

Cite the wrong list and nothing notices.

**Caveat.** The label says what it rests on, and the runtime checks it:

```caveat
bind $m.label = "Could be a duskcap — taste first"
    when $m_unknown and uncertain
    because contradiction;
```

`because contradiction` cites the evidence that `contradiction` is *grounded*
on. If a label cites anything its value and conditions never read, the event
is rejected: `binding ruin.label cites evidence taste_cave that its value and
conditions never read`. An explanation may leave things out; it can never
make things up.

## 2. Knowledge that arrives late

A taste fades from memory after a minute. From then on, everything built on
that taste should carry the caveat "the taste has faded": the mushroom's
label, the belief, and the recovery count that may restore trust. A decision
already made on the taste must *not* change. It was made on what was known at
the time.

**TypeScript** keeps a timestamp per taste, looks caveats up lazily, and
freezes a decision's caveats when it is made:

```ts
const tastedAt = new Map<string, number>();
function faded(evidence: string): boolean {
  const at = tastedAt.get(evidence);
  return at !== undefined && now - at >= TASTE_FADES_AFTER;
}
function caveatsFor(evidence: string[]): string[] {
  return [...new Set(evidence.flatMap((id) => [...(caveatsOf.get(id) ?? []), ...(faded(id) ? ['taste_faded'] : [])]))];
}
// ... and at commit time:
trust = { basis, reopenedBy: [], caveats: caveatsFor(basis) };
```

**Caveat** states the fact once:

```caveat
on tick when $m_tasted_at >= 0 and now - $m_tasted_at >= 60
    qualify taste_$m with taste_faded;
```

The runtime already knows which values depend on `taste_ruin`, because every
value carries its evidence. It adds the caveat to all of them at once. Frozen
decision records are left as they were, because that is what a record is.

## 3. Decisions that remember

Trust is committed after a good mushroom, reopened by a bad one, and
committed again after two good observations. The journal should say why each
time.

**TypeScript** writes history as it goes, alongside the logic:

```ts
if ((!trust && beliefState() === 'probably_safe') || recovered) {
  const basis = recovered ? [...sinceContradiction] : [evidence];
  trust = { basis, reopenedBy: [], caveats: caveatsFor(basis) };
  history.push({ change: 'committed', because: [...basis] });
}
```

**Caveat** declares the decision and how it is revised:

```caveat
decisions trust limit 16;
on absorb when $m_here and sort == sort.glowcap and reopened(trust) and recovery >= 2
    commit trust because enough using recovery;
```

Every revision is grounded on exactly what it used: `trust@2` rests on the
two recovering observations, not on `trust@1`. The runtime publishes a
`decision_journal`: every commitment and reopening in order, each with its
evidence in the order it was observed and its caveats at that moment. The
host reads it; nobody writes it.

## The Caveatist way

1. **Say what you know, and how you know it.** Observations are evidence.
   Derived values carry it.
2. **Never let an explanation lie.** Cite what a value rests on, and let the
   runtime check the citation.
3. **Control is not content.** A guard decides *when* a rule runs; it is not
   what the value is *about*. Caveat keeps both: lineage for audit, grounds
   for meaning.
4. **Decisions are records, not variables.** Reopen them, revise them, and
   keep the old ones.
5. **Late knowledge qualifies; it does not rewrite history.** What you learn
   now reaches everything current, and nothing already decided.
6. **Not allowed means nothing happened.** A rejected event leaves no partial
   trace.

## When not to use Caveat (yet)

- **Hot inner loops.** An event costs about 100 µs in the WebAssembly runtime
  against about 1 µs for plain TypeScript. That is comfortable at event
  boundaries and at a 16 Hz tick. It is not for per-pixel work.
- **Tight download budgets.** The lean reactive runtime is about 340 KB
  gzipped.
- **Programs with nothing to explain.** If no one will ever ask "why?", plain
  code is simpler.

## Try it

- Play the [glowcap explainer](https://wsattazahn.github.io/caveat-lang/glowcap.html),
  or build it locally with `npm run build && npm run serve` and open
  `/glowcap.html`.
- Read [`game/glowcap.cav`](../game/glowcap.cav): 136 lines, the whole
  beat.
- Start writing with the [authoring guide](AI_AUTHORING.md) and the specs
  linked from the [README](../README.md).
