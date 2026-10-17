// Corpus sweep for Neutral Sample 0.1 (spec/caveat-neutral-sample-0.1.md,
// "Corpus sweep"). The profile lets `sample STREAM = EXPRESSION` record a
// reading without a stance, a form that refuses to load today. This reads
// every tracked Caveat program and reports:
//   - every `sample` effect, and every one without `supports|opposes CLAIM`
//     (there must be none: such a program does not load today);
//   - the claims samples point at, and those only samples point at (no
//     declaration, reveal or other statement relates evidence to them): a
//     claim a program may have declared only to give its readings something to
//     bear on, as in issue #218. Source text cannot say why a claim exists;
//     this counts candidates, not mistakes;
//   - `reopened by STREAM supporting|opposing CLAIM` filters, which a neutral
//     reading never matches.
// It reads source text only; it runs no program. Comments and text literals
// are removed before reading.
// Run from any directory: node experiments/neutral-sample-sweep/sweep.mjs [--write]
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');
const listed = spawnSync('git', ['ls-files', '-z', '--', '*.cav'], { cwd: root, encoding: 'utf8' });
assert.equal(listed.status, 0, listed.stderr);
const programs = listed.stdout.split('\0').filter(Boolean).sort();
assert.ok(programs.length > 0, 'the tracked Caveat corpus must not be empty');

// Text literals and `#` or `//` comments carry no source words.
function code(text) {
  let out = '';
  let inString = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (inString) {
      if (c === '\\') { i += 1; continue; }
      if (c === '"') { inString = false; out += '""'; }
      continue;
    }
    if (c === '"') { inString = true; continue; }
    if (c === '#' || (c === '/' && text[i + 1] === '/')) {
      while (i < text.length && text[i] !== '\n') i += 1;
      out += '\n';
      continue;
    }
    out += c;
  }
  return out;
}

const statements = text => text.split(/[;{}]/).map(s => s.replace(/\s+/g, ' ').trim()).filter(Boolean);
const NAME = '[A-Za-z_$][A-Za-z0-9_$]*';
const SAMPLE = new RegExp(`(?:^|\\s)sample (${NAME}) = (.*)$`);
const STANCE = new RegExp(`^(.*) (supports|opposes) (${NAME})$`);
const CLAIM = new RegExp(`^claim (${NAME})$`);
const OTHER_STANCE = new RegExp(`(?:supports|opposes) (${NAME})`, 'g');

const samples = [];
const neutral = [];
const filters = [];
const pointedAt = [];
const onlySamples = [];
for (const path of programs) {
  const text = code(readFileSync(join(root, path), 'utf8'));
  const claims = new Set();
  const targets = new Set();
  const otherTargets = new Set();
  for (const line of statements(text)) {
    const declared = line.match(CLAIM);
    if (declared) claims.add(declared[1]);
    const m = line.match(SAMPLE);
    if (m) {
      const stance = m[2].match(STANCE);
      const row = { path, stream: m[1], relation: stance?.[2] ?? null, claim: stance?.[3] ?? null };
      samples.push(row);
      if (!stance) neutral.push(row);
      else targets.add(stance[3]);
    } else {
      for (const edge of line.matchAll(OTHER_STANCE)) otherTargets.add(edge[1]);
    }
    for (const trigger of line.matchAll(new RegExp(`(${NAME}) (supporting|opposing) (${NAME})`, 'g'))) {
      if (/(?:^|\s)reopened by\s/.test(line)) filters.push({ path, stream: trigger[1], relation: trigger[2], claim: trigger[3] });
    }
  }
  for (const claim of [...targets].sort()) {
    if (!claims.has(claim)) continue;
    pointedAt.push({ path, claim });
    if (!otherTargets.has(claim)) onlySamples.push({ path, claim });
  }
}

const report = {
  note: 'Neutral Sample 0.1 corpus sweep. Source text only; no program is run. See spec/caveat-neutral-sample-0.1.md, "Corpus sweep".',
  revision: spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).stdout.trim(),
  programs: programs.length,
  sample_effects: samples.length,
  programs_with_sample: new Set(samples.map(row => row.path)).size,
  neutral_samples: neutral,
  claims_samples_point_at: pointedAt.length,
  claims_only_samples_point_at: onlySamples,
  reopening_filters: filters,
};
assert.deepEqual(neutral, [], 'a tracked program writes sample without a stance');
const out = JSON.stringify(report, null, 2) + '\n';
if (process.argv.includes('--write')) writeFileSync(join(here, 'RESULTS.json'), out);
console.log(`${programs.length} programs; ${samples.length} sample effects in ${report.programs_with_sample} programs; ` +
  `${neutral.length} without a stance; ${pointedAt.length} claims samples point at, ${onlySamples.length} only samples point at; ` +
  `${filters.length} reopening filters`);
