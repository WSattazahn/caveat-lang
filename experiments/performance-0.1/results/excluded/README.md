# Excluded results

Nothing in this directory is used by any statistic, table or claim in
[RESULTS.md](../../RESULTS.md), [tables.md](../tables.md) or
[attribution.json](../attribution.json). Their raw files are not committed:
discarded data is not evidence, and it would stay in the repository's
history. This index records what was discarded or superseded, when, and why.
Times are UTC on 2026-09-29; total CPU is over 24 logical processors, where
the benchmark itself is about 4% and the clean sessions averaged 8.7–9.3%.

## Discarded: overlapped background load

These runs overlapped other sessions' background jobs. They are not
replicates.

| Run (not committed) | When | Recorded load (total CPU) | Why |
| --- | --- | --- | --- |
| `baseline-loaded-1/` | 02:18:48–03:21:07 | mean 27.3%, median 20.6%, p95 70.2%, max 91.2%; 383 of 742 samples above 20% | Other sessions' jobs ran throughout: an automated Chrome renderer (about 1.4 cores), a Next.js development server recompiling in bursts of 2–6 cores, eslint and vitest runs, and cargo builds and tests. The busiest process before the run was `chrome`. |
| `baseline-loaded-2/` | 03:53:23–05:00:31 | mean 57.0%, median 53.4%, p95 100%, max 100%; 631 of 778 samples above 20% | The same background jobs, and then the 04:50–05:07 window (119 samples fall inside it), when other sessions ran `node settle.mts particleGel` four times (about 4 cores) and a Rust test binary. The busiest processes after the run were two `node` processes, two `rustc` processes and the `indexed_names` test binary, about 8 CPU-seconds each in the sampling window. |

The only use RESULTS.md makes of them is one sentence in section 5. It is
labelled as coming from these discarded runs, and it notes, as an
observation about load sensitivity only, how much slower the same
operations ran under this load.

## Superseded: load not recorded, or above the quiet level; rerun

These runs had too little load data to rule background load out, or recorded
load above the quiet level. Each was rerun on a quiet machine, and the reruns
replace them everywhere. RESULTS.md section 2 lists the reruns and their
load.

| Run (not committed) | When | Recorded load | Why |
| --- | --- | --- | --- |
| `superseded/instrumented/` | 03:28:17–03:52:46 | mean 11.3%, median 11.0%, p95 14.9%, max 21.1% (289 samples); 29% in one sample just before the run | Above the quiet level for the whole run. No record of which processes used the extra CPU. It ran between the two discarded baselines. |
| `superseded/verbatim-1-unpinned.json`, `superseded/verbatim-1-pinned.json` | 03:22:15–03:28:04 | None during the runs. Spot samples before and after: 4–18% | No record during the runs. They started one minute after `baseline-loaded-1` ended. |
| `superseded/method/`, `superseded/verbatim-2-pinned.json`, `superseded/verbatim-2-unpinned.json` | 06:20:31–06:29:00 | None during the runs: the 4 s `alone` runs were shorter than the 5 s monitor interval, and `verbatim.mjs` had no monitor then. Spot samples before and after: 0–15% | No record during the runs |
| `superseded/allocator/allocbench.json` | 08:15:24–08:15:30 | None | No load record |
| `superseded/cores/clocks-1.json`, `superseded/cores/clocks-2.json` | 08:15:30–08:17:52 | After each probe only: 2–16% | No record during the probes |
