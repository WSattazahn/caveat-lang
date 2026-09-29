// summary.md from results.json. Every number is copied from the results;
// rows marked "derived" are differences of medians, not measurements.

const fixed = (value, places = 1) => (Number.isFinite(value) ? value.toFixed(places) : '-');
const short = (hash) => (hash ? `\`${hash.slice(0, 12)}\`` : '-');

function cell(stats) {
  if (!stats?.median) return '-';
  const { median, min, max, runs } = stats.median;
  const range = runs > 1 ? ` (${fixed(min)}–${fixed(max)})` : '';
  return `${fixed(median)}${range} · p95 ${fixed(stats.p95?.median)}`;
}

function segmentCell(stats, segment) {
  const value = stats?.segments?.[segment];
  if (!value) return '-';
  return value.runs > 1 ? `${fixed(value.median)} (${fixed(value.min)}–${fixed(value.max)})` : fixed(value.median);
}

function duringLine(during) {
  if (!during) return 'not monitored';
  const unit = (name) => (name.endsWith('MHz') ? ' MHz' : '%');
  const counters = Object.entries(during.counters).map(([name, stats]) => `${name} mean ${stats.mean}${unit(name)} p95 ${stats.p95}${unit(name)} max ${stats.max}${unit(name)}`).join('; ');
  return `${during.what}, ${during.samples} samples: ${counters}; total above 10% in ${during.totalAbove10Percent} and above 25% in ${during.totalAbove25Percent} samples${during.pinnedSumAbove150Percent === null ? '' : `; pinned processors summing above 150% in ${during.pinnedSumAbove150Percent}`}`;
}

const opStats = (report, target, workload, engine, mode, op) => report.results[target]?.[workload]?.[engine]?.[mode]?.acrossRuns?.[op];

function environmentSection(report) {
  const { machine, pinning, toolchain, loadBefore, loadAfter } = report.environment;
  const windows = machine.windows ?? {};
  const loadLine = (value) => {
    if (!value) return 'not recorded';
    if (value.loadavg) return `loadavg ${value.loadavg.map((v) => v.toFixed(2)).join(' ')}`;
    const busiest = (value.busiestProcesses ? [].concat(value.busiestProcesses) : []).map((p) => `${p.name} ${p.cpuSeconds}s`).join(', ');
    return `total CPU ${[].concat(value.totalCpuPercent ?? []).join('/')}%; busiest over the window: ${busiest || '-'}`;
  };
  return [
    '## Environment',
    '',
    `- CPU: ${windows.cpu ?? machine.cpuModel}; ${windows.cores ?? '?'} cores, ${windows.logicalProcessors ?? machine.logicalProcessors} logical processors; performance cores ${machine.performanceCores?.join(', ') ?? 'n/a'}`,
    `- Memory: ${fixed((windows.ramBytes ?? machine.totalMemoryBytes) / 2 ** 30)} GiB`,
    `- OS: ${windows.os ?? machine.platform} ${windows.osVersion ?? machine.release} (build ${windows.osBuild ?? '?'})`,
    `- Power: ${windows.powerScheme ?? 'n/a'}; battery status ${windows.batteryStatus ?? 'n/a'} (2 = on mains power)`,
    pinning.affinitySet
      ? `- Pinning: every target under each of the masks ${pinning.affinitySet.map((entry) => `${entry.mask} (${entry.logicalProcessors.join(', ')})`).join(', ')} in turn, as LABEL@MASK; priority ${pinning.priority}, via ${pinning.how}`
      : `- Pinning: affinity ${pinning.affinityMask ?? 'none'} (logical processors ${pinning.logicalProcessors?.join(', ') ?? '-'}), priority ${pinning.priority}, via ${pinning.how}`,
    `- Node ${toolchain.node}, V8 ${toolchain.v8}, npm ${toolchain.npm ?? '?'}`,
    `- Rust: ${toolchain.rustc?.split('\n')[0] ?? '?'} (${toolchain.rustc?.split('\n').find((line) => line.startsWith('LLVM')) ?? ''}); ${toolchain.cargo ?? '?'}; ${toolchain.rustupToolchain ?? ''}`,
    `- ${toolchain.wasmBindgen ?? 'wasm-bindgen ?'}; wasm-opt ${toolchain.wasmOpt ?? 'not installed (the build does not use it)'}`,
    `- Load before: ${loadLine(loadBefore)}`,
    `- Load after: ${loadLine(loadAfter)}`,
    `- Load during: ${duringLine(report.environment.loadDuring)}`,
    '',
  ];
}

