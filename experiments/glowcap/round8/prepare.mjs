// Prepares round 8's directories outside the repository (PROTOCOL.md, round
// 7's "Registration notes" carried over). Each agent's directory holds only
// what the protocol gives it; the private store holds the logs, snapshots and
// unreleased phases.
//
//   node prepare.mjs writer                   the change-request writer's directory
//   node prepare.mjs reviewer                 the scenario reviewer's (after the writer)
//   node prepare.mjs authors  --tarball=PATH  the release store and six author directories
//   node prepare.mjs manifest --tarball=PATH  hashes of every frozen input (packet/manifest.json)
//
// --root=DIR sets the directory (default /home/claude/glowcap-r8). --smoke
// prepares two authors in a separate root, for infrastructure checks only;
// --add=C4:caveat prepares one replacement author (PROTOCOL.md, row 7).
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PHASES, allScenarios, applies, phaseText } from './lib.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const ROOT = arg('root', '/home/claude/glowcap-r8');
const TARBALL = arg('tarball');
// Row 1: the first published candidate carrying View 0.2. The SHA256 is the
// tarball npm serves for caveat-lang@0.1.0-rc.17 (its sha512 matches the
// registry's integrity field); the publication record must agree before any
// author starts (see the manifest step).
const RUNTIME = {
  package: 'caveat-lang@0.1.0-rc.17',
  tarballSha256: '03bc86cf7fc396b32a8184028303202d33c6ca33439b86939e07ba6621c991b6',
  tag: 'v0.1.0-rc.17',
  commit: '304a16d6d8134f99f94f42980a759842eccb355b',
  record: 'docs/releases/v0.1.0-rc.17-npm-publication.json',
};
const TYPESCRIPT = '7.0.2';
// --smoke: two authors through every phase, in a separate root, before the
// publication record exists; infrastructure checks, never results.
const SMOKE = process.argv.includes('--smoke');
const ADD = arg('add');
const AUTHORS = SMOKE ? [['SMOKE-C', 'caveat'], ['SMOKE-T', 'ts']]
  : ADD ? [ADD.split(':')]
  : [['C1', 'caveat'], ['T1', 'ts'], ['C2', 'caveat'], ['T2', 'ts'], ['C3', 'caveat'], ['T3', 'ts']];
// Row 2: round 7's one transformation, applied to the rc.17 package.
const GUIDE = 'docs/reference/docs/AI_AUTHORING.md';
const CUT = '## Evidence from fresh authors';

const sha256 = (file) => createHash('sha256').update(readFileSync(file)).digest('hex');
const fill = (text, values) => text.replace(/@@([A-Z]+)@@/g, (_, key) => {
  if (!(key in values)) throw new Error(`no value for @@${key}@@`);
  return values[key];
});
const write = (file, text) => { mkdirSync(path.dirname(file), { recursive: true }); writeFileSync(file, text); };

function fresh(name) {
  const dir = path.join(ROOT, name);
  if (existsSync(dir)) throw new Error(`${dir} exists`);
  return dir;
}

