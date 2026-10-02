# Separate invalid-citation exercise

Installed version: 0.1.0-rc.7. This exercise is separate from first.cav and tracker.cav.

Minimal invalid program (diagnosis-invalid.cav):

```caveat
claim ready;
event consult;
bind result.value = 1 because ready;
```

Exact installed diagnostic from validate, exit 2 ([receipt](receipts/015.stderr.txt)):

```text
diagnosis-invalid.cav does not load: binding result.value because: ready is a declared claim, not a value citation; cite a state, a history read such as latest(STREAM), or an evidence-bearing expression that the value or its conditions actually read
```

Corrected program (diagnosis-corrected.cav):

```caveat
evidence tool from "the supplied numeric tool value";
state value_used = 0;
event observe value min 0 max 100;
on observe reveal tool;
on observe set value_used = qualified(value, tool);
bind result.value = value_used when observed(tool) because value_used;
```

The binding cites value_used, which it actually reads. That state receives qualified(value, tool); tool is neutrally observed, so it honestly identifies the source of the supplied number without asserting a claim. Validation exits 0 (receipt 016). integration-test.mjs dispatches value 73, verifies the binding equals 73 and cites exactly tool, and saves diagnosis-snapshot.json (receipt 017).

Earlier exercise failures are retained: receipt 012 found that the first tiny draft lacked an event; receipt 013 rejected a direct declared-evidence citation. Adding an event exposes the requested claim diagnostic; citing the actual state fixes the value citation. No tracker files were changed by this exercise.
