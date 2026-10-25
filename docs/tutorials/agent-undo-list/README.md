# An agent with an undo list

*A 15-minute, hands-on tutorial for caveat-lang 1.3.0.*

An AI agent looks things up with its tools, then acts: it refunds a customer,
files a claim, sends a credit. Later one of its tools turns out to have been
wrong. Which actions should it take back, and why did it take each one?

Most agents can't answer that, because the reasons lived in a prompt that is
gone. In this tutorial you give a small support agent a Caveat program that
records every action with the evidence it rested on. When a source is
withdrawn, Caveat reopens exactly the actions that rested on it and keeps the
original reasons on record. That list of reopened actions is the agent's undo
list.

You need Node 20 or later and a terminal (on Windows, Git Bash works). You
don't need Rust, Python or an API key.

## 1. Install (1 minute)

In an empty folder:

```sh
npm init -y
npm install caveat-lang@1.3.0
npx --no-install caveat-lang --version
```

Then fetch the five tutorial files from this folder: `support.cav` (the
program), `events.jsonl` and `events-corrected.jsonl` (what the agent
reported), and `agent.mjs` and `support.scenarios.json`, which you'll use
later.

```sh
base=https://raw.githubusercontent.com/WSattazahn/caveat-lang/main/docs/tutorials/agent-undo-list
for f in support.cav events.jsonl events-corrected.jsonl agent.mjs support.scenarios.json; do
  curl -fsSLO "$base/$f"
done
```

## 2. The story

A customer writes: "Where is my package?" The agent has three tools:

| Tool | Returns |
| --- | --- |
| the order database | the order total: 120 |
| the carrier tracking API | days since the last scan: 9, so it looks lost |
| the customer's message | one complaint |

On that evidence the agent takes three actions: a **refund**, a **carrier
claim**, and an **apology credit**. Then a second carrier lookup shows the
package was scanned yesterday. The first carrier reading was wrong.

## 3. Read the program (4 minutes)

Open `support.cav`. Statements end with `;` and comments start with `#`.
The parts that matter for this tutorial are below; the file also has a
`scene` line (a description) and `event` lines that declare what the agent can
send, with bounds such as `days min 0 max 365`.

**Where evidence comes from.** Each tool is a source, and each tool result is
a reading in a stream. Readings are kept, never overwritten: `scan_gap@1` is
the first carrier reading, `scan_gap@2` the second. A reading can `support` or
`oppose` a claim, here `package_lost`.

```caveat
claim package_lost;
evidence carrier from "the carrier tracking API";
evidence orders from "the order database";
evidence customer from "the customer's message";
evidence recheck from "a second carrier lookup";

readings scan_gap from carrier limit 8;     # days since the last carrier scan
readings order_total from orders limit 8;   # what the customer paid
readings complaints from customer limit 8;  # 1 for each "where is my package?"
```

**Each action is a decision.** A decision is a series of revisions that each
keep their grounds.

```caveat
decisions refund limit 4;
decisions carrier_claim limit 4;
decisions apology_credit limit 4;
```

**Rules say when an action is allowed and what it rests on.** The agent sends
an event such as `issue_refund`; the program either refuses it with a reason
or commits the decision. `because enough` gives the kind of reason (the
evidence was enough), and `citing` names the grounds: the refund rests on the
carrier reading *and* the order total, the claim only on the carrier reading,
the credit only on the complaint.

```caveat
on issue_refund when not has_sample(scan_gap) or not has_sample(order_total)
    reject "Look up the order and the tracking first.";
on issue_refund when latest(scan_gap) < 7 reject "The carrier does not show the package as lost.";
on issue_refund when committed(refund) and not reopened(refund) reject "Already refunded.";
on issue_refund commit refund because enough citing latest(scan_gap), latest(order_total);
```

