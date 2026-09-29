# Excluded sessions

Nothing here is used by any table or statement in
[RESULTS.md](../../RESULTS.md) or [tables.md](../tables.md). The raw files of
these sessions are not committed: discarded data is not evidence. This index
records what was discarded, when and why. Times are Pacific daylight time on
2026-09-29 (UTC−7); total CPU is over the 24 logical processors, where the
benchmark itself is about 5% (one busy processor plus V8's helper threads).

The rule, applied per repeat from the run's typeperf samples (every second):
a repeat is kept when its mean total CPU is at most 12% and no sample is above
20%, and a session is used only when every repeat is kept (`load.mjs`).
Session A was judged by the first version of the rule (mean at most 12% and at
most 5% of samples above 20%), and failed it. After session P2, which passed
that first version with 4 of its 84 samples in repeat 1 above 20% (4.8%),
the rule was tightened to no sample above 20%, and P2 was discarded under it.
The rule was only ever tightened. The three kept sessions pass both versions.

| Session (not committed) | When | Recorded load (total CPU) | Why |
| --- | --- | --- | --- |
| A, processors 10–13, 5 repeats | 10:52:15–10:59:37 | mean 12.7%, p95 17.2%, max 36.2%; every repeat's mean 12.3–13.3% (above 12%) | Windows Defender's real-time scanner (`MsMpEng`) was using about 0.9 of a core: 94% of one core at 11:00, right after the session, and 88–95% in every 5 s sample from 11:02 to 11:11 (it was not sampled during the session itself; the session's total CPU was 5.5 points above the kept sessions'). The likely driver was an orphaned `ls.exe -F --color=auto --show-control-chars -R .cache` (Git for Windows, pid 114380, started 07:48:33 by a bash whose process was already gone; not started by this session), which opened about 250 files a second and read 3 bytes of each (about 1,800 file operations a second) while Defender ran about 5,800 file operations a second; by 11:15 both had stopped. My own process watcher (PowerShell `Get-Process` every 10 s, about 0.25 of a core) added about 1%; later sessions replaced it with a native typeperf of three process counters. The pre-session gate (total CPU below 12% for a minute) had passed at 10:52: a total-CPU gate alone does not see one busy core out of 24, so the later gates also required `MsMpEng` below 25% of a core. By 11:15 the `ls` process had ended and Defender was idle. |
| P1, processors 10–13, 5 repeats | 11:16:14–11:23:26 | mean 9.2%, p95 16.2%, max 32.9%; repeat 2 mean 11.4%, 10 samples above 20% | A burst from 11:18:25 to 11:19:08 during repeat 2: `MsMpEng` at 144% and `System` at 77% of a core at 11:19:00–11:19:05. Repeats 1, 3, 4 and 5 were quiet. |
| P2, processors 10–13, 5 repeats | 11:25:11–11:32:14 | mean 7.6%, p95 11.8%, max 52.3%; repeat 1 mean 9.1%, 4 samples above 20% | A burst from 11:25:33 to 11:25:36 (52%, then 20–23%), while the branch's `kit` job on the Glowcap replay ran, as the ChatGPT desktop app's process exited (its counter disappears at 11:25:37) and `System` rose to 55% of a core. Repeats 2–5 were quiet. |

Before any of them, a `view-path-smoke` run (10:45, short streams, one repeat,
not a measurement) checked that every mode runs on both builds.
