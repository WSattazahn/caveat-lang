# Author instructions (@@ID@@)

You will implement a small piece of game logic, the Glowcap beat, in
@@LANGUAGE@@. It is built in phases: the base behaviour, then one change
request at a time. You see a phase's request and scenarios only after you
commit the phase before it. Requests are cumulative: each phase keeps every
earlier rule unless the new request changes it.

## What to build

@@WHAT@@

`dispatch(event)` receives events such as `{ type: 'absorb', id: 'cave', kind: 'glowcap' }`
or `{ type: 'tick', dt: 0.0625 }`. A rejected event must throw and leave the
view exactly as it was. `view()` returns the view shape the phase files
describe. From the phase that asks for it, `createPolicy(saved)` resumes from
a value `save()` returned.

## How to work

1. `node run.mjs status` names the current phase. Read `phases/<phase>.md`
   (and keep every earlier phase file in mind). `scenarios/current.json` holds
   the phase's scenarios: steps such as `['absorb', id, kind]`,
   `['tick', dt, times]`, `['expect', partialView]` (only the listed fields are
   checked; lists are compared as sets except where a request says order
   matters), `['reject', event]` (dispatch must throw and leave the view
   unchanged) and `['resume']` (save, round-trip through JSON, resume, compare
   the complete views, continue on the resumed policy). A step `[type, {…}]`
   dispatches `{ type, …payload }`.
2. Implement it under `impl/`.
3. `node run.mjs test` runs the phase's scenarios. Every run is recorded.
4. When every scenario passes, `node run.mjs commit` ends the phase and
   releases the next. If you cannot make a phase pass, `node run.mjs commit
   --incomplete "why"` ends it anyway, recorded as incomplete; carry on.
5. Repeat until the runner says the last phase is committed.

Other runner commands: `node run.mjs check` (@@CHECK@@), `node run.mjs try
events.jsonl` (replays your own events, one JSON object per line, or
`{"resume": true}`, and prints each outcome and the final view)@@CAVEATCMD@@.

## Rules

- Run your program only through `node run.mjs`. Everything it does is logged;
  only `test` runs are measured. You may write scratch event files in this
  directory.
- Read only: this README, `phases/`, `scenarios/`, `impl/`, your own scratch
  files and notes@@READ@@. Nothing outside this directory, no web, no git
  history, no other agents.
- Do not edit `run.mjs`, `phases/`, `scenarios/`, `tsconfig.json`,
  `package.json` or `node_modules/`.
@@RULES@@
- No one will give you hints or feedback. Decide ambiguities yourself and
  note them.
- Keep `notes.md` in this directory: the references you read, how you checked
  your work, uncertainties, and any departure from these rules. A sentence or
  two per phase is enough.

When the runner says the last phase is committed, finish `notes.md` and stop.
Your final answer: a short report of what you did, per phase, and anything
that went wrong.
