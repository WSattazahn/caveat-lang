// Checks the Lean theorems that Caveat evidence names (docs/design/lean-bridge.md).
// For each entry of a lean-grounds.json manifest: the source compiles, the
// theorem exists, it depends on no axiom beyond Lean's standard three, and its
// statement still hashes to the recorded SHA-256. Writes the events a host
// sends the Caveat program: the entry's on_pass event, or its on_fail event
// with the reason.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const allowedAxioms = new Set(['propext', 'Classical.choice', 'Quot.sound']);

function leanEnv(extra = {}) {
  const env = { ...process.env, ...extra };
  const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') ?? 'PATH';
  if (env.CAVEAT_LEAN_BIN) env[pathKey] = env.CAVEAT_LEAN_BIN + delimiter + (env[pathKey] ?? '');
  return env;
}

function run(command, args, env) {
  const result = spawnSync(command, args, { encoding: 'utf8', env });
  if (result.error) throw result.error;
  return { status: result.status, output: (result.stdout ?? '') + (result.stderr ?? '') };
}

export function statementHash(type) {
  return createHash('sha256').update(type).digest('hex');
}

/** The evidence label a program must give a Lean ground: `lean:THEOREM`. */
export function declaresGround(program, evidence, theorem) {
  const pattern = new RegExp(`^\\s*evidence\\s+${evidence}\\s+from\\s+"lean:${theorem.replaceAll('.', '\\.')}"\\s*;`, 'm');
  return pattern.test(program);
}

/** Reads the audit's CAVEAT_GROUND line and #print axioms line for one theorem. */
export function parseAudit(output, theorem) {
  const line = output.split('\n').find(text => text.startsWith('CAVEAT_GROUND '));
  if (!line) return { found: false };
  const found = JSON.parse(line.slice('CAVEAT_GROUND '.length));
  if (!found.found) return { found: false };
  const axiomsMatch = new RegExp(`'${theorem.replaceAll('.', '\\.')}' (?:depends on axioms:\\s*\\[([^\\]]*)\\]|does not depend on any axioms)`).exec(output);
  if (!axiomsMatch) throw new Error('No axiom report for ' + theorem);
  const axioms = (axiomsMatch[1] ?? '').split(',').map(value => value.trim()).filter(Boolean);
  return { found: true, type: found.type, axioms };
}