**A correction withdraws, it doesn't erase.** `withdraw` marks the bad reading
wrong and keeps it. The reason after `because` must be evidence the program
has observed, so the first rule `reveal`s the recheck. Nothing is reopened by
itself: a small procedure reopens an action if it `rests_on_withdrawn`
evidence, and the correction calls it once per action.

```caveat
proc reopen_if_withdrawn(d decisions, why evidence) {
    when committed(d) and not reopened(d) and rests_on_withdrawn(d) reopen d because why;
};

on carrier_correction when not observed(recheck) reveal recheck opposes package_lost;
on carrier_correction withdraw latest(scan_gap) because recheck;
on carrier_correction call reopen_if_withdrawn(refund, recheck);
on carrier_correction call reopen_if_withdrawn(carrier_claim, recheck);
on carrier_correction call reopen_if_withdrawn(apology_credit, recheck);
```

The last lines of the file `bind` a displayed value, `undo.refund` and so on,
to `"keep"` or `"undo"`, so a user interface can show it.

Check that it loads:

```sh
npx --no-install caveat-lang check support.cav
```

You should see `support.cav: no warnings.`

## 4. Ask why each action was taken (2 minutes)

`events.jsonl` is what the agent reported, one event per line: two lookups,
the customer's message, then the three actions.

```sh
npx --no-install caveat-lang explain support.cav events.jsonl
```

Find the `refund` entry under **Decisions**:

```text
  refund: 1 of at most 4
    refund@1 = null  in force
      based on order_total@1, scan_gap@1
      order_total@1: observed at #1 order_lookup {"total":120}
      scan_gap@1: observed at #2 carrier_lookup {"days":9}
      #4 issue_refund: committed because order_total@1, scan_gap@1
```

Each action names what it rested on and which tool call supplied it, down to
the payload. `= null` means the decision records no number: `citing` keeps
grounds without a value (`using EXPRESSION` would record one, such as the
refund amount).

## 5. Before anything goes wrong: what rests on the carrier? (1 minute)

You can ask the reverse question at any time. If the carrier API is wrong,
what would you have to take back?

```sh
npx --no-install caveat-lang dependents support.cav carrier events.jsonl
```

Under **Decisions** it lists `carrier_claim@1` and `refund@1`. The apology
credit isn't there, because it never rested on the carrier.

## 6. The carrier was wrong (2 minutes)

`events-corrected.jsonl` is the same history plus one line: the second lookup,
`{"event": "carrier_correction", "payload": {"days": 1}}`.

```sh
npx --no-install caveat-lang explain support.cav events-corrected.jsonl
```

Now the refund reads:

```text
    refund@1 = null  reopened
      based on order_total@1, scan_gap@1
      ...
      scan_gap@1 has since been withdrawn at #7 because recheck
      #4 issue_refund: committed because order_total@1, scan_gap@1
      #7 carrier_correction: reopened because recheck
```

Under **Evidence** you'll also see `recheck`, the second lookup, and
`scan_gap@2 = 1`, the corrected reading the last two correction rules record.

Three things to notice:

1. The refund and the claim are **reopened**: that's the undo list. The
   credit stays **in force**.
2. The original grounds are **unchanged**. The record still says the refund
   was made on `scan_gap@1 = 9`, so you can show it was a reasonable call at
   the time.
