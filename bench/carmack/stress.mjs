#!/usr/bin/env node
// Carmack stress cases: the worst inputs we know of for the runtime's own
// limits. Times are reported, never gated. What must hold is that each case
// ends in an accepted event or a refusal with its documented code, never a
// thrown error or a fatal session; any other ending exits 1.
//
//   node bench/carmack/stress.mjs [--runtime DIR] [--out DIR]
//
// 1. Withdrawal reach: K decisions all rest on one piece of evidence, and one
//    event withdraws it and reopens all K. The retract event is timed for
//    several K, so the growth with K shows (K is bounded by the history
//    capacity).
// 2. Lineage growth: every event adds two renewable occurrences to one
//    state's lineage until the provenance limit refuses (provenance_limit).
// 3. Evaluation limit: a heavy reducer folded over a growing stream until the
//    evaluation limit refuses (evaluation_limit).
//
// The programs are generated here rather than kept as .cav files, so they
// stay out of the corpus sweeps. Writes stress.json and stress.md to --out
// (default test-results/carmack).
import { mkdirSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { loadRuntimeFromDirectory } from '../../kit/lib/node.mjs';
import { ROOT } from './workloads.mjs';

const arg = (name) => { const i = process.argv.indexOf(`--${name}`); return i < 0 ? undefined : process.argv[i + 1]; };
const out = path.resolve(arg('out') ?? path.join(ROOT, 'test-results', 'carmack'));
mkdirSync(out, { recursive: true });
const runtime = arg('runtime') ? await loadRuntimeFromDirectory(path.resolve(arg('runtime'))) : await loadRuntimeFromDirectory();

const now = () => process.hrtime.bigint();
const ms = (ns) => Number(ns) / 1e6;
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor((values.length - 1) / 2)];
const round = (n) => Number(n.toFixed(2));
const failures = [];

function reachSource(k) {
  const lines = [
    'claim c;', 'claim wrong;',
    'evidence e from "the shared source";', 'evidence r from "the recheck";',
    'event decide;', 'event retract;',
    'on decide reveal e supports c;',
  ];
  for (let i = 0; i < k; i++) {
    lines.push(`decisions d${i} limit 2;`, `state s${i} = 0;`, `bind v${i}.x = s${i};`,
      `on decide set s${i} = qualified(${i + 1}, e);`, `on decide commit d${i} because enough using s${i};`);
  }
  lines.push('on retract reveal r supports wrong;', 'on retract withdraw e because r;');
  for (let i = 0; i < k; i++) {
    lines.push(`on retract when committed(d${i}) and not reopened(d${i}) and rests_on_withdrawn(d${i}) reopen d${i} because r;`);
  }
  return lines.join('\n') + '\n';
}

const GROW = `claim c;
evidence a from "a source";
evidence b from "b source";
renewable a limit 1024;
renewable b limit 1024;
state acc = 0 min 0 max 1000000;
event step;
on step renew a;
on step renew b;
on step reveal a supports c;
on step reveal b supports c;
on step set acc = acc + qualified(1, a) + qualified(1, b);
`;

const FOLD = `claim c;
evidence probe from "a probe";
readings temps from probe limit 256;
fn f(r) = ${Array(40).fill('r').join(' + ')};
fn g(r) = ${Array(8).fill('f(r)').join(' + ')};
fn heavy(total, reading) = total + g(reading);
state s = 0 min -1000000000000 max 1000000000000;
event read v min 0 max 1;
on read sample temps = v supports c;
on read set s = fold_history(temps, 0, heavy);
`;

// Dispatch one event; a thrown error is recorded as a failure and ends the case.
function dispatch(session, name, event, payload) {
  try { return session.dispatchView(event, payload); } catch (error) {
    failures.push(`${name}: ${event} threw ${error.kind ?? ''} ${String(error.message).slice(0, 200)}`);
    return null;
  }
}

