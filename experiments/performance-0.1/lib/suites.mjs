// Which modes run on which workload. Every mode is one child process per
// target per repeat; see README.md for what each mode times.

const GLOWCAP_NATIVE = ['apply', 'dispatch_view_json', 'dispatch_json', 'dispatch_outcome_json', 'web.dispatch_view',
  'web.dispatch_outcome', 'web.dispatch', 'read', 'web.read', 'lifecycle'];
const GLOWCAP_WASM = ['published-method', 'adapter', 'raw.dispatch_view', 'raw.dispatch_outcome', 'raw.dispatch',
  'abi.dispatch_view', 'abi.dispatch_outcome', 'read', 'abi.read', 'kit', 'kit.read', 'lifecycle', 'micro'];
const SCALED_NATIVE = ['apply', 'web.dispatch_view', 'web.dispatch_outcome', 'read', 'lifecycle'];
const SCALED_WASM = ['raw.dispatch_view', 'abi.dispatch_view', 'read', 'kit', 'lifecycle'];
// The decision workloads get every mode Glowcap gets except the three that
// run through the Glowcap adapter, so their commits, reopens, evidence and
// refusals are measured on every path the replay is.
const DECISION_WASM = GLOWCAP_WASM.filter((mode) => !['published-method', 'adapter', 'adapter-resume'].includes(mode));

const JOBS = [
  { workload: 'glowcap-replay', native: GLOWCAP_NATIVE, wasm: GLOWCAP_WASM },
  { workload: 'glowcap-resume', native: ['lifecycle'], wasm: ['lifecycle', 'adapter-resume'] },
  // Binding evaluation, by difference: the same stream without bindings.
  { workload: 'glowcap-unbound', native: ['apply', 'dispatch_view_json', 'web.dispatch_view', 'read'], wasm: ['abi.dispatch_view', 'raw.dispatch_view'] },
  { workload: 'glowcap-scaled-16', native: SCALED_NATIVE, wasm: SCALED_WASM },
  { workload: 'glowcap-scaled-64', native: SCALED_NATIVE, wasm: SCALED_WASM },
  { workload: 'ledger-session', native: GLOWCAP_NATIVE, wasm: DECISION_WASM },
  { workload: 'trail-rescue-scenarios', native: GLOWCAP_NATIVE, wasm: DECISION_WASM },
];

// Performance Optimization 0.1 (experiments/performance-opt-0.1): the kit's
// dispatchView against the two paths a host wanting the view had, kit
// dispatch() + view() and the raw dispatch_view + JSON.parse, and the raw
// dispatch_view_outcome it is built on, on the workloads whose event classes
// the owner named. A target without the new entry points skips those modes.
const VIEW_PATH_MODES = ['raw.dispatch_view', 'raw.dispatch_view_outcome', 'kit', 'kit.dispatchView'];
const VIEW_PATH_JOBS = ['glowcap-replay', 'ledger-session', 'trail-rescue-scenarios']
  .map((workload) => ({ workload, native: [], wasm: VIEW_PATH_MODES }));

export const SUITES = {
  // The measurement. `published-method` always runs 3 rounds with no
  // warm-up, as the published harness did; every other per-event mode runs
  // one untimed warm-up pass and then `rounds` timed passes.
  baseline: {
    rounds: 3,
    warmup: 1,
    publishedRounds: 3,
    lifecycleSamples: 30,
    lifecycleWarmup: 3,
    resumeRuns: 3,
    microSamples: 2000,
    maxEvents: null,
    repeatCap: null,
    jobs: JOBS,
  },
  // Every job end to end on a short stream, to prove the harness runs.
  // Its numbers are not measurements. A smoke suite runs one repeat by
  // default and on any power plan.
  smoke: {
    smoke: true,
    rounds: 1,
    warmup: 0,
    publishedRounds: 1,
    lifecycleSamples: 3,
    lifecycleWarmup: 1,
    resumeRuns: 1,
    microSamples: 50,
    maxEvents: 300,
    repeatCap: 2,
    jobs: JOBS,
  },
  // The baseline's per-event method (one warm-up pass, 3 timed passes) on
  // the view-path modes only.
  'view-path': {
    rounds: 3,
    warmup: 1,
    publishedRounds: 3,
    lifecycleSamples: 30,
    lifecycleWarmup: 3,
    resumeRuns: 3,
    microSamples: 2000,
    maxEvents: null,
    repeatCap: null,
    jobs: VIEW_PATH_JOBS,
  },
  'view-path-smoke': {
    smoke: true,
    rounds: 1,
    warmup: 0,
    publishedRounds: 1,
    lifecycleSamples: 3,
    lifecycleWarmup: 1,
    resumeRuns: 1,
    microSamples: 50,
    maxEvents: 300,
    repeatCap: 2,
    jobs: VIEW_PATH_JOBS,
  },
};
