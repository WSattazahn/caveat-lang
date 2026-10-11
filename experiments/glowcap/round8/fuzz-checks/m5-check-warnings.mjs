// Post-hoc, not registered: runs `caveat check --json` on each Caveat
// author's final program and on every Caveat M5 mutant of it, and records the
// warning codes. mutate.mjs counted a static catch only when a program failed
// to load (exit status 2), so it stored the number of warnings
// (`checkWarnings` in results/mutation.json) without counting them as
// catches. This shows which warnings those were. It changes no score.
//
//   node fuzz-checks/m5-check-warnings.mjs --caveat=DIR > results/m5-check-warnings.json
//
// DIR is an installed caveat-lang (for round 8, the 0.1.0-rc.17 tarball that
// prepare.mjs checks), for example node_modules/caveat-lang.
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFileSync, mkdirSync, mkdtempSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const round = path.join(here, '..');
const caveat = process.argv.find((a) => a.startsWith('--caveat='))?.slice(9);
if (!caveat) throw new Error('pass --caveat=DIR, an installed caveat-lang');
const cli = path.join(path.resolve(caveat), 'bin', 'caveat.mjs');
const version = execFileSync(process.execPath, [cli, '--version'], { encoding: 'utf8' }).trim();

function check(file) {
  const r = spawnSync(process.execPath, [cli, 'check', '--json', file], { encoding: 'utf8' });
  if (r.status === 2) return { status: 2, codes: [] };
  const codes = JSON.parse(r.stdout).diagnostics.map((d) => d.code);
  return { status: r.status, codes };
}

const tally = (codes) => Object.fromEntries([...new Set(codes)].sort().map((c) => [c, codes.filter((x) => x === c).length]));
const work = mkdtempSync(path.join(tmpdir(), 'm5-check-'));
const programs = [];
const mutants = [];
for (const program of ['C1', 'C2', 'C3']) {
  const original = path.join(round, 'runs', program, 'impl', 'glowcap.cav');
  const base = check(original);
  programs.push({ program, status: base.status, warnings: tally(base.codes) });
  const dir = path.join(round, 'mutation', 'mutants', program);
  for (const site of readdirSync(dir).filter((n) => n.startsWith('M5-'))) {
    const at = path.join(work, program, site);
    mkdirSync(at, { recursive: true });
    copyFileSync(original, path.join(at, 'glowcap.cav'));
    execFileSync('patch', ['-s', '-p1', '-d', at, '-i', path.join(dir, site, 'mutant.diff')]);
    const r = check(path.join(at, 'glowcap.cav'));
    mutants.push({ program, mutant: site, status: r.status, warnings: tally(r.codes) });
  }
}
rmSync(work, { recursive: true, force: true });

const summary = {};
for (const { program, status, warnings } of mutants) {
  const s = (summary[program] ??= { mutants: 0, loads: 0, withC006: 0, otherCodes: [] });
  s.mutants += 1;
  if (status !== 2) s.loads += 1;
  if (warnings.C006) s.withC006 += 1;
  for (const code of Object.keys(warnings)) if (code !== 'C006' && !s.otherCodes.includes(code)) s.otherCodes.push(code);
}
console.log(JSON.stringify({ note: 'post hoc, not registered; changes no score', caveat: version, programs, summary, mutants }, null, 2));
