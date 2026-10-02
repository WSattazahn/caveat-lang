# Which evidence did the decision use?

Run these examples from the copied `agent-evidence` directory after installing
Caveat, using `python` if your system has no `python3`:

```sh
npx --no-install caveat validate qualification.cav
npx --no-install caveat check qualification.cav
npx --no-install caveat validate grounded_assessment.cav
npx --no-install caveat check grounded_assessment.cav
python3 -B -m unittest -v test_qualification test_grounds
```

These are independently authored examples, not the Muse or Dot programs.

## A template is not a reading

`qualification.cav` gives these dependencies different identities:

```text
memory (evidence template)
  ├─ qualified(confidence, memory) → direct (current state)
  └─ sample freshness → freshness@1 (archived occurrence)
                           └─ latest(freshness) → strategy@1 (frozen grounds)
```

A sample observes `freshness@1`; it does not observe `memory`. This example
explicitly reveals `memory supports memory_available`: the authored claim is
that this source is available, not that consulting it proves it trustworthy.
That stance is part of this example's model; Caveat does not infer one.
`qualify memory with stale` still requires the template to have been observed.

After `remember 85`, `assess`, then `learn_stale`:

| Record | Result |
| --- | --- |
| `memory` | Its evidence node carries `stale`. |
| `direct` | Its current value depends directly on `memory`, so its grounds gain `stale`. |
| `freshness@1` | Its archived value and recorded provenance remain unchanged. Its evidence identity is `freshness@1`, not `memory`. |
| `strategy@1` | Its original grounds remain frozen on `freshness@1`. |
| A later `freshness@2` | Sampling copies the template's `stale` caveat to the new occurrence. |
| A decision using `freshness@2` | Its grounds include that occurrence and `stale`. |

The test checks the archive, current grounds and journal separately, including
save/restore. See [late qualification](../../docs/reference/spec/caveat-late-qualification-0.1.md)
and [reading identity](../../docs/reference/spec/caveat-reactive-0.5.md).

**Reassessment does not resample.** The Q1 sequence is executable in
`test_qualification.py`: after learning stale, `reconsider` and `assess` without
another `remember` create `strategy@2` on the same `freshness@1`. Its grounds
still lack `stale`; the separate `direct` state carries it. Merely advancing
the decision revision does not redirect its evidence dependency to the template.
A later sample becomes `freshness@2`, and the test's third decision carries
`stale`. If an application needs a new memory lookup, it must require that
lookup and verify its exact occurrence, or explicitly base its policy on the
current template-dependent value. Do not reinterpret an archived reading to
obtain the desired policy.

## Withdrawal requires an application policy

Withdrawal preserves the archive and frozen decision grounds. A withdrawal
record marks which occurrence is no longer stood behind; later reads of that
occurrence acquire `withdrawn`. The archive itself is not rewritten, and a
new assessment is not automatically refused by the language.

The original `assessment.cav` rejects a withdrawn latest observation.
`grounded_assessment.cav` applies the same rule to clarity. Its Q2 test observes,
assesses, withdraws clarity, reopens and tries assessing without replacement.
The event is refused, the save is unchanged, and the old decision retains its
original grounds. Fresh replacement evidence permits a later attempt.

## Check the application's actual grounds

`Attempt` confirms that required operations completed. It cannot know which
sources a custom decision was meant to use. `grounded_assessment.cav` and
`grounds.py` demonstrate one explicit policy for a single task:

- All three sources (clarity, memory and tool evidence) must be present.
- The caller supplies their exact intended occurrence IDs from accepted
  observation results, and the decision's grounds must match them as a set.
- Clarity and tools must have been observed during this attempt.
- Memory may carry over only when the caller explicitly permits that exact
  memory occurrence with `carry_memory`. There is no universal freshness rule.
- A withdrawn or superseded selected occurrence is refused.
- The approval must still be in force and committed during this attempt.

Save this Python block beside `caller.py` and run it. It records each required
event and captures its current occurrence before assessing; the tests execute
this exact block:

<!-- grounding-example -->
```python
from pathlib import Path
from caller import Attempt, CaveatServer
from grounds import grounded_permits

with CaveatServer(str(Path(__file__).with_name("grounded_assessment.cav"))) as server:
    attempt = Attempt(server)
    expected = {}
    for stream, event in [("clarity", "observe_clarity"),
                          ("freshness", "observe_memory"),
                          ("tools", "observe_tool")]:
        outcome = attempt.require(event, {"confidence": 85})
        if outcome is None or not outcome.accepted:
            break
        expected[stream] = server.snapshot()["reading_streams"][stream]["current"]
    attempt.require("assess")
    result = attempt.finish(lambda snapshot, active:
        grounded_permits(snapshot, active, expected))
    assert result.succeeded
```

Use one attempt at a time on this task's server, as the starter requires.
The helper is tailored to these three streams and this approval policy; adapt
it when sources or legitimate carry-over change. It neither authenticates the
input nor decides which real-world task it belongs to. Use a new session for
an unrelated task.

`test_grounds.py` tests each missing source, accidental old evidence, explicitly
allowed memory carry-over, wrong reading selection, a new decision grounded
on the wrong sources, and withdrawal without replacement. Its negative source
variants deliberately make the wrong decision: all operations can succeed and
a new approval can exist while the application's permit check still refuses it.