function targetsSection(report) {
  const lines = ['## Targets', '', '| Target | Kind | Revision | Reactive WebAssembly sha256 | Glue sha256 | Kit session.mjs sha256 | Native benchmark |', '| --- | --- | --- | --- | --- | --- | --- |'];
  for (const target of report.targets) {
    const revision = target.git ? `${target.git.describe} (${target.git.revision?.slice(0, 7)})${target.git.trackedClean ? '' : ', tracked changes'}`
      : target.packageInfo ? `${target.packageInfo.name}@${target.packageInfo.version}; built ${target.packageInfo.buildInfo?.revision?.slice(0, 7) ?? '?'} on ${target.packageInfo.buildInfo?.host ?? '?'}` : 'n/a';
    const wasm = target.runtimeFiles?.['caveat_runtime_bg.wasm'];
    lines.push(`| ${target.label} | ${target.kind} | ${revision} | ${short(wasm?.sha256)} (${wasm?.bytes ?? '-'} B${target.build?.wasm ? `, ${target.build.wasm}` : ''}) | ${short(target.runtimeFiles?.['caveat_runtime.js']?.sha256)} | ${short(target.kitFiles?.['session.mjs']?.sha256)} | ${target.native ? 'release, lto, 1 codegen unit, reactive-only' : '-'} |`);
  }
  lines.push('');
  return lines;
}

function methodSection(report) {
  const { suite } = report;
  const lines = ['## Method', ''];
  if (suite.smoke || suite.name === 'smoke') lines.push('**Smoke run: short streams and one round, to show the harness runs end to end. These numbers are not measurements.**', '');
  lines.push(
    `- Suite \`${suite.name}\`: ${suite.repeats} repeat(s) of every mode, each in a fresh process; within a repeat every target runs a mode back to back, the first target rotating.`,
    `- Per-event modes: ${suite.warmup} untimed warm-up pass(es), then ${suite.rounds} timed pass(es), each episode on a freshly opened session. \`published-method\` runs ${suite.publishedRounds} round(s) with no warm-up, as the published harness did.`,
    `- Load, save and restore: ${suite.lifecycleWarmup} untimed then ${suite.lifecycleSamples} timed calls; published resume method: ${suite.resumeRuns} run(s).`,
    '- Statistic: quantile q(p) = sorted[floor(p·(n−1))] over the pooled samples of a run (the published rule; the median is the lower median). A cell is the median over runs of each run\'s pooled median, with the lowest and highest run in brackets, then the median over runs of each run\'s p95.',
    '- Units: microseconds per operation unless the operation says otherwise.',
    '',
  );
  return lines;
}

function publishedSection(report) {
  const { dispatchPlusView: event, resume } = report.published;
  return [
    '## Published reference',
    '',
    `- Dispatch + view, Glowcap replay: median ${event.medianUs} µs, p95 ${event.p95Us} µs, max ${event.maxUs} µs (${event.evidence}); runtime ${event.runtimeRevision.slice(0, 7)}, reactive WebAssembly \`${event.reactiveWasmSha256.slice(0, 12)}\`, repository ${event.repositoryRevision.slice(0, 7)}.`,
    `- Resume ten minutes of play: median ${resume.medianMs} ms of runs ${resume.samplesMs.join(', ')} ms; save ${resume.saveBytes} bytes (${resume.evidence}).`,
    '',
  ];
}

function consistencySection(report) {
  const { inconsistent, betweenTargets, byWorkload } = report.consistency;
  const lines = ['## Final-state consistency', ''];
  const checked = Object.values(byWorkload).reduce((sum, hashes) => sum + Object.values(hashes).reduce((n, list) => n + list.length, 0), 0);
  if (!inconsistent.length) lines.push(`Within each target, every run of every mode and engine ended each workload in the same saved state (${checked} runs compared).`);
  else lines.push(...inconsistent.map((key) => `- **CORRECTNESS: different final states for ${key}.** See results.json consistency.byWorkload.`));
  lines.push('', 'Between targets, by workload and kind of state (targets in one group ended in identical states):', '');
  for (const [key, groups] of Object.entries(betweenTargets ?? {})) {
    lines.push(`- ${key}: ${groups.map((group) => `${group.targets.join(', ')} ${short(group.sha256)}`).join(' | ')}`);
  }
  for (const finding of report.correctness ?? []) lines.push(`- **${finding.note}** (${finding.job})`);
  if (report.failures.length) lines.push('', '**Failed jobs:**', ...report.failures.map((failure) => `- ${failure.job}: ${failure.detail.split('\n')[0]}`));
  if (report.skipped?.length) lines.push('', 'Skipped jobs (the target cannot run them):', ...report.skipped.map((skip) => `- ${skip.job}: ${skip.reason}`));
  lines.push('');
  return lines;
}

