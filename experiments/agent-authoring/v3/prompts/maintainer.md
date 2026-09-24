You are taking part in a study of how a newcomer maintains a program written in the Caveat programming language. You have never used it before; everything you need is in the package you will install. Your run ID is {RUN}, and your working directory is {DIR}. It contains four files: `caveat-lang-0.1.0-rc.1.tgz`, the Caveat package; `TASK.md`, the task a program was written for; `pond.cav`, the program another developer wrote for it; and `CHANGE.md`, a change request.

Rules. Work only inside {DIR}. Read only files inside it, including the package once it is installed there. Do not read or list anything else on this computer, do not use the web, and do not start or contact other agents. Do not use git.

1. Install the package. In {DIR}, run `npm init -y`, then `npm install ./caveat-lang-0.1.0-rc.1.tgz`. Its documentation is in `node_modules/caveat-lang/`; start with its `README.md` and the `docs/` directory.
2. Change `{DIR}/pond.cav` so that it does what `TASK.md` asks, as amended by `CHANGE.md`. If you find that the program did not meet `TASK.md` before the change, fix that too. Test it as much as you like, with the package's `caveat` command or your own scripts. Keep any tests you write in {DIR}.
3. When you have finished, write `{DIR}/NOTES.md` with: the documentation you read, what you changed and why, how you tested the program, anything you are unsure of, and any departure from the rules above.

Then reply with a short summary. If you cannot complete everything, leave your best version of `pond.cav` and say what is missing.
