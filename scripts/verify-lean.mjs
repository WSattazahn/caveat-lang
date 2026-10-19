import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { delimiter, dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const proofRoot = join(root, 'proofs', 'lean');
const allowedAxioms = new Set(['propext', 'Classical.choice', 'Quot.sound']);

export function validateInventory(names) {
  if (!Array.isArray(names) || names.length === 0 ||
      names.some(name => typeof name !== 'string' || !/^Caveat\.[A-Za-z0-9_.]+$/.test(name)) ||
      new Set(names).size !== names.length) {
    throw new Error('The theorem inventory must contain unique, fully qualified Caveat names.');
  }
}

export function auditLaws(laws, inventory) {
  validateInventory(laws);
  if (laws.some(name => !inventory.includes(name))) throw new Error("Authored law is missing from the theorem inventory.");
  return laws.length;
}

export function auditAxioms(output, names) {
  validateInventory(names);
  const seen = new Map();
  const pattern = /'([^']+)' (?:depends on axioms:\s*\[([^\]]*)\]|does not depend on any axioms)/g;
  for (const match of output.matchAll(pattern)) {
    const name = match[1];
    if (seen.has(name)) throw new Error('Duplicate axiom audit: ' + name);
    const axioms = (match[2] ?? '').split(',').map(value => value.trim()).filter(Boolean);
    for (const axiom of axioms) {
      if (!allowedAxioms.has(axiom)) throw new Error('Unapproved axiom: ' + axiom);
    }
    seen.set(name, axioms);
  }
  if (seen.size !== names.length || names.some(name => !seen.has(name))) {
    throw new Error('Axiom audit does not exactly match the theorem inventory.');
  }
  return Object.fromEntries(seen);
}

export function auditInventory(output, names) {
  validateInventory(names);
  const actual = [...output.matchAll(/^CAVEAT_THEOREM (\S+)\r?$/gm)].map(match => match[1]);
  if (new Set(actual).size !== actual.length || actual.length !== names.length ||
      actual.some(name => !names.includes(name))) {
    throw new Error('Lean environment does not exactly match the theorem inventory.');
  }
  return actual;
}

/** Model defects in the shapes of D014/D020, D018 and a take-back reaching a record. */
export const sourcesMutants = [
  { id: 'QualifyMissesRecords', breaks: 'qualify_preserves',
    find: '    | _ => addRoute o c k) s.cells',
    replace: '    | .grounds _ => addRoute o c k\n    | _ => k) s.cells' },
  { id: 'DepartureLeavesRoutes', breaks: 'depart_preserves',
    find: 'def depart (o : Occ) (k : Carrier) : Carrier := { k with routes := k.routes.map (keepRoute o) }',
    replace: 'def depart (o : Occ) (k : Carrier) : Carrier := k' },
  { id: 'TakeBackReachesRecord', breaks: 'take_back_keeps_kept_routes',
    find: '  | (_, .kept) => false',
    replace: '  | (c, .kept) => decide (c = caveat)' },
];

export function applyMutant(model, mutant) {
  if (model.split(mutant.find).length !== 2) throw new Error('Mutant anchor is not unique: ' + mutant.id);
  return model.replace(mutant.find, mutant.replace);
}

/** Whether Lean reported an error inside the named theorem's source lines. */
export function errorInTheorem(output, model, theorem) {
  const lines = model.split('\n');
  const start = lines.findIndex(line => line.startsWith('theorem ' + theorem + ' '));
  if (start < 0) return false;
  let end = lines.findIndex((line, index) => index > start && /^(theorem|def|structure|inductive|instance|abbrev|\/-)/.test(line));
  if (end < 0) end = lines.length;
  return [...output.matchAll(/:(\d+):\d+: error/g)].some(match => {
    const line = Number(match[1]) - 1;
    return line >= start && line < end;
  });
}

function sources(directory) {
  return readdirSync(directory, { withFileTypes: true })
    .filter(entry => entry.name !== '.lake')
    .flatMap(entry => entry.isDirectory() ? sources(join(directory, entry.name)) : [join(directory, entry.name)]);
}

