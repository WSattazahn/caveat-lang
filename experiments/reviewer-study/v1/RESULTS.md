# Reviewer study v1: results

Run on 2026-10-12 against `caveat-lang@0.1.0-rc.19` and TypeScript 6.0.3, as
registered in [PROTOCOL.md](PROTOCOL.md) with Amendments 1 and 2. Nothing here
is a claim that Caveat beats TypeScript (`experiments/glowcap/RESULTS.md`
L1338–1339).

**Outcome: a ceiling.** All six reviewers answered every scored item
correctly, in both arms. The study does not separate the arms on accuracy,
so it licenses no README sentence (PROTOCOL.md, "What a result licenses").
The TypeScript reviewers were faster. None of the seven predictions held;
P5 missed its Caveat threshold by 0.1 point.

## Sealed files

Unsealed after phase two froze (commit 811e953) and copied to `sealed/`.
Each SHA-256 matches `registration.json`:

| File | SHA-256 |
| --- | --- |
| `model.mjs` | `f6bf16675b0b67156f13367e6f834c1da1cd1d35d9b0f413ca0417bde62eb8db` |
| `expected.json` | `8f71befce2caba9ed7ef358967983de78070d6636218d7f23319886318c88dac` |
| `keys.json` | `448cf2aba0b7e0f40319b16d048226a5487aac0fb14181447f2286781a7dc305` |
| `references/gate.cav` | `d1f760f2677feb0ee72bd1dde11308f85ae8a4576e9c70c238c384348719771e` |
| `references/gate.ts` | `2a27819ffe0810df0cf8b4add025bada9b71dd10b6b2f4b2ea728d9f4ecafa7e` |

Check: `cd sealed && sha256sum model.mjs expected.json keys.json references/*`.

## Phase one: the programs

Both authors' programs pass all 24 registered files (`node score.mjs cases
sealed/expected.json runs/A-C/cases-output.json`, and the same for A-T). The
Caveat program's record check passed: on H1 to H3, `explain --json` gives
each decision's grounds at the key's lines, the permitting grant as
`grants@1` (line 2), and the reopenings at the key's lines
(`runs/phase1.json`).

The TypeScript author, told a reviewer would come (as both authors were),
wrote read-only `record()` and `explain()` helpers into the program
(`runs/A-T/files/gate.ts` L4, L176, L300–318). Those helpers kept each
observation's caveats with the lines that added and removed them. That
condition is what this study measured, not a TypeScript program without
provenance.

## Phase two: the reviewers

`node score.mjs answers sealed/keys.json runs/R-*/files/ANSWERS.json`
(full output in `scores.json`):

| Run | Q1 core | Q1 caveats | Q1 strict | Q2 | Q2 grant | Q2 fourth | Q3 hits | Q3 false alarms | Receipts: record / code / run |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-C1 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 57 / 6 / 3 |
| R-C2 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 58 / 6 / 2 |
| R-C3 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 63 / 0 / 3 |
| R-T1 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 60 / 6 / 0 |
| R-T2 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 54 / 0 / 12 |
| R-T3 | 6/6 | 6/6 | 6/6 | 12/12 | 3/3 | 3/3 | 6/6 | 0/12 | 66 / 0 / 0 |

No answer was `cannot_determine` and none lacked a receipt. A TypeScript
`record` receipt cites the program's own `record()` or `explain()` output.

Cost, from the transcripts (`runs/phase2.json`):

| Run | Wall clock | Tool calls |
| --- | --- | --- |
| R-C1 | 3 min 50 s | 21 |
| R-C2 | 4 min 4 s | 22 |
| R-C3 | 3 min 26 s | 16 |
| R-T1 | 1 min 24 s | 7 |
| R-T2 | 1 min 38 s | 7 |
| R-T3 | 1 min 29 s | 8 |

**Grounded.** The investigator judged the 24 answers Known property 1 bears
on: Q1(c) and Q1(d) for H2's third decision and H3's second, for every
reviewer. All 24 receipts support their answers (`grounded.json`). The other
372 answers are correct by the key, and their `grounded` stays null.

## Predictions

| # | Result | Held? |
| --- | --- | --- |
| P1 | Q1 core: Caveat 18/18, TypeScript 18/18 (predicted at most 12) | No |
| P2 | Q2: Caveat 36/36, TypeScript 36/36 (predicted at most 30); no grant-probe error in either arm | No |
| P3 | Q3: Caveat 18/18 hits, 0/36 false alarms; TypeScript 18/18 hits (predicted at most 12) | No |
| P4 | Median wall clock: Caveat 3 min 50 s, TypeScript 1 min 29 s, so the Caveat arm took 2.6 times as long (predicted at most half) | No |
| P5 | `record` receipts: Caveat 178/198 (89.9%, predicted at least 90%); TypeScript 180/198 (90.9%, predicted under half) | No |
| P6 | No Caveat reviewer reported the inherited `withdrawn` or `db_stale` under (c) or (d); other four decisions 12/12; TypeScript 18/18 (predicted at most 12) | No |
| P7 | Neither program failed a registered case (`runs/phase1.json`) | No |

## What the runs show

- **The rc.19 display cost the Caveat arm time, not accuracy.** All three
  Caveat reviewers met `retaining: deps_unreviewed, db_stale` on H3's second
  decision (Known property 1), saw that it disagreed with the journal, and
  answered from `explain --json` grounds or `replay`. `db_stale` appears 6,
  10 and 9 times in their notes, against 0, 1 and 1 for the TypeScript
  reviewers. R-C2 wrote an instrumented copy of the program to settle it and
  still could not find where the line came from. This is D009, fixed on main
  by PR #226 (`also carrying, through its lineage:`), which is not in rc.19.
- **The take-back was found from `replay`, as Known property 2 predicted.**
  `explain` does not show it; this is D010 (1.1).
- **Effort follows the record's shape.** The TypeScript program's
  `explain()` printed what the questions ask for: caveats added and removed
  by line, retained by choice, and the decision's lines. The Caveat reviewers
  had to assemble the same facts from `explain`, `explain --json`, `replay`
  and `dependents`.
- **The questions were too easy for this model, or the notes' false
  statements too findable, to separate the arms.** A ceiling in both arms
  says nothing about how either arm compares with a weaker reader or a
  program written without a reviewer in mind.

## Departures

- A-C and R-C3 each wrote one file under `/tmp` (outside their directory).
- `AGENTS.md` reached every agent through harness context (Amendment 2).
  R-C2 reports it did not open it or any file outside its directory.
- The reviewers' tool logs are derived from transcripts. One R-C1 entry
  names a harness path for an oversized output; it holds no account data.

## Candidate findings

Filed the way dogfooding findings are, not here: none new. Every Caveat-arm
friction point is already D009 (fixed in #226, unreleased) or D010 (1.1).
