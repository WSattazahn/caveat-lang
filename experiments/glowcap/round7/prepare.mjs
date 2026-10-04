// Prepares the round-7 directories outside the repository. Each agent gets a
// directory holding only what the protocol gives it; the private store holds
// the logs, snapshots and unreleased phases.
//
//   node prepare.mjs writer   --tarball=PATH        the change-request writer's directory
//   node prepare.mjs reviewer                       the scenario reviewer's (after the writer)
//   node prepare.mjs authors  --tarball=PATH        the release store and six author directories
//   node prepare.mjs manifest --tarball=PATH        hashes of every frozen input (packet/manifest.json)
//
// --root=DIR sets the directory (default /home/claude/glowcap-r7).
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { PHASES, allScenarios, applies } from './lib.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const arg = (name, fallback) => process.argv.find((a) => a.startsWith(`--${name}=`))?.slice(name.length + 3) ?? fallback;
const ROOT = arg('root', '/home/claude/glowcap-r7');
const TARBALL = arg('tarball');
const TARBALL_SHA256 = '13fd6e298731c46e024f10788e1f68834a1c02625fd74bb4715730b4a4887a8e';
const TYPESCRIPT = '7.0.2';
// --smoke: two root-only authors through the phases that have texts, in a
// separate root; infrastructure checks, never results.
const SMOKE = process.argv.includes('--smoke');
const AUTHORS = SMOKE ? [['SMOKE-C', 'caveat'], ['SMOKE-T', 'ts']]
  : [['C1', 'caveat'], ['T1', 'ts'], ['C2', 'caveat'], ['T2', 'ts'], ['C3', 'caveat'], ['T3', 'ts']];
const GUIDE = 'docs/reference/docs/AI_AUTHORING.md';
const CUT = '## Evidence from fresh authors';

const sha256 = (file) => createHash('sha256').update(readFileSync(file)).digest('hex');
const fill = (text, values) => text.replace(/@@([A-Z]+)@@/g, (_, key) => {
  if (!(key in values)) throw new Error(`no value for @@${key}@@`);
  return values[key];
});
const write = (file, text) => { mkdirSync(path.dirname(file), { recursive: true }); writeFileSync(file, text); };

function checkTarball() {
  if (!TARBALL) throw new Error('--tarball=PATH is required');
  const hash = sha256(TARBALL);
  if (hash !== TARBALL_SHA256) throw new Error(`tarball SHA256 ${hash} is not the registered ${TARBALL_SHA256}`);
  return path.resolve(TARBALL);
}

// v2's one transformation: the guide loses its final section, from CUT on.
function transformGuide(file) {
  const text = readFileSync(file, 'utf8');
  const at = text.indexOf(`\n${CUT}\n`);
  if (at < 0) throw new Error(`${file} has no "${CUT}" section`);
  writeFileSync(file, text.slice(0, at + 1));
}

