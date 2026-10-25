#!/usr/bin/env node
// Carmack check 1: where the time per event goes, by event kind, in the
// WebAssembly runtime a host ships, beside the native split into rules,
// record keeping and the transaction. Reports every number, however it looks;
// nothing here passes or fails.
//
//   node bench/carmack/profile.mjs [--runtime DIR] [--native BIN] [--out DIR]
//
// --native is the carmack_profile example built with the event-phase-profile
// feature; without it only the WebAssembly half is measured. Writes
// profile.json and profile.md to --out (default test-results/carmack).
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { loadRuntimeFromDirectory } from '../../kit/lib/node.mjs';
import { WORKLOADS, ROOT } from './workloads.mjs';

const arg = (name) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? undefined : process.argv[i + 1]; };
const out = path.resolve(arg('out') ?? path.join(ROOT, 'test-results', 'carmack'));
const native = arg('native');
mkdirSync(out, { recursive: true });

const runtime = arg('runtime') ? await loadRuntimeFromDirectory(path.resolve(arg('runtime'))) : await loadRuntimeFromDirectory();

const now = () => process.hrtime.bigint();
const us = (ns) => Number(ns) / 1000;
const quantile = (values, q) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(q * (sorted.length - 1)))] : 0;
};
const round = (n) => Number(n.toFixed(1));

// One pass of a workload; fields null is the whole view a host reads, [] is
// the dispatch alone (the view's header only).
function pass(workload, source, fields) {
  const options = fields === null ? {} : { fields };
  const byKind = {};
  const all = [];
  const save = { state: [], compact: [], restoreState: [], restoreCompact: [], stateBytes: 0, compactBytes: 0 };
  const load = [];
  for (let rep = 0; rep < workload.reps; rep += 1) {
    let t = now();
    const session = runtime.open(source);
    load.push(us(now() - t));
    for (const [event, payload] of workload.events) {
      t = now();
      const outcome = session.dispatchView(event, payload, options);
      const elapsed = us(now() - t);
      if (outcome.outcome !== 'accepted') throw new Error(`${workload.name}: ${event} ${JSON.stringify(payload)} was ${outcome.code}`);
      (byKind[event] ??= []).push(elapsed);
      all.push(elapsed);
    }
    t = now(); const state = session.save(); save.state.push(us(now() - t));
    t = now(); const compact = session.save({ compact: true, positional: true }); save.compact.push(us(now() - t));
    save.stateBytes = Buffer.byteLength(state);
    save.compactBytes = Buffer.byteLength(compact);
    session.close();
    t = now(); runtime.restore(source, state).close(); save.restoreState.push(us(now() - t));
    t = now(); runtime.restore(source, compact).close(); save.restoreCompact.push(us(now() - t));
  }
  return { byKind, all, save, load };
}

const report = {
  schema: 'caveat-carmack-profile/1',
  host: { node: process.version, platform: process.platform, arch: process.arch, cpu: os.cpus()[0]?.model ?? null },
  runtime: runtime.identity ?? null,
  workloads: {},
};