3. The wrong reading is still there, marked `withdrawn`, with when (event #7)
   and on what evidence (`recheck`). Nothing was erased.

## 7. Put it in the agent (4 minutes)

`agent.mjs` is a short agent with stand-in tools. Before each action it asks
Caveat; it acts only when Caveat records the decision. Run it:

```sh
node agent.mjs
```

```text
The agent works the ticket:
  issue_refund       refused: Look up the order and the tracking first.
  order_lookup       ok
  carrier_lookup     ok
  customer_message   ok
  issue_refund       ok
  file_claim         ok
  send_credit        ok

A second carrier lookup disagrees: { days: 1 }
  carrier_correction ok
  issue_refund       refused: The carrier does not show the package as lost.

Undo list:
  UNDO refund          taken at #4 because order_total@1, scan_gap@1; reopened at #7 because recheck
  UNDO carrier_claim   taken at #5 because scan_gap@1; reopened at #7 because recheck
  keep apology_credit  taken at #6 because complaints@1
```

The parts that matter:

```js
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { decisionHistory } from 'caveat-lang/session';

// With no argument this loads the runtime that ships in the installed package.
const runtime = await loadRuntimeFromDirectory();
const session = runtime.open(await readFile('support.cav', 'utf8'));

// Ask before acting. A refused event changes nothing.
const result = session.dispatchView('issue_refund', {}, { fields: ['bindings'] });
if (result.outcome === 'accepted') { /* call the payments API */ }

// After a correction: every revision of an action, with why it was made and why it was reopened.
const [first] = decisionHistory(session.view(), 'refund');
first.status;               // 'reopened'
first.committed.because;    // ['order_total@1', 'scan_gap@1']
first.reopened.because;     // ['recheck']
```

Caveat records and reopens; it doesn't reverse the refund for you. Your agent
reads the undo list and calls whatever API takes the action back.

## 8. Lock the behavior in with a test (1 minute)

```sh
npx --no-install caveat-lang test support.scenarios.json
```

```text
PASS S01  A wrong carrier reading reopens only the actions that rested on it, and they keep their grounds  (8 events, 1 rejected, 0 resumes)
PASS S02  The agent cannot act before it has looked, and refused events change nothing  (3 events, 3 rejected, 0 resumes)
2 passed, 0 failed (support.scenarios.json)
```

S01 checks the undo list *and* that the refund's grounds are the same after
the correction as before it. If a later edit to the program breaks either,
this test fails.

## Try it yourself

Solutions for 2 and 3 are in [`solutions/`](solutions/).

1. Make the apology credit also cite the carrier reading
   (`citing latest(complaints), latest(scan_gap)`). Run step 6 again: now it's
   on the undo list too. Also add `or not has_sample(scan_gap)` to the
   credit's first guard: without it, a credit asked for before any carrier
   lookup is refused as a runtime error (`evaluation/expression`) instead of
   with your message. Run step 8 again as well: S01 now fails, because it
   expects the credit to be kept. That's the test doing its job; update its
   expectation.
2. Withdraw the order total instead. You'll need a new evidence declaration
   (`evidence order_recheck from "a second order lookup";`), a new event
   (`event order_correction total min 0 max 100000;`), and rules shaped like
   the carrier correction: reveal `order_recheck` (a bare
   `reveal order_recheck` is fine; a stance is optional), withdraw
   `latest(order_total)` because of it, call `reopen_if_withdrawn` for each
   action, and record the corrected total with `sample order_total = total`.
   Only the refund should reopen.
3. Keep the session across restarts. `session.save({ compact: true })`
   returns a string; write it to a file, and later pass it to
   `runtime.restore(source, save)`. The restored session refuses a second
   refund with "Already refunded." A save belongs to the exact program it was
   made under, so after any edit to `support.cav`, even a comment, an old save
   won't restore. If you'll change the program, also keep the agent's events
   as a JSON-lines file like `events.jsonl` and replay them under the new
   program; the "Revise a program while the record continues" section of
   `node_modules/caveat-lang/docs/reference/docs/AI_AUTHORING.md` shows how.

## What this is and isn't

Caveat keeps each action with what it rested on, refuses actions the evidence
doesn't allow, and reopens actions when their evidence is withdrawn under the
rules you wrote. It doesn't decide whether a tool is right, authenticate a
source, or give your agent permission to act. A source label is a label.

## Next

- `node_modules/caveat-lang/docs/REFERENCE.md`: the language on one page.
- `node_modules/caveat-lang/docs/AGENT_START.md`: a real agent integration,
  including saving the session.
- `node_modules/caveat-lang/examples/agent-evidence/`: the same idea driven
  from Python through `caveat serve`.