function writer() {
  const dir = path.join(ROOT, 'writer');
  if (existsSync(dir)) throw new Error(`${dir} exists`);
  cpSync(path.join(here, 'BEAT.md'), path.join(dir, 'BEAT.md'));
  cpSync(path.join(here, '..', 'scenarios.mjs'), path.join(dir, 'existing-scenarios.mjs'));
  cpSync(path.join(here, 'workspace', 'README-writer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

function reviewer() {
  const from = path.join(ROOT, 'writer');
  const dir = path.join(ROOT, 'reviewer');
  if (existsSync(dir)) throw new Error(`${dir} exists`);
  for (const file of ['BEAT.md', 'existing-scenarios.mjs', 'REQUESTS.md', 'scenarios-r7.mjs']) cpSync(path.join(from, file), path.join(dir, file));
  cpSync(path.join(here, 'workspace', 'README-reviewer.md'), path.join(dir, 'README.md'));
  console.log(dir);
}

const SIDES = {
  caveat: {
    LANGUAGE: 'the Caveat programming language (package `caveat-lang` 0.1.0-rc.11, installed in `node_modules/caveat-lang`)',
    WHAT: 'The game rules, text and explanations go in Caveat source, `impl/glowcap.cav` (more `.cav` files if you like).\n`impl/adapter.mjs` is the host glue: it loads the program with the package\'s runtime\n(`caveat-lang/session`), translates each event into your program\'s events and the program\'s view\nor snapshot into the required view shape, and exports `createPolicy(saved?)` returning\n`{ dispatch(event), view(), save() }`. The adapter must not make the game\'s decisions itself.\nThe adapter counts as code in the measurements, like the program.',
    CHECK: 'runs `caveat check` on each `.cav` file',
    CAVEATCMD: ', and `node run.mjs caveat ARGS…` (the package\'s CLI: `validate`, `check`, `explain`,\n`dependents`, `replay`, `test`)',
    READ: ', and the package\'s documentation:\n  `node_modules/caveat-lang/README.md`, everything under `node_modules/caveat-lang/docs/` (start with\n  `docs/AGENT_START.md`, then `docs/reference/docs/AI_AUTHORING.md` and `docs/REFERENCE.md`) and the\n  type declarations `node_modules/caveat-lang/lib/*.d.mts`. Not the package\'s other files: its\n  runtime, `lib/*.mjs`, `examples/` or `templates/`',
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

function release() {
  const dir = path.join(ROOT, 'private', 'release');
  const phases = SMOKE ? PHASES.filter((p) => existsSync(path.join(here, 'phases', `${p}.md`))) : PHASES;
  return allScenarios().then((scenarios) => {
    write(path.join(dir, 'phases.json'), `${JSON.stringify(phases)}\n`);
    for (const phase of phases) {
      const text = path.join(here, 'phases', `${phase}.md`);
      if (!existsSync(text)) throw new Error(`no phase text for ${phase}: register CR13-CR16 first`);
      cpSync(text, path.join(dir, 'phases', `${phase}.md`));
      write(path.join(dir, 'scenarios', `${phase}.json`), `${JSON.stringify(scenarios.filter((s) => applies(s, phase)), null, 1)}\n`);
    }
    cpSync(path.join(here, 'exec.mjs'), path.join(ROOT, 'private', 'tooling', 'exec.mjs'));
  });
}

async function authors() {
  const tarball = checkTarball();
  await release();
  const template = readFileSync(path.join(here, 'workspace', 'README-author.md'), 'utf8');
  const runner = readFileSync(path.join(here, 'workspace', 'run.mjs'), 'utf8');
  for (const [id, side] of AUTHORS) {
    const dir = path.join(ROOT, 'authors', id);
    const mine = path.join(ROOT, 'private', id);
    if (existsSync(dir) || existsSync(mine)) throw new Error(`${id} already exists`);
    write(path.join(dir, 'README.md'), fill(template, { ID: id, ...SIDES[side] }));
    write(path.join(dir, 'run.mjs'), fill(runner, { ID: id, SIDE: side, PRIVATE: path.join(ROOT, 'private') }));
    mkdirSync(path.join(dir, 'impl'), { recursive: true });
    for (const kind of ['phases', 'scenarios']) mkdirSync(path.join(dir, kind), { recursive: true });
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
  mkdirSync(scratch, { recursive: true });
  execFileSync('tar', ['xzf', tarball, '-C', scratch]);
  const guide = path.join(scratch, 'package', GUIDE);
  const original = sha256(guide);
  transformGuide(guide);
  const docs = {};
  const walk = (dir) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else docs[path.relative(path.join(scratch, 'package'), full)] = sha256(full);
    }
  };
  walk(path.join(scratch, 'package', 'docs'));
  const files = Object.fromEntries(['README.md', ...readdirSync(path.join(scratch, 'package', 'lib')).filter((f) => f.endsWith('.d.mts')).map((f) => `lib/${f}`)]
    .map((f) => [f, sha256(path.join(scratch, 'package', f))]));
  const out = {
    runtime: { package: 'caveat-lang@0.1.0-rc.11', tarballSha256: TARBALL_SHA256, tag: 'v0.1.0-rc.11', commit: '8e7805a57269c2084224df487fe186f0c6f6a4a9' },
    typescript: TYPESCRIPT,
    node: process.versions.node,
    guideTransformation: { file: GUIDE, removedFrom: CUT, originalSha256: original, transformedSha256: sha256(guide) },
    caveatPacket: { ...files, ...docs },
  };
  write(path.join(here, 'packet', 'manifest.json'), `${JSON.stringify(out, null, 2)}\n`);
  console.log(path.join(here, 'packet', 'manifest.json'));
}

const command = process.argv[2];
const commands = { writer, reviewer, authors, manifest };
if (!commands[command]) throw new Error('usage: node prepare.mjs writer|reviewer|authors|manifest [--tarball=PATH] [--root=DIR]');
await commands[command]();