for (const workload of WORKLOADS) {
  const source = readFileSync(workload.program, 'utf8');
  pass({ ...workload, reps: Math.min(5, workload.reps) }, source, null); // warm the JIT and the allocator
  const full = pass(workload, source, null);
  const bare = pass(workload, source, []);
  const kinds = {};
  for (const [kind, samples] of Object.entries(full.byKind)) {
    const dispatch = quantile(bare.byKind[kind], 0.5);
    kinds[kind] = {
      events: samples.length,
      median_us: round(quantile(samples, 0.5)),
      p95_us: round(quantile(samples, 0.95)),
      dispatch_only_median_us: round(dispatch),
      view_median_us: round(Math.max(0, quantile(samples, 0.5) - dispatch)),
    };
  }
  const entry = {
    program: path.relative(ROOT, workload.program).split(path.sep).join('/'),
    events_per_rep: workload.events.length,
    reps: workload.reps,
    wasm: {
      event_median_us: round(quantile(full.all, 0.5)),
      event_p95_us: round(quantile(full.all, 0.95)),
      event_mean_us: round(full.all.reduce((a, b) => a + b, 0) / full.all.length),
      dispatch_only_median_us: round(quantile(bare.all, 0.5)),
      by_kind: kinds,
      load_median_ms: round(quantile(full.load, 0.5) / 1000),
      save: {
        state_bytes: full.save.stateBytes,
        compact_bytes: full.save.compactBytes,
        state_median_us: round(quantile(full.save.state, 0.5)),
        compact_median_us: round(quantile(full.save.compact, 0.5)),
        restore_state_median_ms: round(quantile(full.save.restoreState, 0.5) / 1000),
        restore_compact_median_ms: round(quantile(full.save.restoreCompact, 0.5) / 1000),
      },
    },
  };
  if (native) {
    const eventsFile = path.join(out, `${workload.name}.events.json`);
    writeFileSync(eventsFile, JSON.stringify(workload.events));
    const text = execFileSync(path.resolve(native), [workload.program, eventsFile, String(workload.reps)], { encoding: 'utf8', maxBuffer: 1 << 26 });
    const n = JSON.parse(text);
    const shares = (m) => ({
      rules_us: round(m.rules / 1000), record_us: round(m.record / 1000), transaction_us: round(m.transaction / 1000),
      unphased_us: round(m.unphased / 1000), view_json_us: round(m.view_json / 1000),
    });
    entry.native = {
      event_median_us: round(n.all.event_median_ns / 1000),
      event_p95_us: round(n.all.event_p95_ns / 1000),
      mean_per_event: shares(n.all.mean_ns),
      phases_mean_us: Object.fromEntries(Object.entries(n.all.mean_ns.phases).map(([k, v]) => [k, round(v / 1000)])),
      by_kind: Object.fromEntries(Object.entries(n.by_kind).map(([k, v]) => [k, { events: v.events, dispatch_median_us: round(v.dispatch_median_ns / 1000), ...shares(v.mean_ns) }])),
      load_median_ms: round(n.load_median_ns / 1e6),
      save: n.save,
      scope: n.scope,
    };
  }
  report.workloads[workload.name] = entry;
}

writeFileSync(path.join(out, 'profile.json'), `${JSON.stringify(report, null, 2)}\n`);

// The same numbers as a table a release record can quote.
const lines = ['# Where the time per event goes', '', `Node ${process.version}, ${process.platform}/${process.arch}, ${report.host.cpu ?? 'unknown CPU'}.`, ''];
for (const [name, w] of Object.entries(report.workloads)) {
  lines.push(`## ${name} (${w.program}, ${w.events_per_rep} events x ${w.reps})`, '');
  lines.push(`WebAssembly: median ${w.wasm.event_median_us} µs per event with the whole view, p95 ${w.wasm.event_p95_us} µs; dispatch alone ${w.wasm.dispatch_only_median_us} µs.`);
  lines.push(`Reopening the program (load cache warm) ${w.wasm.load_median_ms} ms. Save ${w.wasm.save.state_bytes} bytes (${w.wasm.save.state_median_us} µs), compact ${w.wasm.save.compact_bytes} bytes (${w.wasm.save.compact_median_us} µs). Restore ${w.wasm.save.restore_state_median_ms} ms, from compact ${w.wasm.save.restore_compact_median_ms} ms.`, '');
  if (w.native) {
    const m = w.native.mean_per_event;
    const total = m.rules_us + m.record_us + m.transaction_us + m.unphased_us + m.view_json_us;
    const pct = (x) => `${Math.round((100 * x) / total)}%`;
    lines.push(`Native: median ${w.native.event_median_us} µs. Mean per event: rules ${m.rules_us} µs (${pct(m.rules_us)}), record keeping ${m.record_us} µs (${pct(m.record_us)}), transaction ${m.transaction_us} µs (${pct(m.transaction_us)}), payload and view building ${m.unphased_us} µs (${pct(m.unphased_us)}), view JSON ${m.view_json_us} µs (${pct(m.view_json_us)}).`, '');
  }
  lines.push('| Event | Count | Wasm median µs | Wasm p95 µs | Dispatch alone µs | View µs |' + (w.native ? ' Native rules µs | Native record µs | Native transaction µs |' : ''));
  lines.push('| --- | ---: | ---: | ---: | ---: | ---: |' + (w.native ? ' ---: | ---: | ---: |' : ''));
  for (const [kind, k] of Object.entries(w.wasm.by_kind)) {
    const n = w.native?.by_kind[kind];
    lines.push(`| ${kind} | ${k.events} | ${k.median_us} | ${k.p95_us} | ${k.dispatch_only_median_us} | ${k.view_median_us} |` + (n ? ` ${n.rules_us} | ${n.record_us} | ${n.transaction_us} |` : ''));
  }
  lines.push('');
}
writeFileSync(path.join(out, 'profile.md'), `${lines.join('\n')}\n`);
process.stdout.write(`${lines.join('\n')}\n`);
