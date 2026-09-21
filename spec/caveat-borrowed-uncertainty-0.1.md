# CAVEAT Borrowed Uncertainty 0.1

[caveat-0.5](caveat-0.5-draft.md) let a program retain a caveat another part
wrote:

```caveat
use weather;

caveat own_doubt consequence low;
commit enter_channel because enough retaining weather::fog_refraction, own_doubt;
```

Both are unresolved qualifications carried by one decision, and the language
treated them identically. But they are not the same act. Committing despite a
limitation you wrote is different from committing despite one someone else
wrote and you may never have read.

[CAVEAT_ESSENCE.md](../docs/CAVEAT_ESSENCE.md) says a commitment "selects an
action without asserting that its assumptions are certainly true". Once
assumptions can come from another part, *whose* assumption it was becomes a
question the record should be able to answer.

## 1. What is recorded

A commitment's retained caveats now carry who wrote each one:

```json
{
  "action": "enter_channel",
  "retained": ["weather__fog_refraction", "own_doubt"],
  "retained_authorship": [
    { "caveat": "weather__fog_refraction", "written_by": "weather", "written_elsewhere": true },
    { "caveat": "own_doubt", "written_by": "crossing", "written_elsewhere": false }
  ]
}
```

`retained` is unchanged — a plain list of names — so readers that only count
caveats or look them up keep working. `retained_authorship` is parallel to it
and adds the attribution [caveat-authorship-0.1](caveat-authorship-0.1.md)
made available.

Nothing new is computed. The commitment is a node with an origin, each
retained caveat is a node with an origin, and this reports a comparison the
graph could already make. That is the whole mechanism.

## 2. `written_elsewhere` is a location, not a verdict

The field says where text was written. It does **not** say that an inherited
caveat is weaker, more likely to be stale, less binding, or less worth
examining. A caveat written in another part has the same consequence, the same
attention state, and the same claim on a decision as one written here. A test
asserts that the same program with the same caveats commits identically
whichever file they live in.

This profile deliberately does not name the field `borrowed`, `foreign`, or
`external`. Those read as judgements, and a reader who saw
`"borrowed": true` beside an unexamined caveat would be invited to discount it.
The graph is not a confidence score, and this must not become the first place
one creeps in.

What an author may reasonably do with the distinction is decide where to spend
attention — examining a qualification nobody in this file wrote is often the
useful thing. That is a choice the author makes, not one the runtime makes for
them.

## 3. Unknown stays unknown

`written_elsewhere` is `null` when either origin is unrecorded — when a
single-file program declared no `origin`, or when a caveat was declared before
one.

It does not collapse to `false`. `false` would claim the committer wrote a
caveat that nobody attributed, which is a confident answer to a question that
was never asked. This is the same refusal
[caveat-authorship-0.1](caveat-authorship-0.1.md) section 3 makes about
defaulting an origin, applied one level up: half an answer is not an answer.

A partly attributed program therefore reports a mix — `false` for what it knows
was written here, `null` for what it cannot say.

## 4. What this does not add

There is no rule that a commitment must examine what it borrows, no budget
consequence, and no new failure. Retaining a caveat from another part is
ordinary and allowed; the record simply now says that is what happened.

Attributing a caveat that a running effect revealed is still out of reach, for
the reason [caveat-authorship-0.1](caveat-authorship-0.1.md) section 3 gives:
declarations are over by then, and the honest answer is that the part is
unknown.
