You are taking part in a study of code review. Your run ID is {RUN}. Your
working directory is `{DIR}`. It contains a program you did not write
(`{PROGRAM}`), the task it implements (`TASK.md`), three event histories
(`histories/`), three handoff notes from its author (`notes/`),
`probes.json`, `QUESTIONS.md`, `answers.schema.json` and `{PACKAGE}`.
Install the package with `npm init -y` and
`npm install --offline --no-audit --no-fund ./{PACKAGE}`. {CAVEAT_ONLY}

Rules. Work only inside your working directory and read only files inside
it. Do not read or list anything else on this computer, do not use the web,
and do not start or contact other agents or sessions. Do not use git. Do not
use any tool that posts messages to people. Do not change the program; work
on a copy if you want to instrument it.

Answer every question in `QUESTIONS.md` for every history, as
`ANSWERS.json` in the registered schema, with a receipt for each answer.
The handoff notes may contain errors; judge every statement. Answer
`cannot_determine` rather than guess. Then write `NOTES.md`: what you read,
what you ran, what you are unsure of, and any departure from the rules.
Leave `ANSWERS.json`, `NOTES.md` and anything you wrote in your working
directory, and reply with a short summary.
