# CAVEAT Language for Python

A decision ledger for agents. A Caveat program records what each decision
rests on: values carry their evidence and caveats, a decision keeps the
grounds it was made on, and when evidence is withdrawn, the decisions that
rested on it reopen, with their original grounds still on record.

This package runs the same Rust runtime as the
[`caveat-lang`](https://www.npmjs.com/package/caveat-lang) npm package,
compiled for Python instead of WebAssembly. It returns the same bytes: every
outcome, snapshot, view and save matches the npm kit on the repository's
whole scenario corpus (see [Same bytes as the npm kit](#same-bytes-as-the-npm-kit)).
It has no Python dependencies.

```python
import caveat_lang as caveat

with open("assessment.cav", encoding="utf-8") as file:
    source = file.read()

with caveat.open(source) as session:
    session.dispatch("observe", {"confidence": 85})   # a tool observation
    outcome = session.dispatch("assess")              # a decision that rests on it
    print(isinstance(outcome, caveat.Accepted))       # True

    session.dispatch("retract")                       # the observation was wrong
    for revision in caveat.decision_history(session.snapshot(), "assessment"):
        print(revision["revision"], revision["status"], revision["committed"]["because"])
        # assessment@1 reopened ['observations@1']
```

The language itself is documented with the npm kit:
[Caveat on one page](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/REFERENCE.md) and the
[worked example](https://github.com/WSattazahn/caveat-lang/blob/main/kit/docs/WORKED_EXAMPLE.md).

## Install

Not yet on PyPI. From a clone of the repository, with a Rust toolchain:

```sh
pip install ./python
```

This builds the native module once (a few minutes). Python 3.9 or later.

## The API

`caveat.open(source)` loads a program and returns a `Session`;
`caveat.restore(source, save)` resumes one from `Session.save()` text, with
the exact source it came from. A session is a context manager.

`session.dispatch(event, payload)` sends one event. The payload holds exactly
the parameters the program declares for that event (`event observe
confidence ...;` takes `{"confidence": 85}`), and an event without parameters
takes none; anything else is rejected as `input/payload_invalid`. It returns:

- `Accepted`, with `.snapshot`, the session's state after the event, or
- `Rejected`, with `.origin`, `.code` and `.message`. The session is
  unchanged. Origins and codes are those of
  [Dispatch 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-dispatch-0.1.md).

Tell them apart with `isinstance(outcome, caveat.Rejected)`, which every type
checker understands; then `.code` and `.message`, or `.snapshot`, are typed.
In Python 3.10 and later, `match outcome:` with
`case caveat.Rejected(code=code):` reads the same. Each also has `.accepted`
(`True` or `False`), for printing and logging.

| Method | Returns |
| --- | --- |
| `dispatch(event, payload=None, *, agent=None)` | `Accepted` (snapshot) or `Rejected` |
| `dispatch_view(event, payload=None, *, fields=None, agent=None)` | `AcceptedView` (View 0.1, optionally only `fields`) or `Rejected` |
| `dispatch_view_delta(event, payload=None, *, agent=None)` | `AcceptedDelta` (View 0.2 delta) or `Rejected` |
| `snapshot()`, `snapshot_text()` | the state, as a dict or the runtime's text |
| `view(schema="0.1")`, `view_text(schema="0.1")` | the per-event view, 0.1 or 0.2 |
| `save(*, compact=False, positional=False)` | JSON text to store |
| `drain_archive()`, `undrained` | departed records ([departure](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-departure-0.1.md)) |
| `close()` | nothing; the session is closed |
| `state` | "open", "closed" or "fatal" |

Module functions: `check(source, *, unused=False, records=False, save=None)`
returns the advisory report, `interface(source)` the program's events and
names, and `decision_history(snapshot_or_view, decision)` every revision of a
decision series with what it was committed on and what reopened it.

Errors are `CaveatError` subclasses: `LoadError` (the program does not load),
`RestoreError` (the save does not restore with this source), `PayloadError`
(the payload cannot be sent as JSON; nothing changed), `FatalError` (the
session failed and cannot be used again), `ClosedError`, and `ThreadError`
(the session was used from another thread; nothing changed). A rejected event
is never an error.

The package is typed (`py.typed`) and checks clean under `mypy --strict` and
pyright. A session belongs to the thread that made it: open one session in
each thread that needs one. Free-threaded Python builds keep the GIL on while
the package is loaded.

### Agents

When a main agent and a sub-agent send events to one program, the program can
say which agent may send which event
([Agents 0.1](https://github.com/WSattazahn/caveat-lang/blob/main/spec/caveat-agents-0.1.md)),
and `agent=` says who sent each one:

```python
session.dispatch("sub_report", {"n": 3}, agent="sub")
```

An event from an agent the program does not allow is rejected as
`input/wrong_agent`, and an accepted event's records carry its agent.

### Payloads

A payload is a dict (or any mapping) of JSON values: None, bools, strings,
ints, floats, lists, tuples and mappings with string keys. Integer types such
as `numpy.int64` count as ints, and `numpy.float64` is a float. Numbers mean
what they mean to the npm kit: `1.0` is sent as `1`, as JavaScript sends it.
An int beyond 2\*\*53, a float that is not finite, a string with a lone
surrogate, and anything else raise `PayloadError`.

### In a notebook

An `Accepted` outcome and an open `Session` show as tables in Jupyter: the
program's bindings, and each decision with its status, what it rests on, its
caveats and what reopened it.

## An agent that records its decisions

[`examples/claim_checker.py`](https://github.com/WSattazahn/caveat-lang/blob/main/python/examples/claim_checker.py) is an agent that
checks a claim against a file with a tool, records the observation, and
approves the claim. The file then changes; the agent reads it again and
withdraws its observation, and Caveat reopens the approval while keeping what
it was made on:

```text
$ python examples/claim_checker.py --claude
First reading: the notes support the claim at 95.
  observe: accepted
  assess: accepted
  verdict: approved
Second reading: the notes support the claim at 2.
  retract: accepted
  verdict: reopened
Decision log:
  assessment@1 is reopened: committed at event 2 on observations@1; reopened at event 3 because of recheck
```

`--claude` has Claude judge through the `claude` command; without it, a
keyword judge runs anywhere.

## Same bytes as the npm kit

`tests/test_differential.py` replays every scenario file in the repository
through the npm kit's WebAssembly runtime and through this package, and
requires identical output at every step: each dispatch outcome (turning
through the three dispatch forms), the final snapshot, both views, all three
saves and their restores, the archive, `check` and `interface`. It also
compares payload number text with JavaScript's on 8,000 values.

```sh
npm run build                                  # the WebAssembly runtime, dist/pkg-reactive
pip install ./python
python -m unittest discover -s python/tests -v
```

Without Node or the WebAssembly build, the differential tests skip; set
`CAVEAT_DIFFERENTIAL=1` to make that a failure.

## Supply chain

- No Python dependencies. The native module compiles the runtime crate
  (`runtime/`) with the locked crate versions the WebAssembly build uses
  (`python/Cargo.lock` adds only PyO3 and what it needs).
- Each wheel carries a CycloneDX bill of materials for the Rust crates in it,
  at `caveat_lang-VERSION.dist-info/sboms/` (PEP 770).
- The crates' license notices are in
  [THIRD_PARTY_NOTICES.md](https://github.com/WSattazahn/caveat-lang/blob/main/python/THIRD_PARTY_NOTICES.md), generated by
  `node scripts/third-party-notices.mjs --python`.
- One wheel per platform serves every CPython from 3.9 on (the stable ABI,
  abi3). The source distribution carries the runtime's source and builds
  alone with maturin and a Rust toolchain, as conda-forge's Rust packages
  do, and its tests run from it.