/** Checks one manifest entry against a source file. Returns { ok, reason, ... }. */
export function checkGround(entry, sourcePath) {
  const work = mkdtempSync(join(tmpdir(), 'lean-grounds-'));
  try {
    const module = basename(sourcePath, '.lean');
    const olean = join(work, module + '.olean');
    const compiled = run('lean', ['--root=' + dirname(sourcePath), '-o', olean, sourcePath], leanEnv());
    if (compiled.status !== 0) return { ok: false, reason: 'broken', detail: compiled.output.trim() };
    const prefix = run('lean', ['--print-prefix'], leanEnv()).output.trim();
    const audit = join(work, 'Audit.lean');
    writeFileSync(audit, [
      'import Lean',
      `import ${module}`,
      'open Lean Elab Command',
      '#eval show CommandElabM Unit from do',
      `  match (← getEnv).find? \`${entry.theorem} with`,
      '  | none => IO.println "CAVEAT_GROUND {\\"found\\":false}"',
      '  | some info => IO.println ("CAVEAT_GROUND " ++ (Json.mkObj [("found", true), ("type", toString info.type)]).compress)',
      `#print axioms ${entry.theorem}`,
      '',
    ].join('\n'));
    const leanPath = [work, join(prefix, 'lib', 'lean')].join(delimiter);
    const audited = run('lean', [audit], leanEnv({ LEAN_PATH: leanPath }));
    if (audited.status !== 0) {
      return { ok: false, reason: 'missing', detail: audited.output.trim() };
    }
    const result = parseAudit(audited.output, entry.theorem);
    if (!result.found) return { ok: false, reason: 'missing', detail: entry.theorem + ' is not declared' };
    const extra = result.axioms.filter(axiom => !allowedAxioms.has(axiom));
    if (extra.length > 0) return { ok: false, reason: 'broken', detail: 'unapproved axioms: ' + extra.join(', '), axioms: result.axioms };
    const hash = statementHash(result.type);
    if (hash !== entry.statement_sha256) {
      return { ok: false, reason: 'changed', detail: `statement hash ${hash}, recorded ${entry.statement_sha256}`, type: result.type };
    }
    return { ok: true, axioms: result.axioms, statement_sha256: hash, type: result.type };
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

/** The event a host sends the program for one checked entry. */
export function eventFor(entry, result) {
  return result.ok ? { event: entry.on_pass } : { event: entry.on_fail, payload: { why: result.reason } };
}

export function checkManifest(manifestPath) {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  if (manifest.schema !== 'caveat-lean-grounds/0.1') throw new Error('Unknown manifest schema: ' + manifest.schema);
  const base = dirname(manifestPath);
  const program = readFileSync(join(base, manifest.program), 'utf8');
  return manifest.grounds.map(entry => {
    if (!declaresGround(program, entry.evidence, entry.theorem)) {
      throw new Error(`${manifest.program} must declare: evidence ${entry.evidence} from "lean:${entry.theorem}";`);
    }
    return { entry, result: checkGround(entry, join(base, entry.source)) };
  });
}

/** Each control breaks the demo theorem one way and must be refused for that reason. */
export const controls = [
  { id: 'sorry', expect: 'broken', find: /:= by\n[\s\S]*?\n\nend/, replace: ':= by\n  sorry\n\nend' },
  { id: 'statement', expect: 'changed', find: 'n < 2 ^ n', replace: 'n ≤ 2 ^ n' },
  { id: 'renamed', expect: 'missing', find: 'theorem lt_two_pow', replace: 'theorem lt_two_pow_renamed' },
  { id: 'false_proof', expect: 'broken', find: 'omega', replace: 'rfl' },
];

export function runControls(manifestPath) {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  const entry = manifest.grounds[0];
  const original = readFileSync(join(dirname(manifestPath), entry.source), 'utf8');
  return controls.map(control => {
    const mutated = original.replace(control.find, control.replace);
    if (mutated === original) throw new Error('Control anchor not found: ' + control.id);
    const work = mkdtempSync(join(tmpdir(), 'lean-grounds-control-'));
    try {
      const source = join(work, basename(entry.source));
      writeFileSync(source, mutated);
      const result = checkGround(entry, source);
      return { id: control.id, expect: control.expect, got: result.ok ? 'passed' : result.reason, refused: !result.ok && result.reason === control.expect };
    } finally {
      rmSync(work, { recursive: true, force: true });
    }
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const manifestPath = resolve(args.find(arg => !arg.startsWith('--')) ?? join(root, 'examples', 'lean-bridge', 'lean-grounds.json'));
  const out = join(root, 'test-results', 'lean-grounds');
  mkdirSync(out, { recursive: true });
  const checked = checkManifest(manifestPath);
  const events = checked.map(({ entry, result }) => eventFor(entry, result));
  writeFileSync(join(out, 'events.jsonl'), events.map(event => JSON.stringify(event)).join('\n') + '\n');
  const controlResults = args.includes('--controls') ? runControls(manifestPath) : [];
  const report = { manifest: manifestPath.slice(root.length + 1), checked, events, controls: controlResults };
  writeFileSync(join(out, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  for (const { entry, result } of checked) {
    console.log(`${result.ok ? 'ok    ' : 'FAILED'} ${entry.evidence} <- ${entry.theorem}${result.ok ? '' : ` (${result.reason}: ${result.detail})`}`);
  }
  for (const control of controlResults) {
    console.log(`${control.refused ? 'ok    ' : 'FAILED'} control ${control.id}: expected ${control.expect}, got ${control.got}`);
  }
  const failed = checked.some(({ result }) => !result.ok) || controlResults.some(control => !control.refused);
  process.exitCode = failed ? 1 : 0;
}
