# CAVEAT Reject 0.1

An event that is not allowed should fail and keep nothing. Before this
profile, a program could only fail an event by making an expression fail. The
usual way was `set check = require(allowed, 0)`, which needs a state nobody
reads and puts the validation's dependencies into that state's lineage.

```caveat
on absorb when consumed == 1 reject "already absorbed";
on absorb when known == 2 reject "a known duskcap";
```

## Syntax

```text
on EVENT [when CONDITION] reject "MESSAGE";
[when CONDITION] reject "MESSAGE";        -- as a procedure step
```

The message is one quoted text literal. Anything after it, a missing message,
or unquoted words are rejected before execution.

## Meaning

Executing `reject` fails the event with the error `rejected: MESSAGE`. As with
any other runtime failure, nothing the event did is kept:
- values, graph effects and cues are discarded;
- lineage and grounds stay as they were;
- the event does not consume a sequence number.

Rules still run in source order, so a `reject` after a `set` also discards
that `set`. A procedure step can reject its calling event.

A skipped `reject` names no target, so it leaves no dependency behind. A
program can therefore validate on qualified state without changing the
lineage of anything else.

`reject` is reserved.
