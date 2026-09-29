# Excluded sessions

Nothing here is used by any table or statement in
[RESULTS.md](../../RESULTS.md) or [tables.md](../tables.md). The raw files of
these sessions are not committed here: discarded data is not evidence. This
index records what was discarded, when and why. Times are Pacific daylight
time on 2026-09-29 (UTC−7); total CPU is over the 24 logical processors, where
the benchmark itself is about 5% (one busy processor plus V8's helper threads).

## The rules, in the order they were applied

1. **Per repeat** (sessions A to S3): from the run's typeperf samples (every
   second), a repeat is kept when its mean total CPU is at most 12% and no
   sample is above 20%, and a session is used only when every repeat is kept.
   Session A was judged by the first version (mean at most 12% and at most 5%
   of samples above 20%) and failed it. After P2, which passed that first
   version with 4 of its 84 samples in repeat 1 above 20% (4.8%), the rule was
   tightened to no sample above 20%, and P2 was discarded under it.
2. **Per timed job** (the committed sessions): the review of the first
   measurement found two jobs of its replicate session that overlapped bursts
   its own logs recorded, which a rule averaged over an 80 s repeat could not
   see. Every timed job is now judged by the 1-s samples covering it, as it
   ends, and run again in place if it overlapped a burst
   (`experiments/performance-0.1/lib/guard.mjs`, `run.mjs --job-guard`); the
   monitor now also samples `System` and `MsMpEng` every second and starts 5 s
   before the first job. The per-repeat rule still applies, over the samples
   covering the kept jobs. The rules were only ever tightened.

## Sessions

| Session | When | Recorded load (total CPU) | Why |
| --- | --- | --- | --- |
| A, processors 10–13, 5 repeats | 10:52:15–10:59:37 | mean 12.7%, p95 17.2%, max 36.2%; every repeat's mean 12.3–13.3% (above 12%) | Windows Defender's real-time scanner (`MsMpEng`) was using about 0.9 of a core: 94% of one core at 11:00, right after the session, and 88–95% in every 5 s sample from 11:02 to 11:11 (it was not sampled during the session itself; the session's total CPU was 5.5 points above the kept sessions'). The likely driver was an orphaned `ls.exe -F --color=auto --show-control-chars -R .cache` (Git for Windows, pid 114380, started 07:48:33 by a bash whose process was already gone; not started by this session), which opened about 250 files a second and read 3 bytes of each (about 1,800 file operations a second) while Defender ran about 5,800 file operations a second; by 11:15 both had stopped. My own process watcher (PowerShell `Get-Process` every 10 s, about 0.25 of a core) added about 1%; later sessions replaced it with a native typeperf of process counters. The pre-session gate (total CPU below 12% for a minute) had passed at 10:52: a total-CPU gate alone does not see one busy core out of 24, so the later gates also required `MsMpEng` below 25% of a core. |
| P1, processors 10–13, 5 repeats | 11:16:14–11:23:26 | mean 9.2%, p95 16.2%, max 32.9%; repeat 2 mean 11.4%, 10 samples above 20% | A burst from 11:18:25 to 11:19:08 during repeat 2: `MsMpEng` at 144% and `System` at 77% of a core at 11:19:00–11:19:05. |
| P2, processors 10–13, 5 repeats | 11:25:11–11:32:14 | mean 7.6%, p95 11.8%, max 52.3%; repeat 1 mean 9.1%, 4 samples above 20% | A burst from 11:25:33 to 11:25:36 (52%, then 20–23%), while the branch's `kit` job on the Glowcap replay ran, as the ChatGPT desktop app's process exited and `System` rose to 55% of a core. |
| S1, processors 10–13, 5 repeats (the first commit's primary session) | 11:34:05–11:41:05 | per repeat mean 7.0–7.5%, max 15.7% | Kept by rule 1 and used by the first commit (d5938cc), superseded by rule 2. Judged by rule 2 afterwards, from its load.csv, 6 of its 105 timed jobs break it: the monitor's first sample did not cover the first job's start, nor any sample the last job's end; and 4 jobs overlapped `System`'s burst of about one core, which recurs about once a minute on this laptop (2.1 cores busy outside the pinned processors; one also a mean of 13.1% total). |
| S2, processors 0–1, 3 repeats (the first commit's) | 11:42:33–11:46:12 | per repeat mean 6.9–7.1%, max 13.2% | As S1: 2 of 63 timed jobs break rule 2, the first and the last, whose starts and ends the monitor did not cover. |
| S3, processors 10–13, 5 repeats (the first commit's replicate) | 11:47:25–11:54:17 | per repeat mean 7.0–7.2%, max 17.4% | As S1, and the two jobs the review found: the branch's `raw.dispatch_view` on the Glowcap replay in repeat 1 (11:47:33.9–11:47:36.0; samples 14.7% and 17.4%, mean 16.1%, 2.6 cores busy outside the pinned processors; `MsMpEng` 15% and `System` 20% of a core over the 5 s sample at 11:47:36), with main's next job (2.6 cores outside), and the branch's `raw.dispatch_view_outcome` on the Glowcap replay in repeat 3 (11:50:17; mean 12.9%; `System` 59% of a core over the 5 s sample at 11:50:22). Their idle-tick medians were 46.2 µs (against 38.7–40.1 µs for the same job in repeats 2–5) and 41.9 µs (against 38.1–39.5 µs), and they drove that session's widest ranges. |

S1–S3's files are in this branch's history at d5938cc, under
`results/view-path-0x3C00/`, `view-path-0x3/` and
`view-path-0x3C00-replicate/`. Their `processes.csv` header names two counters
while every row has three values: its typeperf also sampled the ChatGPT
process, which had exited, and typeperf leaves a counter whose process has no
instance out of its header but still writes its column, as -1. The columns are
time, `MsMpEng`, ChatGPT (-1) and `System`. The committed sessions have no
`processes.csv`: their `load.csv` carries `System` and `MsMpEng`, and its
header names every column.

Before any of them, a `view-path-smoke` run (10:45, short streams, one repeat,
not a measurement) checked that every mode runs on both builds, and another
(12:45) that the job guard works.
