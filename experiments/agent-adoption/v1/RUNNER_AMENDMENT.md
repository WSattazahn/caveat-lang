# Post-run capture amendment â€” 2026-10-02

The registered A01â€“A03 executions and their evaluators finished at 00:55:54 UTC,
using the frozen runner SHA-256
`38e703c7765bae0c7f98cdac21caf5a5bf5a015e6959d8e4e68adf34f6c509d6`.
Its exact bytes are retained as `registration/executed-runner.py` and in commit
`247c7f4`. The registration remains unchanged; the task, scorer, fixtures and
submissions were not amended or rerun.

Review found that the runner suppressed output-reader I/O exceptions. No such
failure was observed in these runs: each retained transcript parses as JSONL
and includes its completed turn. Still, future executions must fail explicitly
if capture is incomplete. The current `run-trials.py` records read/write/flush/
close errors, marks capture incomplete, stops its owned process, and refuses
success even if that child exits zero. It also counts bytes actually retained.
The six synthetic controls in `runner-capture.test.py` exercise write, flush,
close, stdin/EOF, output-cap and timeout behavior; no trial agents run in them. CI now runs both these controls and the frozen
scorer mutation controls after building the runtime.

Historical reproduction uses the archived runner with the original study files
from the frozen commit. A future run with the amended runner must use `prepare`
to create a new registration. Changing current files never silently updates an
old registration or selects a replacement result.
