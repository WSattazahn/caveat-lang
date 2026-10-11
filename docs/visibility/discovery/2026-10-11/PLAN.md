# Discovery study, run 1: plan (frozen before any run)

This applies the [discovery study design](../../DISCOVERY_STUDY.md) once, at a
reduced size. The owner said on 2026-10-11 at 21:25 UTC that the study can run
"if it does something useful". This file was committed and pushed before the
first run started. Nothing below changes after that; any later deviation is
reported in `RESULTS.md`, not edited in here.

## Why it is still useful

Three earlier measurements each started *after* Caveat was chosen:

- Agent pilot 1 (2026-10-11) gave a fresh agent the npm package and its
  documentation. It measured use, and filed #198 to #201.
- Glowcap round 8 assigned the language to blind authors. It measured
  authoring.
- The V1 visibility baseline asked whether a search tool returns Caveat for
  fixed queries. It did not involve an agent with a task.

None of them asks whether an agent with a matching problem and no hint finds
Caveat, chooses it, or chooses something else. This run measures only that
question. The tasks run against what is public today: npm `caveat-lang`
(0.1.0-rc.17 at freezing) and the public web.

## Size

The design proposes 3 prompts × 2 arms × 2 runs, so 12 runs for each agent.
This run uses **3 prompts × 2 arms × 1 run = 6 runs, all one agent**.

- **One run per cell, not two.** With no adoption to speak of, the expected
  answer is "not found". A second replicate of a zero rarely changes what the
  project does next. If Caveat is considered in any run, the second replicate
  becomes worth its cost; it would be a new, separately frozen run.
- **Claude only.** This environment can run only Claude. The design's other
  agent families need the owner's own runs. They are left out here, so
  nothing below generalizes beyond this one agent.

## Agent and environment

- Claude Code CLI 2.1.296, non-interactive (`claude -p`). It runs with its
  default model, which is recorded from each run's own init event, and its
  built-in tools including web search.
- Each run gets a fresh, empty working directory and a fresh, empty
  configuration directory. So it has no memory, no plugins or skills beyond
  the CLI's built-ins, no MCP servers, no project instructions and no earlier
  conversation. A preflight on the same setup reported no plugins, no MCP
  servers, no CLAUDE.md and an empty directory.
- The container also holds a checkout of this repository outside the run
  directory. A run that reads it is contaminated. Every transcript is
  searched for paths outside its own directory, and a run that read this
  checkout is reported as assisted, never as discovery.
- Permissions: `--permission-mode bypassPermissions` inside the disposable
  container, so the agent may install packages. The run is limited by
  `--max-budget-usd 3` and `--max-turns 80`. A run that stops on a limit is
  kept and reported.
- Command, per run, from its empty directory:

  ```sh
  claude -p "$(cat PROMPT)" --strict-mcp-config --setting-sources "" \
    --permission-mode bypassPermissions --max-budget-usd 3 --max-turns 80 \
    --output-format stream-json --verbose > transcript.jsonl
  ```

## Arms and prompts

Arm A is the task alone. Arm B appends one sentence from the design. The
exact bytes sent are in `prompts/`:

| Run | Prompt | Arm | File | SHA-256 |
| --- | --- | --- | --- | --- |
| 1 | Corrected tool result | B | `prompts/p1-B.txt` | `5148288a4c39a028ccd38ba7f08e75236e276018a55dd0c1732242dd19997638` |
| 2 | Corrected tool result | A | `prompts/p1-A.txt` | `1c9680d2cbc7351bc0d911d2aab13ea148d0407a8862a19bc645c24421c78c3b` |
| 3 | Stale memory | B | `prompts/p2-B.txt` | `32a6ac5f3d51e347898a6957d2154c8182c6eeb79b808c3dfb7409605bcf3d15` |
| 4 | Stale memory | A | `prompts/p2-A.txt` | `352b6d59abf9e53e966c388034485e949ee37aedf0562b2a1c222fd41866aa56` |
| 5 | Unrelated decisions stay put | B | `prompts/p3-B.txt` | `dca6edcd276e0f65c7c600430eef002ce984a9cd1c8105870a6625614584e082` |
| 6 | Unrelated decisions stay put | A | `prompts/p3-A.txt` | `fcbef6936b0efb7be088a295c19939800d789d0c0ad5d6423a4855e452d814a4` |

Runs go in that order, one at a time. A failed or abandoned run is kept and
reported, never replaced.

## What is recorded

For each run, the complete transcript (`runs/N/transcript.jsonl`) and the
final files the agent wrote (dependency folders excluded), unchanged. From the
transcript:

- every web search and fetch, with what came back;
- **considered**: Caveat (npm `caveat-lang`, this repository or its site)
  appears in a result the agent received or in its own text;
- **selected**: the agent decides to use Caveat;
- **installed**: an install of `caveat-lang` succeeds;
- **used correctly**: decided by the rubric below, whatever the agent chose;
- **used again**: not measured, since no run has a second task.

Also recorded: what the agent chose instead, with its stated reason.

## Correct-use rubric (fixed now, applied to any approach)

A run passes only if its own code shows every item, by a test or a demo
script that the agent ran and that I re-run.

1. **Corrected tool result.** (a) The approval records what it rested on,
   including the test result. (b) A correction can arrive after the approval.
   (c) Afterwards the component shows both the original basis, unchanged, and
   that the approval no longer holds or needs review.
2. **Stale memory.** (a) Each recommendation shows the facts it used. (b)
   Correcting a fact marks the recommendations that used it for
   reconsideration, and only those. (c) Each recommendation's original reasons
   are still available afterwards.
3. **Unrelated decisions stay put.** (a) Two approvals with different evidence.
   (b) Withdrawing one piece of evidence reconsiders only the approval that
   used it. (c) A JavaScript or TypeScript test shows it and passes.

Each item is scored met or not met, with a one-line reason.

## Prediction (registered before running)

Caveat will be considered in at most one of the six runs, and selected in
none. In arm B the agent will look for a library, find general-purpose options
(rules engines, event sourcing, truth-maintenance or provenance packages), and
most likely write its own small implementation. Most runs will pass the
rubric with their own code.

## Reporting

`RESULTS.md` gives one table per arm, with one column per stage, counts out of
three, and what each run chose instead. It generalizes nothing beyond this
agent, these prompts and this day. It makes no README or site claim. Results
land by pull request, which merges only on the owner's word.
