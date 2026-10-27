// Checks the Lean theorems that Caveat evidence names (docs/design/lean-bridge.md).
// For each entry of a lean-grounds.json manifest: the source compiles, the
// theorem exists, it depends on no axiom beyond Lean's standard three and the
// results the entry lists as assumed for now, and its statement and each
// assumed result's statement still hash to the recorded SHA-256. Writes the
// events a host sends the Caveat program: the entry's on_pass event (with the
// number of assumed results the proof still uses, when the entry lists any),
// or its on_fail event with the reason.
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

/** Reads the audit's CAVEAT_GROUND lines and the #print axioms line for one theorem. */
export function parseAudit(output, theorem) {
  const types = new Map();
  for (const line of output.split('\n').filter(text => text.startsWith('CAVEAT_GROUND '))) {
    const found = JSON.parse(line.slice('CAVEAT_GROUND '.length));
    types.set(found.name, found.found ? found.type : null);
  }
  if (!types.get(theorem)) return { found: false };
  const axiomsMatch = new RegExp(`'${theorem.replaceAll('.', '\\.')}' (?:depends on axioms:\\s*\\[([^\\]]*)\\]|does not depend on any axioms)`).exec(output);
  if (!axiomsMatch) throw new Error('No axiom report for ' + theorem);
  const axioms = (axiomsMatch[1] ?? '').split(',').map(value => value.trim()).filter(Boolean);
  return { found: true, type: types.get(theorem), axioms, types };
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
    const assumes = entry.assumes ?? [];
    const lookup = name => [
      `  match (← getEnv).find? \`${name} with`,
      `  | none => IO.println ("CAVEAT_GROUND " ++ (Json.mkObj [("name", "${name}"), ("found", false)]).compress)`,
      `  | some info => IO.println ("CAVEAT_GROUND " ++ (Json.mkObj [("name", "${name}"), ("found", true), ("type", toString info.type)]).compress)`,
    ];
    writeFileSync(audit, [
      'import Lean',
      `import ${module}`,
      'open Lean Elab Command',
      '#eval show CommandElabM Unit from do',
      ...[entry.theorem, ...assumes.map(assumption => assumption.name)].flatMap(lookup),
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
    const declared = new Map(assumes.map(assumption => [assumption.name, assumption]));
    const extra = result.axioms.filter(axiom => !allowedAxioms.has(axiom));
    const undeclared = extra.filter(axiom => !declared.has(axiom));
    if (undeclared.length > 0) return { ok: false, reason: 'broken', detail: 'unapproved axioms: ' + undeclared.join(', '), axioms: result.axioms };
    const hash = statementHash(result.type);
    if (hash !== entry.statement_sha256) {
      return { ok: false, reason: 'changed', detail: `statement hash ${hash}, recorded ${entry.statement_sha256}`, type: result.type };
    }
    const assumed = extra.filter(axiom => declared.has(axiom));
    for (const name of assumed) {
      const type = result.types.get(name);
      const assumedHash = type ? statementHash(type) : null;
      if (assumedHash !== declared.get(name).statement_sha256) {
        return { ok: false, reason: 'changed', detail: `assumed ${name} hash ${assumedHash}, recorded ${declared.get(name).statement_sha256}`, type };
      }
    }
    return { ok: true, axioms: result.axioms, assumed, statement_sha256: hash, type: result.type };
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

/** The event a host sends the program for one checked entry. */
export function eventFor(entry, result) {
  if (!result.ok) return { event: entry.on_fail, payload: { why: result.reason } };
  return entry.assumes ? { event: entry.on_pass, payload: { assumed: result.assumed.length } } : { event: entry.on_pass };
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

/**
 * Each control changes one entry's source and must get its expected result: a
 * refusal for that reason (with that detail, where given), or a pass with
 * that many assumed results.
 */
export const controls = [
  { id: 'sorry', entry: 0, expect: 'broken', detail: /sorryAx/, find: /(theorem lt_two_pow[^\n]*:= by\n)[\s\S]*?\n\n/, replace: '$1  sorry\n\n' },
  { id: 'statement', entry: 0, expect: 'changed', find: 'n < 2 ^ n', replace: 'n + 0 < 2 ^ n' },
  { id: 'renamed', entry: 0, expect: 'missing', find: /\blt_two_pow\b/g, replace: 'lt_two_pow_renamed' },
  { id: 'false_proof', entry: 0, expect: 'broken', detail: /error: Tactic `rfl` failed/, find: 'omega', replace: 'rfl' },
  { id: 'undeclared_assumption', entry: 1, expect: 'broken', detail: /unapproved axioms: LeanBridgeDemo\.Assumed\.two_pow_add_unlisted/, find: /two_pow_add\b/g, replace: 'two_pow_add_unlisted' },
  { id: 'assumption_changed', entry: 1, expect: 'changed',
    find: 'axiom two_pow_add (a b : Nat) : 2 ^ (a + b) = 2 ^ a * 2 ^ b',
    replace: 'axiom two_pow_add (a b : Nat) : 2 ^ (a + b + 0) = 2 ^ a * 2 ^ b' },
  { id: 'assumption_proven', entry: 1, expect: { assumed: 0 },
    find: 'axiom two_pow_add (a b : Nat) : 2 ^ (a + b) = 2 ^ a * 2 ^ b',
    replace: 'theorem two_pow_add (a b : Nat) : 2 ^ (a + b) = 2 ^ a * 2 ^ b := Nat.pow_add 2 a b' },
];

export function controlOutcome(control, result) {
  if (typeof control.expect === 'string') {
    const passed = !result.ok && result.reason === control.expect && (!control.detail || control.detail.test(result.detail));
    return { got: result.ok ? 'passed' : result.reason, passed };
  }
  const got = result.ok ? { assumed: result.assumed.length } : result.reason;
  return { got, passed: result.ok && result.assumed.length === control.expect.assumed };
}

export function runControls(manifestPath) {
  const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  return controls.map(control => {
    const entry = manifest.grounds[control.entry];
    const original = readFileSync(join(dirname(manifestPath), entry.source), 'utf8');
    const mutated = original.replace(control.find, control.replace);
    if (mutated === original) throw new Error('Control anchor not found: ' + control.id);
    const work = mkdtempSync(join(tmpdir(), 'lean-grounds-control-'));
    try {
      const source = join(work, basename(entry.source));
      writeFileSync(source, mutated);
      return { id: control.id, expect: control.expect, ...controlOutcome(control, checkGround(entry, source)) };
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
    const note = result.ok ? (result.assumed.length ? ` (assuming ${result.assumed.join(', ')})` : '') : ` (${result.reason}: ${result.detail})`;
    console.log(`${result.ok ? 'ok    ' : 'FAILED'} ${entry.evidence} <- ${entry.theorem}${note}`);
  }
  for (const control of controlResults) {
    console.log(`${control.passed ? 'ok    ' : 'FAILED'} control ${control.id}: expected ${JSON.stringify(control.expect)}, got ${JSON.stringify(control.got)}`);
  }
  const failed = checked.some(({ result }) => !result.ok) || controlResults.some(control => !control.passed);
  process.exitCode = failed ? 1 : 0;
}
