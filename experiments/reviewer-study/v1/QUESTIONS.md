# Questions

Answer these for each history in `histories/` (`H1`, `H2`, `H3`), as of after
all of that history's lines. Lines are numbered from 1 in the history file,
and an observation is named by the line that recorded it (`#3` is the
observation recorded by line 3). Decisions are numbered in the order they were
made: the first merge decision of a history is decision 1.

## Q1. What each decision rests on

For every merge decision made in the history:

- (a) `value`: its value.
- (b) `rests_on`: the observations it rests on, by line.
- (c) `caveats`: the caveats those observations carried when the decision was
  made.
- (d) `retains`: the caveat it retains by choice.
- (e) `grant`: the grant that permitted it, by line.
- (f) `state`: its current state, one of `in_force`, `reopened`, or
  `superseded` (a later decision has been made); and, if it was ever reopened,
  `reopened_by`, the line of the event that reopened it, and `reason`, the
  reason text that reopening produced. Otherwise both are `null`.

## Q2. What a withdrawal would touch

The task says that withdrawing an observation the decision in force rests on
reopens it, and that withdrawing anything else reopens nothing. For each probe
in `probes.json` (an observation or a grant, by line): if that item were
withdrawn now, would the decision in force be reopened? Answer `yes`, `no`,
`none_in_force` or `cannot_determine`. No event in the task withdraws a
review, a scan or a grant, so this is a question about the record, not a run.

## Q3. The note

For each numbered statement in `notes/H<n>.md`: `true`, `false` or
`cannot_determine`. A note may contain errors; judge every statement.

## Answers and receipts

Write `ANSWERS.json` as `answers.schema.json` describes. Every answer is an
object `{"answer": …, "receipt": {"kind": …, "text": …}}`. Any answer may be
the string `"cannot_determine"`; answer that rather than guess. The receipt
says what the answer rests on:

- `record`: output of a command the package or the program ships (`explain`,
  `dependents`, `replay`, `test`, or a dump of the program's own view or logs).
  Quote the lines.
- `code`: a file and line of the program, such as `gate.ts:42`.
- `run`: a script or scenario you wrote, and its output.
- `none`: nothing behind the answer.
