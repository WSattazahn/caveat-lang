# CAVEAT Identifiers 0.1

An event parameter was a number, a name from a list fixed in the source, or an
entity the source declares. None of these can carry a commit SHA, a pull
request number or a file path: things a program first hears about while it
runs. The [agent ledger](../experiments/agent-ledger/README.md) had to read a
commit as the first eight hex digits of its SHA, so a decision read
`= 1613487084` rather than `602bdbec…`.

```caveat
identifiers limit 4096;
state head = 0;
event pushed commit id;
event checks commit id, result in failed passed;
on pushed set head = commit;
on checks when commit != head reject "Checks for a commit that is not the head.";
bind pr.head = id_text(head);
```

## Syntax

```text
identifiers limit N;       -- once per program, N in 1..65536
NAME id                    -- an event parameter
id_text(EXPRESSION)        -- a text function
```

A program with an `id` parameter must declare `identifiers`, at most once.
`id_text` is a built-in function, like `text`.

## Meaning

**What an identifier is.** An identifier is a text a host sends for an `id`
parameter. Two identifiers are the same only when their texts are the same
sequence of bytes. Nothing is trimmed, case-folded, normalized or shortened:
`602bdbec` and `602BDBEC` are different identifiers, and so are `a/b` and
`a//b`.

**What a program sees.** Inside the program an identifier is a number, its
**handle**:
- the session gives out 1, 2, 3, … in the order it first receives each
  identifier;
- the same text always gets the same handle in that session;
- within one event, new identifiers are numbered in the order the event
  declares its parameters.

So everything that works on numbers works on handles:
- equality (`commit != head`);
- keeping one in state (`set head = commit`);
- recording one as a reading (`sample seen = commit supports …`).

A handle is local to its session. Handle 7 in one session need not be the
same text as handle 7 in another. Hosts use texts, never handles.

**This is not a type system.** A handle is an ordinary number, so arithmetic
on it is allowed and meaningless. `id_text` can check that a number is the
handle of an existing identifier. It cannot know how the number was produced:
`head + 1` may be another identifier's handle.

**What an identifier is not.** Receiving an identifier observes nothing. A
handle carries no evidence and no caveats, like any event parameter, so a
SHA is not trusted because it was sent. Identity is not observation:
- two readings about the same commit share its handle;
- they remain two readings, each with its own occurrence and grounds.

**`id_text(x)`** is the text of the identifier whose handle is `x`, and keeps
`x`'s evidence and caveats, as other text functions do. `id_text(0)` is the
empty text, so a state that starts at 0 reads as "no identifier yet". Any
other value that is not a handle is an evaluation error. Under
[Dispatch outcomes 0.1](caveat-dispatch-0.1.md) that makes the event fatal
`unclassified`, as `1 / 0` is.

## Payloads

A JSON payload sends an `id` parameter as a string. A string of 1 to 1,024
bytes is accepted. An empty or longer string, a number, or any other JSON
value is refused as `input/payload_invalid`. A host cannot know a session's
handles, so it may not send one. The native Rust `apply`, which takes
numbers for every parameter, accepts only the handle of an existing
identifier for an `id` parameter.

## Limits

A session holds at most `N` identifiers, totalling at most 1 MiB of text. An
event that would add an identifier past either bound is refused as
`limit/identifier_limit`, and the session continues. An identifier the session
already holds is still accepted at the limit. Nothing is truncated.

## Transactions

A new identifier is part of the event that brought it. If the event is
refused or fails for any reason, it adds no identifier, so no later handle
changes. This includes a `reject` in a later rule, a bound, a history limit,
an identifier limit and a fatal error.

## Snapshot and save

The snapshot lists `identifiers`, the texts in handle order: the entry at
index 0 has handle 1. An `id` parameter's `domain` is
`{"identifier": {"limit": N}}`.

A save holds the same list as `identifiers`, and leaves it out when it is
empty. A save made before this profile has no such list, so it restores with
none. Restoring refuses a list that:
- is not an array of strings;
- repeats a text;
- holds an empty or over-long text;
- exceeds either bound;
- appears in a program that declares no `identifiers`.

The same event history always produces the same list.

## Not in this profile

This profile does not include:
- declarations created per identifier, such as a decision series for each
  pull request;
- withdrawing an observation;
- permission;
- superseding earlier readings.
The [agent ledger](../experiments/agent-ledger/README.md) describes each.
