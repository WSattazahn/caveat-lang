# Procedure argument kinds

These examples use existing Caveat 1.2.0 features. They show how a procedure can
name an evidence declaration or a reading stream without changing their meanings.

## Choose the parameter kind

A symbol parameter takes a bare declared name, not an expression. `e evidence`
takes an evidence name; `s readings` takes a stream name. For example, with
`review` a decision series, `correction_note` evidence and `samples` a stream:

```caveat
proc by_evidence(e evidence) {
  when committed(review) and not reopened(review) reopen review because e;
};
proc by_reading(s readings) {
  when committed(review) and not reopened(review) reopen review because latest(s);
};
```

Call `by_evidence(correction_note)` after observing that evidence. Call
`by_reading(samples)` after sampling that stream; `latest(s)` inside the procedure
names its latest reading. Passing `latest(samples)` to `e evidence` is invalid:
a reading occurrence is not a declared evidence name. Do not substitute the
stream's evidence template, which names different evidence. An untyped parameter
is a numeric value, not a way to pass an evidence handle.

## Run the example

From this directory, using your installed command:

```sh
caveat-lang validate handles.cav
caveat-lang check --strict handles.cav
caveat-lang test handles.scenarios.json
```

The [source](handles.cav) samples before calling the stream procedure and reveals
the correction note before calling the evidence procedure. The
[scenarios](handles.scenarios.json) distinguish the reopening reasons `samples@2`
and `correction_note`, preserve old grounds, and continue across save/restore.
The correction event here requests reconsideration; it does not withdraw a
measurement. Neither procedure automatically reassesses the decision.

See [procedure symbols](../../docs/reference/spec/caveat-procedure-symbols-0.1.md)
for the language contract.