// 1. Withdrawal reach.
const reach = [];
for (const k of [100, 200, 300, 400, 500]) {
  const source = reachSource(k);
  const times = [];
  let reopened = null;
  for (let rep = 0; rep < 3; rep++) {
    const session = runtime.open(source);
    const decided = dispatch(session, `reach K=${k}`, 'decide', {});
    if (!decided) break;
    if (decided.outcome !== 'accepted') { failures.push(`reach K=${k}: decide ${decided.outcome} ${decided.code}`); session.close(); break; }
    const start = now();
    const retracted = dispatch(session, `reach K=${k}`, 'retract', {});
    times.push(ms(now() - start));
    if (!retracted) break;
    if (retracted.outcome !== 'accepted') { failures.push(`reach K=${k}: retract ${retracted.outcome} ${retracted.code}`); session.close(); break; }
    reopened = retracted.view.commitments.filter(c => c.open).length;
    if (reopened !== k) failures.push(`reach K=${k}: reopened ${reopened}, expected ${k}`);
    session.close();
  }
  reach.push({ k, retract_ms: times.length ? round(median(times)) : null, reopened, source_bytes: source.length });
}

// Run events until the first refusal, timing each band of `band` events.
function untilRefused(name, source, event, payload, band, max, expectCode) {
  const session = runtime.open(source);
  const bands = [];
  let start = now();
  let ending = null;
  for (let i = 1; i <= max; i++) {
    const outcome = dispatch(session, name, event, payload);
    if (!outcome) { ending = { event: i, outcome: 'threw' }; break; }
    if (outcome.outcome !== 'accepted') { ending = { event: i, outcome: outcome.outcome, code: outcome.code ?? null }; break; }
    if (i % band === 0) {
      bands.push({ events: `${i - band + 1}-${i}`, us_per_event: round(ms(now() - start) * 1000 / band) });
      start = now();
    }
  }
  if (!ending) failures.push(`${name}: no refusal within ${max} events, expected ${expectCode}`);
  else if (ending.outcome !== 'rejected' || ending.code !== expectCode) {
    failures.push(`${name}: event ${ending.event} ended ${ending.outcome} ${ending.code ?? ''}, expected rejected ${expectCode}`);
  } else {
    // A refusal leaves the session usable: the same event is rejected again, the same way.
    const again = dispatch(session, name, event, payload);
    if (again && (again.outcome !== 'rejected' || again.code !== expectCode)) {
      failures.push(`${name}: after the refusal the next ${event} ended ${again.outcome} ${again.code ?? ''}`);
    }
  }
  session.close();
  return { bands, ending };
}

const growth = untilRefused('lineage growth', GROW, 'step', {}, 100, 600, 'provenance_limit');
const fold = untilRefused('evaluation limit', FOLD, 'read', { v: 1 }, 20, 256, 'evaluation_limit');

const report = {
  schema: 'caveat-carmack-stress/1',
  measured_at: new Date().toISOString(),
  host: { node: process.version, platform: process.platform, arch: process.arch, cpus: os.cpus()[0]?.model ?? null },
  reach,
  reach_growth: reach.length > 1 && reach[0].retract_ms
    ? round(reach.at(-1).retract_ms / reach[0].retract_ms) : null,
  lineage_growth: growth,
  evaluation_limit: fold,
  failures,
};
writeFileSync(path.join(out, 'stress.json'), JSON.stringify(report, null, 2) + '\n');

const md = ['## Stress cases', ''];
md.push('Withdrawal reach (one event withdraws the evidence under K decisions and reopens them all; median of 3):', '');
md.push('| K | retract event (ms) | reopened |', '| ---: | ---: | ---: |');
for (const row of reach) md.push(`| ${row.k} | ${row.retract_ms ?? 'n/a'} | ${row.reopened ?? 'n/a'} |`);
if (report.reach_growth) {
  const ratio = reach.at(-1).k / reach[0].k;
  md.push('', `From K=${reach[0].k} to K=${reach.at(-1).k} (${ratio}x) the retract event took ${report.reach_growth}x as long (linear would be ${ratio}x, quadratic ${ratio * ratio}x).`);
}
for (const [title, result] of [['Lineage growth (two names added per event)', growth], ['Evaluation limit (heavy reducer over a growing stream)', fold]]) {
  md.push('', `${title}:`, '', '| events | µs per event |', '| --- | ---: |');
  for (const band of result.bands) md.push(`| ${band.events} | ${band.us_per_event} |`);
  if (result.ending) md.push('', `Ended at event ${result.ending.event}: ${result.ending.outcome}${result.ending.code ? ` (${result.ending.code})` : ''}.`);
}
md.push('', failures.length ? `**Failures:**\n\n${failures.map(f => `- ${f}`).join('\n')}` : 'Every case ended accepted or with its documented refusal.');
writeFileSync(path.join(out, 'stress.md'), md.join('\n') + '\n');
console.log(md.join('\n'));
process.exit(failures.length ? 1 : 0);
