// One definition of every statistic, used for native and WebAssembly samples
// alike. Quantiles use the rule experiments/glowcap/harness.mjs published
// with: sort ascending and take samples[floor(p * (n - 1))], so the median of
// an even count is the lower median and matches the published 51.6 µs method.

export function quantile(sorted, p) {
  if (sorted.length === 0) return null;
  return sorted[Math.floor(p * (sorted.length - 1))];
}

const round = (value, places = 3) => (value === null || value === undefined ? null : Number(value.toFixed(places)));

// samples: numbers in one unit. Returns null for an empty set.
export function summarize(samples) {
  const n = samples.length;
  if (n === 0) return null;
  const sorted = Float64Array.from(samples).sort();
  let sum = 0;
  for (const value of sorted) sum += value;
  const p25 = quantile(sorted, 0.25);
  const p75 = quantile(sorted, 0.75);
  return {
    n,
    median: round(quantile(sorted, 0.5)),
    p95: round(quantile(sorted, 0.95)),
    p99: round(quantile(sorted, 0.99)),
    p25: round(p25),
    p75: round(p75),
    iqr: round(p75 - p25),
    min: round(sorted[0]),
    max: round(sorted[n - 1]),
    mean: round(sum / n),
  };
}

// The spread of one statistic across independent runs (processes): its
// median, lowest and highest value, and the range as a share of the median.
export function spread(values) {
  const present = values.filter((value) => typeof value === 'number' && Number.isFinite(value));
  if (present.length === 0) return null;
  const sorted = Float64Array.from(present).sort();
  const median = quantile(sorted, 0.5);
  const min = sorted[0];
  const max = sorted[sorted.length - 1];
  return {
    runs: present.length,
    median: round(median),
    min: round(min),
    max: round(max),
    rangePercent: median === 0 ? null : round(((max - min) / median) * 100, 1),
  };
}

// rounds: an array of per-round sample arrays, each indexed by event position
// (null where an operation did not run for that event). Summarizes the pooled
// samples, each round on its own, and any named segments of event positions.
// Segments are [from, to) positions within one pass over the episodes.
export function summarizeRounds(rounds, segments = []) {
  const flat = (round) => round.filter((value) => value !== null && value !== undefined);
  const pooled = rounds.flatMap(flat);
  const result = {
    pooled: summarize(pooled),
    rounds: rounds.map((round) => summarize(flat(round))),
  };
  if (segments.length) {
    result.segments = {};
    for (const { name, from, to } of segments) {
      const values = rounds.flatMap((round) => flat(round.slice(from, to)));
      if (values.length) result.segments[name] = summarize(values);
    }
  }
  return result;
}

// Nanosecond integers (native) or microsecond floats (Node) to microseconds.
export function toMicroseconds(samples, unit) {
  if (unit === 'us') return samples;
  if (unit === 'ns') return samples.map((value) => (value === null ? null : value / 1000));
  throw new Error(`unknown sample unit ${unit}`);
}
