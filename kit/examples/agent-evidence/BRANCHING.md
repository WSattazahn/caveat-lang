# Try a branch without replacing the live session

There is no `whatif` command or Serve operation in this candidate. Existing
save/restore can branch, provided the hypothetical work gets its own runtime.
Restoring into the live server replaces its session; it is not a hypothetical.

For Python, a second `CaveatServer` gives the branch its own process and WASM
instance. Keep the exact source file and runtime command unchanged. Restore
validates source identity and saved state; do not assume arbitrary source or
version compatibility. This runnable recipe uses the shipped policy:

<!-- branching-example -->
```python
from pathlib import Path
from caller import CaveatServer, assess_answer

program = str(Path(__file__).with_name("assessment.cav"))
with CaveatServer(program) as live:
    assert assess_answer(live, 85).succeeded
    saved, snapshot = live.save(), live.snapshot()
    with CaveatServer(program) as branch:
        assert branch.ready["runtime"] == live.ready["runtime"]
        branch.restore(saved)
        assert branch.dispatch("observe", {"confidence": 40}).accepted
        refusal = branch.dispatch("assess")
        assert (refusal.origin, refusal.code) == ("policy", "reject")
        report = branch.request("explain")["report"]
        assert report
    assert live.save() == saved
    assert live.snapshot() == snapshot
    assert live.dispatch("observe", {"confidence": 30}).accepted
```

Save that block beside `caller.py` and run it with the same `CAVEAT_COMMAND`
as the other examples. `python3 -B -m unittest -v test_branching` executes
exactly this block. Accepted, rejected and ordinary fatal branches are also
covered by the repository's JavaScript isolation tests.

In JavaScript, call `loadRuntimeFromDirectory()` again before restoring: that
loader creates a separate module instance each time. Merely calling
`runtime.restore(source, saved)` on the **same** runtime gives another session
sharing its WASM instance. A WASM trap invalidates every session on that
instance. Reusing an already imported module namespace does not create a
fresh instance. Use a fresh loader invocation, or a separate process, for a
disposable hypothetical runtime. A documented ordinary session-level fatal
error and a WASM trap have different scopes.

The repository's trap test injects an actual WASM `unreachable` trap through
the production session adapter and checks two unaffected live sessions on
another instance, plus the same-instance inverse control. It is an injected
failure test, not a naturally Caveat-authored trap or proof of protection
against resource exhaustion, process-wide failure, or external host effects.
A public `whatif` API is deferred until its complete isolation and resource
lifecycle contract is verified. This recipe does not add such an API.

A branch starting after `observe 85; assess` and adding `observe 40` keeps
both readings and the earlier decision. Replacing the original 85 with 40
requires a checkpoint **before** that observation, then replay from there.
Those questions are different. Keep checkpoints explicitly; save/restore
does not edit recorded history.

Branch output answers what this authored policy does from this state with
these events. Whether that outcome is better needs an external evaluator or
an authored objective. The runtime cannot infer it.
