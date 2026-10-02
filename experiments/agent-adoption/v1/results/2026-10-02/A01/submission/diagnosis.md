# Separate invalid-citation exercise

Version: caveat-lang 0.1.0-rc.7. This exercise is separate from first.cav and the tracker.

## Deliberately invalid program: invalid-because.cav

```caveat
claim ready;
evidence tool from "consulted source";
state score = 0;
event read value min 0 max 100;
on read reveal tool;
on read set score = qualified(value, tool) because ready;
bind result.score = score;
```

Command: node node_modules/caveat-lang/bin/caveat.mjs validate invalid-because.cav
Exit status: 2. Receipt: receipts/016.json.
Exact stderr:

```text
invalid-because.cav does not load: set score because: ready is a declared claim, not a value citation; cite a state, a history read such as latest(STREAM), or an evidence-bearing expression that the value or its conditions actually read
```

The name ready exists, but it is a claim rather than a value citation.

## Self-directed diagnostic correction

The first correction used a bare tool citation. It also failed (exit 2, receipts/017.json):

```text
corrected-because.cav does not load: set score because: tool is a declared evidence, not a value citation; cite a state, a history read such as latest(STREAM), or an evidence-bearing expression that the value or its conditions actually read
```

The final corrected program cites the evidence-bearing expression that the assignment actually reads:

```caveat
claim ready;
evidence tool from "consulted source";
state score = 0;
event read value min 0 max 100;
on read reveal tool;
on read set score = qualified(value, tool) because qualified(value, tool);
bind result.score = score;
```

It neutrally reveals tool, computes qualified(value, tool), and cites that same computation. No support relation is invented. Final validate exits 0 (receipts/018.json). Replay with diagnosis.events.jsonl also exits 0: score is 85, with exact evidence grounds ["tool"].
