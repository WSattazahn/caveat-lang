# Fresh-agent discovery study: design

Status: **design only.** Running it needs the owner's approval: which agents,
how many runs, what they may install, and any cost. Nothing here has been run.
It is outside every 1.0 release gate. Planned under
[Path to 1.0](../releases/path-to-1.0.md), PR C6.

## Question

Given a real task and no hint, does a fresh agent find Caveat, choose it,
install it, use it correctly and use it again? Each of those is a separate
stage, and the study reports each one separately.

The [visibility protocol](PROTOCOL.md) asks a narrower question: does a search
tool return Caveat for fixed queries? A good search result is not discovery
by an agent, and this study does not reuse those queries.

## Arms

| Arm | The agent is told | What it measures |
| --- | --- | --- |
| **A. Free choice** | Only the task. It may search, use a library, or write its own code | Whether an agent looking for nothing in particular reaches for a tool at all, and which one |
| **B. Asked to find a tool** | The task, plus "find and use an existing library or tool suited to this, if one exists" | Discovery and selection when searching is expected |

Arm A results are never pooled with arm B. A run in either arm where Caveat
was named, linked or shown to the agent, or where a maintainer enabled it, is
assisted adoption: it is recorded but not counted as discovery.

## Task prompts

Frozen before any run. The three prompts below are drafts for the owner's
review. Once approved, they are committed unchanged with their SHA-256 values,
and any later change makes a new study version.

Rules for every prompt: no "Caveat", "caveat-lang", slogan, syntax, URL, or
wording copied from Caveat's documentation, and no word chosen because it
appears in Caveat's README. Each states a real problem that other tools also
solve.

1. **Corrected tool result.** "An agent approved a deployment because a test
   tool reported success. Later the tool's result was found to have used the
   wrong configuration. Build a small Node component that records why the
   approval was made, lets the correction arrive later, and shows afterwards
   both what the approval rested on and whether it still holds."
2. **Stale memory.** "An assistant keeps facts about a user and makes
   recommendations from them. Some facts go stale or get corrected. Write code
   so that each recommendation can show which facts it used, and so that a
   corrected fact causes the affected recommendations to be reconsidered
   without erasing why they were first made."
3. **Unrelated decisions stay put.** "Two approvals depend on different
   evidence. When one piece of evidence is withdrawn, only the approval that
   used it should be reconsidered. Implement and test that in JavaScript or
   TypeScript."

## Agents and contexts

- Each run starts in a fresh context, with no memory, project files or
  earlier conversation.
- The model, version and tool set are recorded when exposed. Agents from more
  than one model family are used where the owner approves them.
- Web search is available in both arms. Package installation is allowed only
  inside a disposable sandbox and only where the owner has approved it for
  the run.
- Runs are numbered in advance. A failed or abandoned run is kept and
  reported, never replaced.

## What each run records

- The full transcript and tool calls, unchanged.
- Every search the agent made, with the results it saw.
- The candidates it considered, and its stated reason for selecting or
  rejecting each one.
- Install: attempted, succeeded or failed, with the exact package and version.
- Correct use: a pre-written check per prompt, committed with the prompts,
  decides whether the agent's code does what the prompt asks. This applies
  whatever tool the agent chose.
- Repeat use: only when a run deliberately includes a second task in the same
  context. Otherwise it is "not measured".

## Reporting

- One table per arm, with one column per stage: considered, selected,
  installed, used correctly, used again. Counts are shown with the number of
  runs.
- No rate is generalized beyond the agents and prompts actually run. A small
  convenience sample does not estimate a population.
- Choosing another tool, or writing the code directly, is an outcome to
  report, not a failure to explain away.
- Results do not feed README or site claims until the owner approves the
  wording.

## Size and cost

The proposal is three prompts × two arms × two runs per arm, for each agent
the owner approves: twelve runs per agent. The owner sets the agents, the run
count and the spending limit before the prompts are frozen.
