You are taking part in a study of how a newcomer maintains a program written in the Caveat programming language. You have never used it before; everything you need is in the package in this directory. Your run ID is {RUN}. Your working directory is the directory this session started in. It contains `caveat-lang-0.1.0-rc.2.tgz`, the Caveat package; `TASK.md`, the task a program was written for; `ferry.cav`, the program another developer wrote for it; and `CHANGE.md`, a change request.

Rules. Work only inside your working directory. Read only files inside it, including the package once it is installed there. Do not read or list anything else on this computer, do not use the web, and do not start or contact other agents or sessions. Do not use git except to commit and push your finished work in step 4.

1. Install the package. In your working directory, run `npm init -y`, then `npm install --offline --no-audit --no-fund ./caveat-lang-0.1.0-rc.2.tgz`. Its documentation is in `node_modules/caveat-lang/`; start with its `README.md` and the `docs/` directory.
2. Change `ferry.cav` so that it does what `TASK.md` asks, as amended by `CHANGE.md`. If you find that the program did not meet `TASK.md` before the change, fix that too. Test it as much as you like, with the package's `caveat` command or your own scripts. Keep any tests you write in your working directory.
3. When you have finished, write `NOTES.md` with: the documentation you read, what you changed and why, how you tested the program, anything you are unsure of, and any departure from the rules above.
4. Commit `ferry.cav`, `NOTES.md` and your tests (not `node_modules/`), and push them to this session's branch.

Then reply with a short summary. If you cannot complete everything, commit and push your best version of `ferry.cav` and say what is missing.
