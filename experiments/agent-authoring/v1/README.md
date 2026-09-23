# Fresh-agent authoring study

Six independent author contexts, two per synthetic task, use a fixed Caveat
documentation packet and runtime. The study measures exact first-submission
and final-submission correctness under a fixed revision/check budget. It is
not a comparison with another language or model family.

- [Registered protocol](PROTOCOL.md), commit `fba9c78`.
- [Results and limitations](RESULTS.md): 2/6 first-source and 6/6 final-source
  corpus passes; two final sources have known clock-horizon contract violations.
- [Frozen file hashes and case counts](registration.json).
- [Task contracts](tasks/), [exact author prompts](prompts/), and
  [public tool instructions](packet/README.md).
- [Independent oracle](private/oracles.mjs), [scorer](verify.mjs),
  [scorer corrections](VERIFIER_CORRECTIONS.md), and [corpus coverage](corpus.json).
- Every first source, revision, self-check input/output, final source and author
  note is retained in `runs/A1` through `runs/C2`. `runs/SMOKE` is a root-authored
  tool test and does not count as a candidate.

## Reproduce verification

Use Node 20 or later and the runtime from `04f72dc` (runtime commit `c5c0183`).
The study branch contains that runtime. From the repository root:

```sh
npm ci
npm run build
node experiments/agent-authoring/v1/prepare-packet.mjs
node --test experiments/agent-authoring/v1/checks.test.mjs
node experiments/agent-authoring/v1/audit.mjs
node experiments/agent-authoring/v1/verify.mjs
node experiments/agent-authoring/v1/session-metadata.mjs
```

Preparation checks the generated runtime and documentation against the frozen
manifest. A later language change may require rebuilding the pinned revision;
do not overwrite the manifest to present that later runtime as the original.
The scorer refuses to run until all six final submissions are frozen. Candidate
failures are study outcomes, not verifier crashes, so a completed scoring run
can exit successfully while reporting failed candidates.

The integrity audit checks preserved source hashes, budgets, capture before
execution, pinned references/runtime, and the documented scorer correction.
The run directory disables Git text conversion so original line endings and
source hashes survive commit and checkout; preserved formatting is not cleaned up.
It cannot prove what files an agent read. Isolation is by fresh context,
instructions and author disclosures; the filesystem is shared.
