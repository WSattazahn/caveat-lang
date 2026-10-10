# Engineering readiness pilot

Status: draft for the owner's review. Nothing here has been sent to anyone.
The pilot runs only on a published candidate carrying the integration
starter (planned for rc.17), and only after the owner approves the target and
the message ([Path to 1.0](../releases/path-to-1.0.md), B2 and B3). No
customer, user or result is claimed here.

## The brief (one page)

**The problem.** A release or deploy approval rests on evidence: a test run,
an audit, a sign-off. When that evidence turns out to be wrong, such as a test
that ran under the wrong configuration, most systems either forget why the
approval was given or revoke everything nearby. A team then has to
reconstruct by hand which approvals rested on the bad evidence and which did
not.

**What Caveat offers.** A small program states what an approval rests on and
what should happen when that evidence is qualified. The runtime records each
decision with its grounds. When evidence is qualified, only the policy the
program states can reopen a decision; decisions that do not rest on that
evidence stay in force. The original grounds stay readable after a restart
and after old records move to the host's archive. The
[integration starter](../../kit/docs/STARTER.md) keeps that state in a
directory or in IndexedDB and answers each event with separate facts: handled,
accepted, durable, archived.

**What the pilot is.** One developer or team integrates the starter with one
real approval step of their own, such as a release gate in CI, using the
[readiness example](../../kit/examples/readiness/README.md) as the starting
point. They keep their own systems as the place actions happen; Caveat records
the decision and its reasons.

**What it is not.** Caveat records the evidence it is given. It does not
authenticate that evidence, and the source digest identifies a program's text
without authenticating a save. It does not authorize or take external
actions. It is a candidate release whose interfaces may change before 1.0. It
is not a hosted service.

**What we want to learn.** Whether an outside developer can install and
integrate it correctly, whether the recorded grounds and the authored
reopening are useful for their approval step, whether they use it again, and
whether they would pay for help doing it. Each is measured separately.

## Acceptance criteria

The pilot package (this document and the example) is ready when all of these
hold on the candidate's packed tarball. Each names the check that shows it.

| # | Criterion | Shown by |
| --- | --- | --- |
| P1 | Approval of revision A rests on test result T for that exact revision; a result for another revision, or a failing one, is refused | Scenario P01 |
| P2 | Qualifying T with `wrong_configuration` reopens the approval only through the authored policy, and the reopening names the caveat | Scenarios P02, P04; `run.mjs` step 3 |
| P3 | A caveat with no policy reopens nothing | Scenario P03 |
| P4 | The unrelated decision on U is unchanged | Scenario P02; `run.mjs` step 4 |
| P5 | The original grounds are readable after the reopening, after a restart and through the host archive | Scenario P02 (resume); `run.mjs` step 5 |
| P6 | A refused event changes nothing; an accepted event whose checkpoint write fails reads as not durable until flushed | `run.mjs` refusal and storage-failure steps |
| P7 | Application revision, Caveat source digest, runtime identity and evidence identity are reported as separate things | Example README, "Four identities"; `run.mjs --json` |
| P8 | Every input is illustrative; no lending, clinical or personal data | Review of the example's files |

The pilot run itself (B3) is measured on five separate questions, none of
which is required to pass for 1.0:

1. **Install:** did they install the candidate and run the example?
2. **Correct integration:** does their integration act only on `accepted` and
   `durable` sends and only while `permits` is true?
3. **Useful behavior:** did the recorded grounds or the authored reopening
   change or confirm a decision they cared about?
4. **Repeat use:** did they run it again, on later revisions, without help?
5. **Willingness to pay:** would they pay for the offer below, and how much?

Defects they find are triaged in the open, as issues, with their permission
for anything they share.

## Scoped offer

**Price: the first pilot is free** (owner's decision, 2026-10-10), in exchange
for the measurements above. Whether a team would pay for the same scope is
asked during the pilot, not assumed.

| | |
| --- | --- |
| Scope | One approval step, one program written with the pilot team, the starter integrated into their environment, and one review of the result |
| Out of scope | Hosting, authentication of evidence, actions taken on the team's systems, other approval steps, support after the pilot |
| Duration | Two weeks of elapsed time (hypothesis) |
| Price for the first pilot | Free, in exchange for the measurements above |
| Price question to test | Whether a team would pay a fixed fee for the same scope, and how much; asked, not assumed |
| Data | The team's own data stays with them; anything shared with the project is sanitized and shared only with their permission |

## Before outreach

- rc.17, carrying the starter, is published and its record verified.
- The owner approves the named target and the exact message.