function workloadSection(report, workload) {
  const labels = report.targets.map((target) => target.label);
  const lines = [
    `## ${workload.id}`,
    '',
    `${workload.description} Program \`${workload.program.path}\`${workload.program.addMushrooms ? ` + ${workload.program.addMushrooms} mushrooms` : ''} (${short(workload.program.sha256)}), stream ${short(workload.streamSha256)}, ${workload.episodes} episode(s) × ${workload.repeats} = ${workload.eventsPerPass} events per pass${workload.truncatedTo ? `, truncated to ${workload.truncatedTo} events per episode` : ''}.`,
    '',
    `| Engine | Mode | Operation | ${labels.join(' | ')} |`,
    `| --- | --- | --- | ${labels.map(() => '---').join(' | ')} |`,
  ];
  const rows = new Map();
  for (const label of labels) {
    for (const [engine, byMode] of Object.entries(report.results[label]?.[workload.id] ?? {})) {
      for (const [mode, { acrossRuns }] of Object.entries(byMode)) {
        for (const op of Object.keys(acrossRuns)) rows.set(`${engine}\u0000${mode}\u0000${op}`, [engine, mode, op]);
      }
    }
  }
  const order = (engine) => (engine === 'native' ? 0 : 1);
  for (const [engine, mode, op] of [...rows.values()].sort((a, b) => order(a[0]) - order(b[0]))) {
    lines.push(`| ${engine} | ${mode} | ${op} | ${labels.map((label) => cell(opStats(report, label, workload.id, engine, mode, op))).join(' | ')} |`);
  }
  lines.push('');
  if (workload.segments?.length) {
    const segments = workload.segments.map((segment) => segment.name);
    lines.push(`Segment medians (events ${workload.segments.map((s) => `${s.name} ${s.from}–${s.to - 1}`).join(', ')}):`, '');
    lines.push(`| Engine | Mode | Operation | Target | ${segments.join(' | ')} |`, `| --- | --- | --- | --- | ${segments.map(() => '---').join(' | ')} |`);
    for (const [engine, mode, op] of [...rows.values()].sort((a, b) => order(a[0]) - order(b[0]))) {
      for (const label of labels) {
        const stats = opStats(report, label, workload.id, engine, mode, op);
        if (!stats || !Object.keys(stats.segments ?? {}).length) continue;
        lines.push(`| ${engine} | ${mode} | ${op} | ${label} | ${segments.map((segment) => segmentCell(stats, segment)).join(' | ')} |`);
      }
    }
    lines.push('');
  }
  return lines;
}

