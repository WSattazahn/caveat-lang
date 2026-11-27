# Wording for people who manage agents (draft)

Status: draft wording for review. Nothing here changes a live page yet; each
change below goes in its own PR once this wording is approved.

## The angle

When agents write most of the code, a new language matters less as something
people type. Caveat is not mainly a language for people to type. It is what you
have an agent write when you need to check the agent's reasoning: the program
keeps each claim with its evidence and caveats, and the record it produces can
be checked and replayed. Within 1.x that record keeps the same meaning: minors
only add, keeping output bytes and exit statuses ([compatibility](COMPATIBILITY.md)).

The lead line stays the one from #402: **Caveat keeps an AI's reasons in a
record you can check.**

## Draft wording

### caveat-lang (README, kit README, about page)

The lead line is already in place on main (`README.md`, `kit/README.md`,
`web/about.html`, `web/break.html`). Proposed paragraph to follow it:

> Caveat is meant to be written by agents and read by the people who manage
> them. An agent writes a short Caveat program for a decision; the runtime
> keeps each claim with the evidence and caveats it rests on. When a source
> turns out wrong, the decisions that rested on it reopen, and `caveat explain`
> shows what each one stood on. Agents can call it directly through
> `npx caveat-lang mcp`. [Authoring and checking Caveat programs](AI_AUTHORING.md) is the
> guide for agent authors; [Limits](LIMITS.md) says what it does not do.

### waltsdesigns.com (Caveat apartment demo)

`public/demos/caveat-apartment/index.html` (header line, around L168) still
carries the older angle:

> Helps AI remember **why** it made each choice, so it knows what to rethink
> when a fact turns out wrong.

Proposed replacement:

> Keeps an AI's reasons in a record you can check, so you can see what to
> rethink when a fact turns out wrong.

## What someone managing agents would want to see

1. **Several agents, one record.** Exists today at
   [Three agents, one record](https://wsattazahn.github.io/caveat-lang/agents.html)
   ([`web/agents.html`](../web/agents.html), [`web/agents.cav`](../web/agents.cav)).
   Three named agents (`security`, `perf`, `comms`, using
   [Agents 0.1](../spec/caveat-agents-0.1.md), shipped in 1.4.0) each add claims
   to one Caveat record. One source turns out stale; only the decisions that
   rested on it reopen, and live `caveat explain` and `caveat dependents` show
   what each stood on. Limit stated on the page: Caveat enforces `agent` rules
   against the sender label the host passes without authenticating processes
   ([Limits](LIMITS.md)).
2. **A tool the agent calls itself.** Exists today. `npx caveat-lang mcp`
   exposes validate, check, test, explain, dependents, `caveat_dispatch` (one
   step of a running ledger) and `caveat_find` (search past decisions) in the
   published 1.5.0 ([MCP tools](../kit/docs/MCP.md)).
3. **Evidence that agents can write it.** In the
   [v3](../experiments/agent-authoring/v3/RESULTS.md) and
   [v4](../experiments/agent-authoring/v4/RESULTS.md) authoring trials, fresh
   agents given only the package passed their registered cases (4 of 4, then
   8 of 8 across two models). Those trials do not estimate how often an agent
   succeeds, and each event costs more than plain code
   ([Limits: what it costs](LIMITS.md#what-it-costs)).

## What this wording must not claim

- That Caveat checks evidence is true, authenticates saves or knows who sent
  an event ([Limits](LIMITS.md)).
- That agents using Caveat are more accurate; no published record shows that.
- Anything about a named public figure's views or endorsement.
