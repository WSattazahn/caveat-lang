# Baseline withdrawal extraction attribution

Baseline c1d8fea1164e9d89edc8ff33af5a3bb608bc37d3; diagnostic source cb5faadfa2a9b40a11764c303f475fffb981a9f8. Six diagnostic and three plain control processes completed. No production optimization is included.

The diagnostic release build used rustc 1.98.1, `--no-default-features --features withdrawal-extraction-profile`, and the separate `withdrawal_extraction_profile` example. It completed successfully in 1m19s from clean tracked source. Diagnostic executable SHA-256 is `4c7791a31239195acd17075993c1acec97ae29ecf35a1205d344a64acbcc8254`; plain executable SHA-256 is `c206c83292dc10e908246a3bace2d6d0dc3a1dc9b4a8b6c0c9126bc1edfe32fd`. Exact compiler/build argv and stdout/stderr are retained in `diagnostic-build.json` and its sidecars. Registration SHA-256 is `53044ddfe5bab62cfd28ac484a27412a0c68f3e521f9cff7c640b1f7a28f83aa`.

All three large matched pairs have exact native save/archive/inventory/work equality. Every raw stdout/stderr, command, exit status and hash is preserved. Small diagnostic processes also execute the unchanged rollback assertions; they have no same-size plain process in this stage.

| Cycles / trial | Successful probes | Shifted elements | Estimated shifted header bytes | COW detaches / cloned elements |
|---|---:|---:|---:|---|
| 60 / 1 | 2,435 | 4,822 | 385,760 | 1 / 120 |
| 300 / 1 | 80,819 | 99,478 | 7,958,240 | 1 / 600 |
| 1000 / 1 | 590,511 | 1,410,486 | 112,838,880 | 1 / 2,000 |
| 3000 / 1 | 8,137,067 | 9,865,930 | 789,274,400 | 1 / 6,000 |
| 3000 / 2 | 8,137,067 | 9,865,930 | 789,274,400 | 1 / 6,000 |
| 3000 / 3 | 8,137,067 | 9,865,930 | 789,274,400 | 1 / 6,000 |

Three rejected attempts in each process retain their individual diagnostic counters despite transactional rollback; complete per-attempt values are in JSON.

| Phase / trial | Plain apply ms | Diagnostic apply ms | Diagnostic block ms | Block / diagnostic apply | Diagnostic departure ms |
|---|---:|---:|---:|---:|---:|
| release / 1 | 75.7946 | 77.5113 | 15.1440 | 19.54% | 66.1367 |
| release / 2 | 75.7196 | 76.7637 | 16.2988 | 21.23% | 64.8041 |
| release / 3 | 75.6426 | 77.7640 | 16.2572 | 20.91% | 65.6944 |
| rejected / 1 | 69.7491 | 72.1923 | 15.8552 | 22.45% | 65.7618 |
| rejected / 2 | 70.0551 | 71.6750 | 15.6618 | 22.22% | 65.3780 |
| rejected / 3 | 69.9391 | 72.7942 | 16.3144 | 22.04% | 66.6095 |

Successful rows are single observations; rejected rows use each process’s median of three observations (block shares are medians of per-attempt fractions). Across the three processes:

| Phase | Median plain apply ms | Median diagnostic apply ms | Median block ms | Median block share (observed process range) |
|---|---:|---:|---:|---|
| release | 75.7196 | 77.5113 | 16.2572 | 20.91% (19.54–21.23%) |
| rejected | 69.9391 | 72.1923 | 15.8552 | 22.22% (22.04–22.45%) |

Compared with the plain controls, the diagnostic median-of-process-values is higher by 1.7917 ms (2.37%) for successful release and 2.2532 ms (3.22%) for rejected release. These observed differences include timing variation and all instrumentation effects; they are not an isolated clock-cost calibration. The measured block is about one fifth of dispatch, and it includes required COW work that a replacement still has to handle. Achieving 20% improvement in both full-dispatch medians is therefore a tight experimental target, not an established consequence of batching. The measured quadratic search/shift work justifies one scoped implementation trial under the unchanged acceptance gates; a miss must remain a miss.

The block timer covers the original position/Arc::make_mut/Vec::remove path, including its first COW. Exact probes are inferred from index/length without a counter inside the search predicate. Shifted bytes describe Withdrawal headers only; this is not measured memory traffic. Clock reads, TLS updates, and different diagnostic sample representation perturb the diagnostic process. No guessed overhead is subtracted. These shares support a scoped optimization hypothesis, not a promised speedup or the final >=20% gate.

The final registered target remains the unmodified collector_profile executable, eight balanced matched pairs, both latency-ratio criteria and per-pair successful/rejected peak limits. Baseline attribution is preserved even if the later target misses. No end-to-end performance acceptance follows from these diagnostics.
