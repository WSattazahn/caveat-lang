# Token cost

What Caveat costs an agent in tokens: the findings are in
[docs/TOKEN_COST.md](../../docs/TOKEN_COST.md).

| File | What it holds |
| --- | --- |
| `measure.mjs` | Counts every number in the findings; the command to run it is in its header and in the findings |
| `results.json` | Its output for main at `9185f46` with the caveat-lang 1.4.1 runtime |
| `growth.cav` | The probe program for how `explain` text grows with grounds and revisions |