// One Glowcap event + view, piece by piece, on the steady idle ticks that
// make up the pooled median. Measured rows come from different processes
// (one mode each), so their sum need not equal the whole exactly.
function decompositionSection(report) {
  const workload = 'glowcap-replay';
  if (!report.workloads.some((w) => w.id === workload)) return [];
  const labels = report.targets.map((target) => target.label);
  const segment = report.workloads.find((w) => w.id === workload).segments?.find((s) => s.name === 'steady');
  const steady = (label, engine, mode, op, from = workload) => (segment
    ? opStats(report, label, from, engine, mode, op)?.segments?.steady?.median
    : opStats(report, label, from, engine, mode, op)?.median?.median);
  const rows = [
    ['Published method: adapter dispatch + view', 'wasm', 'published-method', 'adapter.dispatch+view'],
    ['Adapter dispatch (stringify, WebAssembly dispatch_view, JSON.parse)', 'wasm', 'adapter', 'adapter.dispatch'],
    ['Adapter view() (JavaScript reshaping only)', 'wasm', 'adapter', 'adapter.view'],
    ['JSON.stringify(payload)', 'wasm', 'raw.dispatch_view', 'js.stringify_payload'],
    ['wasm-bindgen dispatch_view call, returning the view text', 'wasm', 'raw.dispatch_view', 'raw.dispatch_view'],
    ['  arguments copied into WebAssembly memory', 'wasm', 'abi.dispatch_view', 'abi.encode_args'],
    ['  WebAssembly execution (resolve, apply, view, serialize)', 'wasm', 'abi.dispatch_view', 'abi.exec'],
    ['  result decoded to a JavaScript string', 'wasm', 'abi.dispatch_view', 'abi.decode'],
    ['  result freed', 'wasm', 'abi.dispatch_view', 'abi.free'],
    ['  the four pieces, timed together in one process', 'wasm', 'abi.dispatch_view', 'abi.dispatch_view'],
    ['JSON.parse(view text)', 'wasm', 'raw.dispatch_view', 'js.parse_view'],
    ['WebAssembly view() alone, execution only', 'wasm', 'abi.read', 'abi.view.exec'],
    ['Kit dispatch (dispatch_outcome with the full snapshot, parsed)', 'wasm', 'kit', 'kit.dispatch'],
    ['Kit dispatch + view()', 'wasm', 'kit', 'kit.dispatch+view'],
    ['Native web::dispatch_view (the exported function, natively)', 'native', 'web.dispatch_view', 'web.dispatch_view'],
    ['Native apply (numeric parameters: rules and bindings only)', 'native', 'apply', 'apply'],
    ['Native dispatch_view_json (resolve + apply + view, not serialized)', 'native', 'dispatch_view_json', 'dispatch_view_json'],
    ['Native view() build', 'native', 'read', 'view'],
    ['Native view serialize', 'native', 'read', 'view.serialize'],
    ['Native snapshot() build', 'native', 'read', 'snapshot'],
    ['Native session clone (upper bound of the transaction copy)', 'native', 'read', 'clone'],
    ['Native apply, the same program without bindings (glowcap-unbound)', 'native', 'apply', 'apply', 'glowcap-unbound'],
  ];
  const lines = [
    '## One Glowcap event + view, piece by piece',
    '',
    `${segment ? `Medians over runs of each run's median on the steady idle ticks (events ${segment.from}–${segment.to - 1}), where the pooled median sits` : 'Pooled medians (this run has no steady segment)'}; each row is its own process, so parts need not sum exactly to the whole.`,
    '',
    `| Piece | ${labels.join(' | ')} |`,
    `| --- | ${labels.map(() => '---').join(' | ')} |`,
  ];
  const value = {};
  for (const [name, engine, mode, op, from = workload] of rows) {
    const values = labels.map((label) => steady(label, engine, mode, op, from));
    value[`${from} ${mode} ${op}`] = values;
    lines.push(`| ${name} | ${values.map((v) => fixed(v)).join(' | ')} |`);
  }
  const derived = (name, fn) => lines.push(`| *derived:* ${name} | ${labels.map((_, index) => fixed(fn(index))).join(' | ')} |`);
  const get = (key, index, from = workload) => value[`${from} ${key}`]?.[index];
  derived('native resolve_payload ≈ dispatch_view_json − apply − view', (i) => get('dispatch_view_json dispatch_view_json', i) - get('apply apply', i) - get('read view', i));
  derived('native binding evaluation ≈ apply − apply without bindings', (i) => get('apply apply', i) - get('apply apply', i, 'glowcap-unbound'));
  derived('WebAssembly ÷ native, same exported function (ratio)', (i) => get('abi.dispatch_view abi.exec', i) / get('web.dispatch_view web.dispatch_view', i));
  lines.push('');
  return lines;
}

export function renderSummary(report) {
  const lines = [
    `# Caveat performance baseline: ${report.runId}`,
    '',
    `Written by \`experiments/performance-0.1/run.mjs\` from results.json in this directory. Command: \`${report.command}\``,
    '',
    ...methodSection(report),
    ...targetsSection(report),
    ...environmentSection(report),
    ...publishedSection(report),
    ...consistencySection(report),
    ...decompositionSection(report),
    ...report.workloads.flatMap((workload) => workloadSection(report, workload)),
  ];
  return `${lines.join('\n')}\n`;
}