function writer() {
  const dir = fresh('writer');
  cpSync(path.join(here, 'BEAT.md'), path.join(dir, 'BEAT.md'));
  cpSync(path.join(here, '..', 'scenarios.mjs'), path.join(dir, 'existing-scenarios.mjs'));
  cpSync(path.join(here, '..', 'round7', 'scenarios-r7.mjs'), path.join(dir, 'existing-scenarios-r7.mjs'));
  cpSync(path.join(here, 'workspace', 'README-writer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

function reviewer() {
  const from = path.join(ROOT, 'writer');
  const dir = fresh('reviewer');
  for (const file of ['BEAT.md', 'existing-scenarios.mjs', 'existing-scenarios-r7.mjs', 'REQUESTS.md', 'scenarios-r8.mjs']) {
    cpSync(path.join(from, file), path.join(dir, file));
  }
  cpSync(path.join(here, 'workspace', 'README-reviewer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

function checkTarball() {
  if (!TARBALL) throw new Error('--tarball=PATH is required');
  const hash = sha256(TARBALL);
  if (hash !== RUNTIME.tarballSha256) throw new Error(`tarball SHA256 ${hash} is not the registered ${RUNTIME.tarballSha256}`);
  if (SMOKE) return path.resolve(TARBALL); // infrastructure checks only; never results
  const record = path.join(here, '../../..', RUNTIME.record);
  if (!existsSync(record)) throw new Error(`${RUNTIME.record} is not in this checkout: rc.17's publication is not recorded as verified yet`);
  const verified = JSON.parse(readFileSync(record, 'utf8'));
  if (verified.status !== 'verified' || verified.tarball?.sha256 !== RUNTIME.tarballSha256) {
    throw new Error(`${RUNTIME.record} does not record ${RUNTIME.tarballSha256} as verified`);
  }
  return path.resolve(TARBALL);
}

// The packet's one transformation: the guide loses its final section, from CUT on.
function transformGuide(file) {
  const text = readFileSync(file, 'utf8');
  const at = text.indexOf(`\n${CUT}\n`);
  if (at < 0) throw new Error(`${file} has no "${CUT}" section`);
  writeFileSync(file, text.slice(0, at + 1));
}

// Round 7's sides, with the rc.17 package and its tooling (row 4): the CLI's
// check-first `test`, `types` and `init` (the starter) are listed, and the
// starter's templates may be read.
const SIDES = {
  caveat: {
    LANGUAGE: 'the Caveat programming language (package `caveat-lang` 0.1.0-rc.17, installed in `node_modules/caveat-lang`)',
    WHAT: 'The game rules, text and explanations go in Caveat source, `impl/glowcap.cav` (more `.cav` files if you like).\n`impl/adapter.mjs` is the host glue: it loads the program with the package\'s runtime\n(for example `caveat-lang/session` or `caveat-lang/starter`), translates each event into your program\'s events and the program\'s view\nor snapshot into the required view shape, and exports `createPolicy(saved?)` returning\n`{ dispatch(event), view(), save() }`. The adapter must not make the game\'s decisions itself.\nThe adapter counts as code in the measurements, like the program.',
    CHECK: 'runs `caveat check` on each `.cav` file',
    CAVEATCMD: ', and `node run.mjs caveat ARGS…` (the package\'s CLI: `validate`, `check`, `types`, `explain`,\n`dependents`, `replay`, `test`, `init`)',
    READ: ', and the package\'s documentation:\n  `node_modules/caveat-lang/README.md`, everything under `node_modules/caveat-lang/docs/` (start with\n  `docs/AGENT_START.md`, then `docs/reference/docs/AI_AUTHORING.md` and `docs/REFERENCE.md`), the\n  type declarations `node_modules/caveat-lang/lib/*.d.mts` and the starter\'s `templates/`. Not the\n  package\'s other files: its runtime, `lib/*.mjs` or `examples/`',
    RULES: '- Use no npm package other than `caveat-lang`.',
  },
  ts: {
    LANGUAGE: `TypeScript, run by Node ${process.versions.node} through its built-in type stripping`,
    WHAT: 'Write it in `impl/glowcap.ts` (more `.ts` files if you like), exporting `createPolicy(saved?)`\nreturning `{ dispatch(event), view(), save() }`. Node runs the file by stripping its types, so use\nonly erasable syntax: no `enum`, `namespace`, parameter properties or other syntax that emits code,\nand import your own files with their `.ts` extension.',
    CHECK: `runs \`tsc\` ${TYPESCRIPT} in strict mode over \`impl/\``,
    CAVEATCMD: '',
    READ: '',
    RULES: '- Use no npm package: plain TypeScript and Node built-ins only.',
  },
};

async function release() {
  const dir = path.join(ROOT, 'private', 'release');
  const phases = SMOKE ? PHASES.filter((p) => existsSync(phaseText(p))) : PHASES;
  const scenarios = await allScenarios();
  write(path.join(dir, 'phases.json'), `${JSON.stringify(phases)}\n`);
  for (const phase of phases) {
    if (!existsSync(phaseText(phase))) throw new Error(`no phase text for ${phase}`);
    cpSync(phaseText(phase), path.join(dir, 'phases', `${phase}.md`));
    write(path.join(dir, 'scenarios', `${phase}.json`), `${JSON.stringify(scenarios.filter((s) => applies(s, phase)), null, 1)}\n`);
  }
  cpSync(path.join(here, 'exec.mjs'), path.join(ROOT, 'private', 'tooling', 'exec.mjs'));
}

async function authors() {
  const tarball = checkTarball();
  if (!ADD) await release();
  const template = readFileSync(path.join(here, 'workspace', 'README-author.md'), 'utf8');
  const runner = readFileSync(path.join(here, 'workspace', 'run.mjs'), 'utf8');
  for (const [id, side] of AUTHORS) {
    if (!SIDES[side]) throw new Error(`unknown side ${side}`);
    const dir = path.join(ROOT, 'authors', id);
    const mine = path.join(ROOT, 'private', id);
    if (existsSync(dir) || existsSync(mine)) throw new Error(`${id} already exists`);
    write(path.join(dir, 'README.md'), fill(template, { ID: id, ...SIDES[side] }));
    write(path.join(dir, 'run.mjs'), fill(runner, { ID: id, SIDE: side, PRIVATE: path.join(ROOT, 'private') }));
    for (const kind of ['impl', 'phases', 'scenarios']) mkdirSync(path.join(dir, kind), { recursive: true });
    cpSync(path.join(ROOT, 'private', 'release', 'phases', 'base.md'), path.join(dir, 'phases', 'base.md'));
    cpSync(path.join(ROOT, 'private', 'release', 'scenarios', 'base.json'), path.join(dir, 'scenarios', 'base.json'));
    cpSync(path.join(ROOT, 'private', 'release', 'scenarios', 'base.json'), path.join(dir, 'scenarios', 'current.json'));
    write(path.join(dir, 'package.json'), `${JSON.stringify({ name: `glowcap-${id.toLowerCase()}`, private: true, type: 'module' }, null, 2)}\n`);
    execFileSync('npm', ['install', '--no-audit', '--no-fund', '--save-exact', side === 'caveat' ? tarball : `typescript@${TYPESCRIPT}`], { cwd: dir, stdio: 'inherit' });
    if (side === 'caveat') transformGuide(path.join(dir, 'node_modules', 'caveat-lang', GUIDE));
    else {
      write(path.join(dir, 'tsconfig.json'), `${JSON.stringify({ compilerOptions: { strict: true, noEmit: true, target: 'esnext', module: 'nodenext', allowImportingTsExtensions: true, erasableSyntaxOnly: true, verbatimModuleSyntax: true, types: [] }, include: ['impl/**/*.ts'] }, null, 2)}\n`);
    }
    write(path.join(mine, 'state.json'), `${JSON.stringify({ phase: 'base' }, null, 2)}\n`);
    write(path.join(mine, 'log.jsonl'), '');
    mkdirSync(path.join(mine, 'repo'), { recursive: true });
    execFileSync('git', ['init', '-q'], { cwd: path.join(mine, 'repo') });
    console.log(dir);
  }
}

function manifest() {
  const tarball = checkTarball();
  const scratch = path.join(ROOT, 'private', 'manifest-unpack');
  if (existsSync(scratch)) throw new Error(`${scratch} exists`);
  mkdirSync(scratch, { recursive: true });
  execFileSync('tar', ['xzf', tarball, '-C', scratch]);
  const pkg = path.join(scratch, 'package');
  const guide = path.join(pkg, GUIDE);
  const original = sha256(guide);
  transformGuide(guide);
  const packet = {};
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else packet[path.relative(pkg, full)] = sha256(full);
    }
  };
  walk(path.join(pkg, 'docs'));
  walk(path.join(pkg, 'templates'));
  for (const f of ['README.md', ...readdirSync(path.join(pkg, 'lib')).filter((f) => f.endsWith('.d.mts')).map((f) => `lib/${f}`)]) {
    packet[f] = sha256(path.join(pkg, f));
  }
  const sorted = Object.fromEntries(Object.entries(packet).sort(([a], [b]) => (a < b ? -1 : 1)));
  const out = {
    runtime: { ...RUNTIME },
    typescript: TYPESCRIPT,
    node: process.versions.node,
    guideTransformation: { file: GUIDE, removedFrom: CUT, originalSha256: original, transformedSha256: sha256(guide) },
    caveatPacket: sorted,
  };
  write(path.join(here, 'packet', 'manifest.json'), `${JSON.stringify(out, null, 2)}\n`);
  console.log(path.join(here, 'packet', 'manifest.json'));
}

const steps = { writer, reviewer, authors, manifest };
const step = steps[process.argv[2]];
if (!step) throw new Error(`usage: node prepare.mjs ${Object.keys(steps).join('|')} [--tarball=PATH] [--root=DIR]`);
await step();
