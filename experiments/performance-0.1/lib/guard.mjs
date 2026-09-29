// The job guard (run.mjs --job-guard): the keep rule applied to every timed
// job as it ends, not only to a whole repeat afterwards, so that a job that
// overlapped a burst of background load is discarded and run again in its
// place, before the next job, keeping the interleaving.
//
// It reads the load monitor's samples (typeperf, every second): total CPU over
// all logical processors, each pinned processor's use, and Windows' System
// process and Defender's scanner (MsMpEng) in percent of one core. A sample
// stamped T covers the time since the sample before it, so the samples of a
// job are those whose interval overlaps it, from the one covering its start
// to the one covering its end.
//
// The rule was fixed before any session it judged, from the load recorded in
// earlier sessions and a 150 s idle trace (Performance Optimization 0.1,
// RESULTS.md section 7): on this laptop the System process takes one core for
// 3-4 s about once a minute, and Defender scans in bursts; each is under 16%
// of a core at other times, and the benchmark's own load stays on its pinned
// processors.
export const JOB_GUARD = Object.freeze({
  // Over the job: the per-repeat rule, applied per job.
  meanTotalAtMost: 12,
  totalAtMost: 20,
  // In any one sample.
  processes: Object.freeze(['System', 'MsMpEng']),
  processAtMost: 30,
  outsideCoresAtMost: 2,
  // A discarded job is run again after the load settles: up to this many
  // attempts, each after SETTLE samples in a row within the process rule and
  // with total CPU at most meanTotalAtMost (waiting at most settleSeconds).
  attempts: 5,
  settleSamples: 3,
  settleSeconds: 60,
});

export function describeGuard(rule = JOB_GUARD) {
  return `a timed job is kept when, over the 1-s samples covering it, the mean total CPU is at most ${rule.meanTotalAtMost}% and no sample is above ${rule.totalAtMost}%, `
    + `and in no sample does ${rule.processes.join(' or ')} use more than ${rule.processAtMost}% of one core or are more than ${rule.outsideCoresAtMost} cores busy outside the pinned processors; `
    + `otherwise it is discarded and run again in place (up to ${rule.attempts} attempts), after ${rule.settleSamples} quiet samples in a row`;
}

// typeperf rows (cells: time, then one value per counter in the monitor's
// order) to samples. typeperf writes -1 for a process with no instance, and
// now and then for a processor or the total when a reading failed; such a
// value is missing, not zero. A missing total leaves the sample out of the
// total CPU figures; a missing pinned processor counts as idle there, which
// can only raise the cores counted as busy outside the pinned processors.
export function toSamples(rows, counters) {
  const index = (name) => counters.indexOf(name) + 1;
  const processors = counters.filter((name) => /^processor \d+$/.test(name)).map((name) => Number(name.slice(10)));
  const processes = counters.filter((name) => name.startsWith('process ')).map((name) => name.slice(8));
  const samples = [];
  for (const row of rows) {
    if (row.length !== counters.length + 1) continue;
    const at = new Date(row[0]).getTime();
    if (!Number.isFinite(at)) continue;
    const value = (cell) => {
      const number = Number(cell);
      return Number.isFinite(number) && number >= 0 ? number : null;
    };
    samples.push({
      at,
      total: value(row[index('total')]),
      pinned: Object.fromEntries(processors.map((processor) => [processor, value(row[index(`processor ${processor}`)])])),
      processes: Object.fromEntries(processes.map((name) => [name, value(row[index(`process ${name}`)])])),
    });
  }
  samples.sort((a, b) => a.at - b.at);
  return samples.map((sample, position) => ({ ...sample, since: position ? samples[position - 1].at : sample.at - 1000 }));
}

// The samples whose interval overlaps [fromMs, toMs], and whether they cover
// all of it.
export function covering(samples, fromMs, toMs) {
  const within = samples.filter((sample) => sample.at > fromMs && sample.since < toMs);
  const covered = within.length > 0 && within[0].since <= fromMs && within.at(-1).at >= toMs;
  return { samples: within, covered };
}

const round = (value, places = 2) => (Number.isFinite(value) ? Number(value.toFixed(places)) : null);

// One job's load and the verdict. processors: the job's pinned processors;
// logicalProcessors: all of the machine's.
export function judge(samples, { processors = [], logicalProcessors, covered = true, rule = JOB_GUARD } = {}) {
  const reasons = [];
  if (!samples.length) reasons.push('no load samples covered the job');
  else if (!covered) reasons.push('the load samples did not cover the whole job');
  const totals = samples.map((sample) => sample.total).filter((value) => value !== null);
  const meanTotal = totals.length ? totals.reduce((sum, value) => sum + value, 0) / totals.length : null;
  const maxTotal = totals.length ? Math.max(...totals) : null;
  const outside = samples.filter((sample) => sample.total !== null).map((sample) => {
    const pinned = processors.reduce((sum, processor) => sum + (sample.pinned[processor] ?? 0), 0);
    return (sample.total * logicalProcessors - pinned) / 100;
  });
  const maxOutside = outside.length ? Math.max(...outside) : null;
  const maxProcess = Object.fromEntries(rule.processes.map((name) => {
    const values = samples.map((sample) => sample.processes?.[name]).filter((value) => value !== null && value !== undefined);
    return [name, values.length ? Math.max(...values) : null];
  }));
  if (meanTotal !== null && meanTotal > rule.meanTotalAtMost) reasons.push(`mean total CPU ${round(meanTotal, 1)}% > ${rule.meanTotalAtMost}%`);
  if (maxTotal !== null && maxTotal > rule.totalAtMost) reasons.push(`a sample of ${round(maxTotal, 1)}% total CPU > ${rule.totalAtMost}%`);
  for (const [name, value] of Object.entries(maxProcess)) {
    if (value !== null && value > rule.processAtMost) reasons.push(`${name} at ${round(value, 0)}% of a core > ${rule.processAtMost}%`);
  }
  if (maxOutside !== null && maxOutside > rule.outsideCoresAtMost) reasons.push(`${round(maxOutside, 1)} cores busy outside the pinned processors > ${rule.outsideCoresAtMost}`);
  return {
    kept: reasons.length === 0,
    reasons,
    samples: samples.length,
    meanTotal: round(meanTotal),
    maxTotal: round(maxTotal),
    maxOutsideCores: round(maxOutside),
    maxProcess: Object.fromEntries(Object.entries(maxProcess).map(([name, value]) => [name, round(value, 1)])),
  };
}

// True when the last settleSamples samples are quiet: total CPU at most
// meanTotalAtMost and each watched process within processAtMost.
export function settled(samples, rule = JOB_GUARD) {
  const last = samples.slice(-rule.settleSamples);
  return last.length === rule.settleSamples && last.every((sample) => sample.total !== null && sample.total <= rule.meanTotalAtMost
    && rule.processes.every((name) => (sample.processes?.[name] ?? 0) <= rule.processAtMost));
}
