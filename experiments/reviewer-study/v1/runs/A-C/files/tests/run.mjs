// Runs every history in histories/ and cases/ (and tests/extra/) through gate.cav
// in the package's own session, and compares each line's outcome, the view,
// and every decision's grounds, caveats, grant and reopening with tests/model.mjs.
// Usage: node tests/run.mjs [--verbose]
import { readFile, readdir } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { model } from './model.mjs';

const root = new URL('../', import.meta.url);
const verbose = process.argv.includes('--verbose');
const source = await readFile(new URL('gate.cav', root), 'utf8');
const runtime = await loadRuntimeFromDirectory();
const files = [];
for (const dir of ['histories', 'cases', 'tests/extra']) {
  let names = [];
  try { names = (await readdir(new URL(dir + '/', root))).filter((n) => n.endsWith('.jsonl')).sort(); } catch {}
  for (const n of names) files.push(`${dir}/${n}`);
}
let failures = 0;
for (const f of files) {
  const lines = (await readFile(new URL(f, root), 'utf8')).split('\n').filter((l) => l.trim()).map((l) => JSON.parse(l));
  const expected = model(lines);
  const session = runtime.open(source);
  const idToLine = {};
  const counts = {};
  const name = (base, k) => (k === 1 ? base : `${base}@${k}`);
  const problems = [];
  let seq = 0;
  lines.forEach((env, i) => {
    const line = i + 1;
    const out = session.dispatch(env.event, env.payload ?? {});
    if (out.outcome !== 'accepted' && out.outcome !== 'rejected') throw new Error(`${f}:${line} fatal ${JSON.stringify(out)}`);
    const accepted = out.outcome === 'accepted';
    if (accepted) {
      seq++;
      const ids = { check: (k) => `checks@${k}`, review: (k) => name('reviewer', k), scan: (k) => name('scanner', k),
        approve: (k) => `grants@${k}`, recheck: (k) => name('requeue', k) };
      if (ids[env.event]) { counts[env.event] = (counts[env.event] ?? 0) + 1; idToLine[ids[env.event](counts[env.event])] = line; }
    }
    const b = session.view().bindings.gate;
    const view = { status: b.status, head: b.head, revisions: b.revisions, reason: b.reason };
    const exp = expected.steps[i];
    const tag = accepted ? 'accepted' : `refused (${out.origin}/${out.code}: ${out.message})`;
    if (verbose) console.log(`  ${f}:${line} ${env.event} ${JSON.stringify(env.payload ?? {})} -> ${tag} ${JSON.stringify(view)}`);
    if (accepted !== exp.accepted) problems.push(`line ${line}: runtime ${tag}, model ${exp.accepted ? 'accepted' : 'refused: ' + exp.why}`);
    if (JSON.stringify(view) !== JSON.stringify(exp.view)) problems.push(`line ${line}: view ${JSON.stringify(view)} vs model ${JSON.stringify(exp.view)}`);
  });
  const snap = session.snapshot();
  if (snap.sequence !== seq) problems.push(`sequence ${snap.sequence} vs accepted ${seq}`);
  // Decisions: grounds, caveats, grant, value, reopening.
  const journal = snap.decision_journal;
  expected.decisions.forEach((d, j) => {
    const id = `merge@${j + 1}`;
    const g = snap.commitment_grounds[id];
    if (!g) { problems.push(`${id} missing`); return; }
    const rests = g.evidence.map((e) => idToLine[e]).sort((a, b) => a - b);
    if (JSON.stringify(rests) !== JSON.stringify(d.rests)) problems.push(`${id} rests on lines ${rests} (${g.evidence}) vs model ${d.rests}`);
    if (JSON.stringify([...g.caveats].sort()) !== JSON.stringify(d.caveats)) problems.push(`${id} caveats ${g.caveats} vs model ${d.caveats}`);
    const perm = snap.commitment_permissions[id];
    if (!perm || idToLine[perm.grant] !== d.grant) problems.push(`${id} grant ${JSON.stringify(perm)} vs model line ${d.grant}`);
    if (snap.commitment_bases[id].value !== d.value) problems.push(`${id} value ${snap.commitment_bases[id].value} vs ${d.value}`);
    const re = journal.find((e) => e.commitment === id && e.change === 'reopened');
    if (!!re !== !!d.reopened) problems.push(`${id} reopened ${!!re} vs model ${!!d.reopened}`);
    else if (re) {
      const cited = re.because.map((e) => idToLine[e]);
      if (cited.length !== 1 || cited[0] !== d.reopened.line) problems.push(`${id} reopened because ${re.because} (lines ${cited}) vs model line ${d.reopened.line}`);
    }
    if (verbose) console.log(`  ${id} = ${d.value} rests on ${g.evidence.map((e) => `${e}=#${idToLine[e]}`).join(', ')} caveats [${g.caveats}] permitted by ${perm?.grant}=#${idToLine[perm?.grant]}${re ? `; reopened because ${re.because.map((e) => `${e}=#${idToLine[e]}`)}` : ''}`);
  });
  if (Object.keys(snap.commitment_grounds).length !== expected.decisions.length) problems.push(`decision count ${Object.keys(snap.commitment_grounds).length} vs ${expected.decisions.length}`);
  // The save restores and agrees.
  const resumed = runtime.restore(source, session.save());
  if (JSON.stringify(resumed.view().bindings) !== JSON.stringify(session.view().bindings)) problems.push('restored view differs');
  resumed.close();
  session.close();
  failures += problems.length ? 1 : 0;
  console.log(`${problems.length ? 'FAIL' : 'PASS'} ${f}`);
  for (const p of problems) console.log('     ' + p);
}
console.log(`${files.length - failures} passed, ${failures} failed`);
process.exit(failures ? 1 : 0);