export function verify() {
  const outputDirectory = join(root, 'test-results', 'lean-verification');
  mkdirSync(outputDirectory, { recursive: true });
  const report = {
    scope: 'provenance algebra, outcome contract, bounded executable bridge laws, late-qualification laws, reopening laws and the Caveat Sources invariant; no Rust/Lean conformance claim',
    startedAt: new Date().toISOString(),
    status: 'failed',
    commands: [],
    sources: {},
  };
  writeFileSync(join(outputDirectory, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  const env = { ...process.env };
  const pathKey = Object.keys(env).find(key => key.toLowerCase() === 'path') ?? 'PATH';
  if (env.CAVEAT_LEAN_BIN) env[pathKey] = env.CAVEAT_LEAN_BIN + delimiter + (env[pathKey] ?? '');
  function run(command, args, expected = 'success') {
    const result = spawnSync(command, args, {
      cwd: proofRoot, env, encoding: 'utf8', timeout: 600_000, maxBuffer: 16 * 1024 * 1024,
    });
    const output = (result.stdout ?? '') + (result.stderr ?? '');
    const log = String(report.commands.length + 1).padStart(2, '0') + '.log';
    writeFileSync(join(outputDirectory, log), output);
    report.commands.push({ command, args, exitCode: result.status, signal: result.signal,
      expected, log, error: result.error?.message });
    if (result.error || result.signal || result.status === null) {
      throw new Error('Verification infrastructure failed: ' + command + ' ' + args.join(' ') +
        ': ' + (result.error?.message ?? result.signal));
    }
    if (expected === 'success' && result.status !== 0) throw new Error(output);
    if (expected === 'failure' && result.status !== 1) throw new Error('Negative proof control must fail normally with exit code 1.');
    return output;
  }
  try {
    const proofSources = sources(proofRoot).sort();
    const gateSources = ['scripts/verify-lean.mjs', 'scripts/verify-lean.test.mjs', '.github/workflows/lean.yml', 'package.json'].map(path => join(root, path));
    report.sources = Object.fromEntries([...proofSources, ...gateSources].map(path => [
      relative(root, path).replaceAll('\\', '/'),
      createHash('sha256').update(readFileSync(path)).digest('hex'),
    ]));
    report.revision = run('git', ['rev-parse', 'HEAD']).trim();
    report.workingTree = run('git', ['status', '--short']).trim();
    const pin = readFileSync(join(proofRoot, 'lean-toolchain'), 'utf8').trim();
    const match = /^leanprover\/lean4:v(\d+\.\d+\.\d+)$/.exec(pin);
    if (!match) throw new Error('Expected an exact stable Lean version pin.');
    const version = run('lean', ['--version']).trim();
    if (!version.startsWith('Lean (version ' + match[1] + ',')) {
      throw new Error('Active Lean does not match lean-toolchain: ' + version);
    }
    report.toolchain = { pin, version };
    run('lake', ['build', '--wfail']);
    const names = JSON.parse(readFileSync(join(proofRoot, 'theorems.json'), 'utf8'));
    validateInventory(names);
    const laws = JSON.parse(readFileSync(join(proofRoot, 'laws.json'), 'utf8'));
    report.authoredLawCount = auditLaws(laws, names);
    const audit = run('lake', ['env', 'lean', '--error=hasSorry', 'Audit.lean']);
    auditInventory(audit, names);
    report.axioms = auditAxioms(audit, names);
    report.theoremCount = names.length;
    const replay = run('lake', ['env', 'leanchecker', '--verbose', 'Caveat']);
    const modules = proofSources.filter(path => path.endsWith('.lean') &&
      relative(proofRoot, path) !== 'Audit.lean').map(path =>
      relative(proofRoot, path).replaceAll('\\', '.').replaceAll('/', '.').replace(/\.lean$/, ''));
    for (const moduleName of modules) {
      if (!replay.split(/\r?\n/).includes('replaying ' + moduleName)) {
        throw new Error('Kernel replay did not report module ' + moduleName);
      }
    }
    report.replay = { modules,
      boundary: 'Lean kernel replay of project modules with imported toolchain environment' };

    // Exercise actual Lean failures and transitive audit rejection, not only parser mocks.
    const controls = mkdtempSync(join(proofRoot, '.lake', 'verification-controls-'));
    const sorryPath = join(controls, 'Sorry.lean');
    writeFileSync(sorryPath, 'import Caveat\nnamespace Caveat.Control\ntheorem incomplete : False := by sorry\nend Caveat.Control\n');
    const sorry = run('lake', ['env', 'lean', '--error=hasSorry', sorryPath], 'failure');
    if (!/error: declaration uses .sorry./.test(sorry)) throw new Error('Incomplete-proof control failed for an unrelated reason.');
    const axiomPath = join(controls, 'Axiom.lean');
    writeFileSync(axiomPath, 'import Caveat\nnamespace Caveat.Control\naxiom impossible : False\ntheorem indirect : False := impossible\ntheorem accepted : False := indirect\nend Caveat.Control\n#print axioms Caveat.Control.accepted\n');
    const custom = run('lake', ['env', 'lean', '--error=hasSorry', axiomPath]);
    let rejected = false;
    try { auditAxioms(custom, ['Caveat.Control.accepted']); }
    catch (error) {
      if (error.message === 'Unapproved axiom: Caveat.Control.impossible') rejected = true;
      else throw error;
    }
    if (!rejected) throw new Error('Transitive custom axiom was accepted.');
    // A model that drifts from Caveat Sources 0.1 in a finding's shape must fail
    // to prove the theorem that covers it, so the invariant proof is not vacuous.
    const sourcesModel = readFileSync(join(proofRoot, 'Caveat', 'Sources.lean'), 'utf8');
    const mutantResults = [];
    for (const mutant of sourcesMutants) {
      const path = join(controls, 'Sources' + mutant.id + '.lean');
      writeFileSync(path, applyMutant(sourcesModel, mutant));
      const output = run('lake', ['env', 'lean', '--error=hasSorry', path], 'failure');
      if (!errorInTheorem(output, applyMutant(sourcesModel, mutant), mutant.breaks)) {
        throw new Error('Caveat Sources mutant ' + mutant.id + ' failed outside ' + mutant.breaks + '.');
      }
      mutantResults.push(mutant.id + ': ' + mutant.breaks + ' refused');
    }
    report.negativeControls = ['incomplete proof rejected by Lean', 'transitive custom axiom rejected by audit',
      ...mutantResults];
    report.status = 'passed';
    console.log('Lean scaffold verified: ' + laws.length + ' authored laws, ' + names.length + ' inventoried theorems, transitive axiom audit, kernel replay, negative controls, ' + sourcesMutants.length + ' model mutants refused.');
  } catch (error) {
    report.error = error.message;
    throw error;
  } finally {
    report.finishedAt = new Date().toISOString();
    writeFileSync(join(outputDirectory, 'report.json'), JSON.stringify(report, null, 2) + '\n');
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { verify(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
