// Packs the developer kit with the reactive runtime inside it, installs the
// tarball into a fresh consumer directory, and uses it only through the
// installed package: the caveat command, the library by package name, the
// session library in a browser, the packaged getting-started guide and worked
// example followed step by step from an empty directory, and the packaged
// agent-evidence example's Python tests, which need Python 3.9 or later. Run
// after `npm run build`. The pinned CLI collision check also needs npm registry
// access (or a primed cache); third-party lifecycle scripts are disabled.
//
//   node scripts/test-kit-package.mjs            PLAYWRIGHT_CHANNEL=chrome uses an installed Chrome
//   node scripts/test-kit-package.mjs --no-browser
//
// In CI the runtime comes from the Linux build, and the job keeps the tested
// tarball with this report, build-info.json and SHA256SUMS. Publishing it is
// a separate, authorized step (docs/CONSOLIDATION_PLAN.md).
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';
import { copyFile, mkdir, readFile, readdir, rmdir, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { CLOCK_SOURCE, assertBrowserResults, checkKitInBrowser } from './test-kit-browser.mjs';
import { followGuide } from '../kit/test/guide.mjs';
import { isRelative, markdownLinks, packageLinkPath, rewriteLinks } from '../kit/test/links.mjs';
import { runCallerTests } from '../kit/test/python.mjs';
import { checkCliCollision, installedCommand } from './test-cli-collision.mjs';
import { checkMcpClient, checkMcpStatelessProbe } from './test-mcp-client.mjs';
import { checkPackageDocuments, installationCommands, packageIdentity } from './kit-docs.mjs';

const root = fileURLToPath(new URL('../', import.meta.url));
const kit = path.join(root, 'kit');
const bundled = path.join(kit, 'runtime');
const dist = path.join(root, 'dist');
const run = path.join(root, 'test-results', 'kit-package', new Date().toISOString().replace(/[:.]/g, '-'));
const consumer = path.join(run, 'consumer');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
// wasm-bindgen's declarations for the runtime module ship beside it, for a
// TypeScript host that imports caveat-lang/runtime/caveat_runtime.js itself.
const RUNTIME_FILES = ['caveat_runtime.js', 'caveat_runtime_bg.wasm', 'caveat_runtime.d.ts'];
const manifest = JSON.parse(await readFile(path.join(kit, 'package.json'), 'utf8'));
// Generate before building/packing, never repair source silently during a gate.
await checkPackageDocuments(kit);
// Caveat's license and the notices of the crates compiled into the runtime.
const LEGAL_FILES = ['LICENSE', 'THIRD_PARTY_NOTICES.md'];
// Documentation copied from the repository: [source, path in the package].
const packDocs = JSON.parse(await readFile(path.join(kit, 'pack-docs.json'), 'utf8'));
const STAGED_DOCS = packDocs.reference.map(file => [file, `docs/reference/${file}`]);
const STAGED_ROOTS = ['docs/reference'];
// The build being packed. A copied document's links that leave the package
// are rewritten to point to GitHub at its revision.
const buildInfo = JSON.parse(await readFile(path.join(dist, 'build-info.json'), 'utf8'));
const shipped = file => packDocs.reference.includes(file);
const staged = (source, text) => source.endsWith('.md') ? rewriteLinks(text, { from: source, revision: buildInfo.revision, shipped }) : { text, rewritten: [] };
// The kit's own documents, committed in kit/.
const KIT_DOCS = ['README.md', 'docs/README.md', 'docs/GETTING_STARTED.md', 'docs/REFERENCE.md', 'docs/WORKED_EXAMPLE.md', 'docs/NAMES.md', 'docs/AGENT_START.md', 'docs/MCP.md'];
// The library's modules, each with its TypeScript declarations.
const LIBRARY = ['authoring', 'authoring-worker', 'mcp', 'demo', 'doctor', 'check', 'explain', 'node', 'scenarios', 'serve', 'session', 'types'];
// The agent-evidence example, committed in kit/examples/. Nothing else, such
// as Python's bytecode or test output, may be packed with it.
const EXAMPLE = 'examples/agent-evidence';
const EXAMPLE_FILES = ['README.md', 'QUALIFICATION.md', 'BRANCHING.md', 'test_branching.py', 'assessment.cav', 'caller.py', 'test_caller.py',
  'test_lifecycle.py', 'qualification.cav', 'test_qualification.py',
  'grounded_assessment.cav', 'grounds.py', 'test_grounds.py', 'save-text.mjs'].map(file => `${EXAMPLE}/${file}`);

function npm(args, cwd) {
  // npm is a .cmd on Windows, which Node only starts through a shell, so the
  // command is one quoted string there. Arguments here never contain quotes.
  const result = process.platform === 'win32'
    ? spawnSync(`npm ${args.map(arg => `"${arg}"`).join(' ')}`, { cwd, encoding: 'utf8', shell: true })
    : spawnSync('npm', args, { cwd, encoding: 'utf8' });
  if (result.status !== 0) throw new Error(`npm ${args.join(' ')} failed:\n${result.stdout}\n${result.stderr}`);
  return result.stdout;
}

function node(args, cwd) {
  return spawnSync(process.execPath, args, { cwd, encoding: 'utf8' });
}

// A command exactly as the guide prints it. The commands are fixed text from
// the guide and contain no quotes.
function shell(command, cwd) {
  return process.platform === 'win32'
    ? spawnSync(command, { cwd, encoding: 'utf8', shell: true })
    : spawnSync('sh', ['-c', command], { cwd, encoding: 'utf8' });
}

// The bundled runtime and the legal files exist in kit/ only while packing, so
// development never runs a stale runtime and the repository keeps one copy of
// each. They are regular files, removed one by one.
async function stageRuntime() {
  for (const target of [bundled, ...LEGAL_FILES.map(file => path.join(kit, file)), ...STAGED_ROOTS.map(dir => path.join(kit, dir))]) {
    if (existsSync(target)) throw new Error(`${target} already exists; remove it before packing`);
  }
  await mkdir(bundled);
  for (const file of RUNTIME_FILES) await copyFile(path.join(dist, 'pkg-reactive', file), path.join(bundled, file));
  await copyFile(path.join(dist, 'build-info.json'), path.join(bundled, 'build-info.json'));
  for (const file of LEGAL_FILES) await copyFile(path.join(root, file), path.join(kit, file));
  for (const [source, target] of STAGED_DOCS) {
    await mkdir(path.dirname(path.join(kit, target)), { recursive: true });
    if (source.endsWith('.md')) await writeFile(path.join(kit, target), staged(source, await readFile(path.join(root, source), 'utf8')).text);
    else await copyFile(path.join(root, source), path.join(kit, target));
  }
}
async function removeRuntime() {
  for (const file of LEGAL_FILES) if (existsSync(path.join(kit, file))) await unlink(path.join(kit, file));
  // Staged documentation: the copied files, then their now-empty directories,
  // deepest first. Nothing here is removed recursively.
  const directories = new Set();
  for (const [, target] of STAGED_DOCS) {
    if (existsSync(path.join(kit, target))) await unlink(path.join(kit, target));
    for (let dir = path.posix.dirname(target); dir !== 'docs'; dir = path.posix.dirname(dir)) directories.add(dir);
  }
  for (const dir of [...directories].sort((a, b) => b.split('/').length - a.split('/').length)) {
    if (existsSync(path.join(kit, dir))) await rmdir(path.join(kit, dir));
  }
  if (!existsSync(bundled)) return;
  for (const file of await readdir(bundled)) await unlink(path.join(bundled, file));
  await rmdir(bundled);
}

const report = { schema: 1, checks: {} };
await mkdir(consumer, { recursive: true });
await stageRuntime();
let tarball;
try {
  const dry = JSON.parse(npm(['pack', '--dry-run', '--json'], kit))[0];
  const files = dry.files.map(file => file.path).sort();
  assert.deepEqual(files, [
    'LICENSE', 'THIRD_PARTY_NOTICES.md', 'bin/caveat.mjs', 'package.json',
    ...LIBRARY.flatMap(name => [`lib/${name}.mjs`, `lib/${name}.d.mts`]),
    'templates/events.jsonl', 'templates/umbrella.cav', 'templates/umbrella.scenarios.json',
    'runtime/build-info.json', ...RUNTIME_FILES.map(file => `runtime/${file}`),
    ...KIT_DOCS, ...EXAMPLE_FILES, ...STAGED_DOCS.map(([, target]) => target),
  ].sort(), 'the tarball holds exactly the library and its types, command, runtime, documentation, example, license and notices');
  const packed = JSON.parse(npm(['pack', '--json', '--pack-destination', run], kit))[0];
  assert.equal(packed.filename, `${manifest.name}-${manifest.version}.tgz`);
  tarball = path.join(run, packed.filename);
  const bytes = await readFile(tarball);
  Object.assign(report, {
    name: manifest.name, version: manifest.version, distTag: manifest.publishConfig?.tag ?? null,
    private: manifest.private === true,
    tarball: packed.filename, size: bytes.length, sha256: sha256(bytes), files,
  });
  // The build that produced the packed runtime, kept beside the tarball.
  await copyFile(path.join(dist, 'build-info.json'), path.join(run, 'build-info.json'));
} finally {
  await removeRuntime();
}

// A fresh consumer with nothing from the repository but the tarball. The kit
// has no dependencies, so the install is offline. The example it runs is the
// one the package ships.
await writeFile(path.join(consumer, 'package.json'), `${JSON.stringify({ name: 'kit-consumer', private: true, type: 'module' }, null, 2)}\n`);
npm(['install', tarball, '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], consumer);
const installed = path.join(consumer, 'node_modules', manifest.name);
for (const file of ['thermostat_history.cav', 'thermostat_history.scenarios.json']) {
  await copyFile(path.join(installed, 'docs', 'reference', 'examples', file), path.join(consumer, file));
}
const installedManifest = JSON.parse(await readFile(path.join(installed, 'package.json'), 'utf8'));
for (const key of ['name', 'version', 'private', 'publishConfig', 'bin', 'exports']) {
  assert.deepEqual(installedManifest[key], manifest[key], `installed package preserves ${key}`);
}
for (const file of RUNTIME_FILES) {
  assert.equal(sha256(await readFile(path.join(installed, 'runtime', file))), sha256(await readFile(path.join(dist, 'pkg-reactive', file))), `installed ${file} matches the build`);
}
assert.equal(await readFile(path.join(installed, 'runtime', 'build-info.json'), 'utf8'),
  await readFile(path.join(run, 'build-info.json'), 'utf8'), 'retained build metadata matches the installed runtime');
assert.deepEqual(manifest.bin, { caveat: 'bin/caveat.mjs', 'caveat-lang': 'bin/caveat.mjs' });
const aliasOutputs = {};
for (const name of ['caveat', 'caveat-lang']) {
  const help = installedCommand(consumer, name, ['help']);
  const version = installedCommand(consumer, name, ['--version']);
  for (const result of [help, version]) assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(help.stdout, /^CAVEAT Language — Programs that remember why\./);
  assert.equal(version.stdout, `CAVEAT Language ${manifest.version}\n`);
  aliasOutputs[name] = { help: help.stdout, version: version.stdout };
}
assert.deepEqual(aliasOutputs['caveat-lang'], aliasOutputs.caveat, 'both npm executable names invoke the same CLI and package version');
report.checks.cliAliases = { names: Object.keys(aliasOutputs), version: aliasOutputs.caveat.version.trim(), helpMatches: true };
const commandReports = {};
for (const name of ['caveat', 'caveat-lang']) {
  const health = installedCommand(consumer, name, ['doctor', '--json']);
  const demo = installedCommand(consumer, name, ['demo', 'agent', '--json']);
  for (const result of [health, demo]) assert.equal(result.status, 0, result.stdout + result.stderr);
  const diagnosed = JSON.parse(health.stdout);
  const demonstrated = JSON.parse(demo.stdout);
  assert.equal(diagnosed.ok, true);
  assert.equal(diagnosed.package.version, manifest.version);
  assert.equal(demonstrated.preservation.unchanged, true);
  assert.equal(demonstrated.steps.length, 5);
  commandReports[name] = { health: diagnosed, demo: demonstrated };
}
assert.deepEqual(commandReports.caveat, commandReports['caveat-lang']);
report.checks.adoptionCommands = { aliases: Object.keys(commandReports), doctor: true, demo: true };

report.checks.cliCollision = await checkCliCollision({
  tarball, directory: path.join(run, 'cli-collision'), manifest, ...aliasOutputs.caveat,
});
// Every entry point but the runtime's names its declarations, which arrived.
for (const [entry, target] of Object.entries(installedManifest.exports)) {
  if (entry === './runtime/*') continue;
  assert.deepEqual(Object.keys(target), ['types', 'default'], `${entry} lists its types first`);
  assert.equal(target.types, target.default.replace(/\.mjs$/, '.d.mts'), `${entry} declares its own module`);
  assert.ok(existsSync(path.join(installed, target.types)), `${entry}: ${target.types} is installed`);
}
report.checks.mcp = await checkMcpClient({ installed, directory: path.join(run, 'mcp-client') });
report.checks.mcpStateless = await checkMcpStatelessProbe({ installed, directory: path.join(run, 'mcp-stateless') });
report.checks.install = true;

// The license ships unchanged, and the metadata says what it is.
for (const file of LEGAL_FILES) {
  assert.equal(await readFile(path.join(installed, file), 'utf8'), await readFile(path.join(root, file), 'utf8'), `installed ${file} matches the repository`);
}
assert.match(await readFile(path.join(installed, 'LICENSE'), 'utf8'), /^MIT License\n/);
assert.equal(installedManifest.license, 'MIT');
report.checks.license = 'MIT';

// The documentation ships as copied, except that a copied document's links
// that leave the package point to GitHub at the build's revision.
let rewrittenLinks = 0;
for (const [source, target] of STAGED_DOCS) {
  const expected = staged(source, await readFile(path.join(root, source), 'utf8'));
  assert.equal(await readFile(path.join(installed, target), 'utf8'), expected.text, `installed ${target} matches ${source}, its links rewritten`);
  rewrittenLinks += expected.rewritten.length;
}
// Every relative link in every installed document resolves inside the package.
const installedDocs = [];
const walk = async directory => {
  for (const entry of await readdir(path.join(installed, directory), { withFileTypes: true })) {
    const file = path.posix.join(directory, entry.name);
    if (entry.isDirectory()) await walk(file);
    else if (file.endsWith('.md')) installedDocs.push(file);
  }
};
await walk('.');
const inside = [];
for (const file of installedDocs) {
  for (const { target: link } of markdownLinks(await readFile(path.join(installed, file), 'utf8'))) {
    if (!isRelative(link)) continue;
    const target = packageLinkPath(file, link);
    assert.ok(existsSync(path.join(installed, target)), `${file} links to ${link}, which is not in the package`);
    inside.push(`${file} -> ${link}`);
  }
}
// Routed blocks ship with their profile, and the references that link to it
// reach it inside the package.
const routedSpec = 'docs/reference/spec/caveat-routed-repetition-0.1.md';
assert.ok(existsSync(path.join(installed, routedSpec)), `${routedSpec} is in the package`);
for (const from of ['caveat-repetition-0.1.md', 'caveat-check-0.1.md']) {
  const link = `docs/reference/spec/${from} -> caveat-routed-repetition-0.1.md`;
  assert.ok(inside.includes(link), `${link} resolves inside the package`);
}
const packageDocuments = await checkPackageDocuments(installed);
report.docs = { copied: STAGED_DOCS.length, documents: installedDocs.length, linksInside: inside.length, linksRewritten: rewrittenLinks,
  packageIdentity: packageIdentity(installedManifest),
  generatedBlocks: packageDocuments.flatMap(document => document.blocks.map(block => `${document.file}:${block}`)),
};
report.checks.docs = true;

// Follow the README from a fresh consumer. Its exact-version npm install is
// deliberately substituted with the retained tarball: this checks unpublished
// candidates without downloading a different artifact or repacking after tests.
const readmeReader = path.join(run, 'readme-reader');
await mkdir(readmeReader);
npm(['init', '-y'], readmeReader);
npm(['install', tarball, '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], readmeReader);
const readmeInstalled = path.join(readmeReader, 'node_modules', manifest.name);
const readmeManifest = JSON.parse(await readFile(path.join(readmeInstalled, 'package.json'), 'utf8'));
assert.equal(packageIdentity(readmeManifest), packageIdentity(manifest));
const readme = await readFile(path.join(readmeInstalled, 'README.md'), 'utf8');
const printed = /<!-- caveat-package:starter -->\n```sh\n([\s\S]*?)\n```\n<!-- \/caveat-package:starter -->/.exec(readme)?.[1].split('\n');
assert.deepEqual(printed, installationCommands(readmeManifest, 'starter'), 'the installed README pins and runs this package');
const readmeCommands = [];
for (const command of printed.slice(2)) {
  const result = shell(command, readmeReader);
  assert.equal(result.status, 0, `${command}\n${result.stdout}${result.stderr}`);
  if (command.endsWith('--version')) assert.equal(result.stdout, `CAVEAT Language ${manifest.version}\n`);
  if (command.endsWith(' doctor')) assert.match(result.stdout, /^CAVEAT Language doctor: checks passed/);
  if (command.endsWith('demo agent')) assert.match(result.stdout, /^CAVEAT Language: agent demo/);
  if (command.includes(' test ')) assert.match(result.stdout, /2 passed, 0 failed/);
  if (command.includes(' explain ')) {
    assert.match(result.stdout, /umbrella@1/);
    assert.match(result.stdout, /rain_chance@1/);
    assert.match(result.stdout, /forecast_is_old/);
  }
  readmeCommands.push(command);
}
report.checks.readme = { packageIdentity: packageIdentity(readmeManifest), tarballSha256: report.sha256,
  documentedInstall: printed[1], installedFrom: report.tarball, commands: readmeCommands };

// The command, by its installed path.
const cli = path.join(installed, 'bin', 'caveat.mjs');
const passing = node([cli, 'test', 'thermostat_history.scenarios.json'], consumer);
assert.equal(passing.status, 0, passing.stdout + passing.stderr);
const doc = JSON.parse(await readFile(path.join(consumer, 'thermostat_history.scenarios.json'), 'utf8'));
doc.scenarios[0].steps[4].expect['/bindings/heating/text'] = '50%';
await writeFile(path.join(consumer, 'altered.scenarios.json'), JSON.stringify(doc));
const failing = node([cli, 'test', 'altered.scenarios.json'], consumer);
assert.equal(failing.status, 1);
assert.match(failing.stdout, /FAIL T01 step 5 expect \[primary\]: \/bindings\/heating\/text expected "50%", actual "0%"/);
await writeFile(path.join(consumer, 'invalid.scenarios.json'), JSON.stringify({ ...doc, extra: true }));
assert.equal(node([cli, 'test', 'invalid.scenarios.json'], consumer).status, 2);
const json = JSON.parse(node([cli, 'test', '--json', 'thermostat_history.scenarios.json'], consumer).stdout);
assert.equal(json.runtime.revision, buildInfo.revision, 'the report names the bundled runtime');
report.runtime = json.runtime;
await writeFile(path.join(consumer, 'readings.jsonl'), '{"event":"read","payload":{"value":17}}\n{"event":"read","payload":{"value":99}}\n');
const explained = node([cli, 'explain', 'thermostat_history.cav', 'readings.jsonl'], consumer);
assert.equal(explained.status, 0, explained.stdout + explained.stderr);
assert.match(explained.stdout, /heating@1 = 1 {2}in force\n {6}based on temperature@1 \(caveats: calibration_offset\)/);
assert.match(explained.stdout, /read \{"value":99\} {2}refused \(input\/bound_exceeded\)/);
const rests = node([cli, 'dependents', 'thermostat_history.cav', 'temperature@1', 'readings.jsonl'], consumer);
assert.equal(rests.status, 0, rests.stdout + rests.stderr);
assert.match(rests.stdout, /heating@1 = 1 {2}in force {2}based on temperature@1/);
assert.match(node([cli, 'validate', 'thermostat_history.cav'], consumer).stdout, /^thermostat_history\.cav loads\./);
const replayed = node([cli, 'replay', 'thermostat_history.cav', 'readings.jsonl'], consumer);
const lines = text => text.trim().split('\n').map(line => JSON.parse(line));
assert.deepEqual(lines(replayed.stdout).map(record => record.outcome ?? record.record), ['initial', 'accepted', 'rejected']);
const served = spawnSync(process.execPath, [cli, 'serve', 'thermostat_history.cav'], {
  cwd: consumer, encoding: 'utf8', input: '{"id":1,"op":"dispatch","event":"read","payload":{"value":17}}\n{"id":2,"op":"close"}\n',
});
assert.equal(served.status, 0, served.stderr);
assert.deepEqual(lines(served.stdout).map(line => line.ready ?? line.outcome ?? line.ok), [true, 'accepted', true]);
assert.equal(node([cli, 'init', 'started'], consumer).status, 0);
assert.equal(node([cli, 'test', 'umbrella.scenarios.json'], path.join(consumer, 'started')).status, 0);
report.checks.command = true;

// The library, imported by package name from the consumer.
await writeFile(path.join(consumer, 'clock.cav'), CLOCK_SOURCE);
await writeFile(path.join(consumer, 'use.mjs'), `
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { loadRuntimeFromDirectory } from '${manifest.name}/node';
import { CaveatError } from '${manifest.name}/session';
import { parseScenarioFile } from '${manifest.name}/scenarios';
import { dependents, explain } from '${manifest.name}/explain';
import { createServer } from '${manifest.name}/serve';
import { declarations, typed } from '${manifest.name}/types';
const runtime = await loadRuntimeFromDirectory();
const source = await readFile('thermostat_history.cav', 'utf8');
const session = runtime.open(source);
assert.equal(session.dispatch('read', { value: 17 }).snapshot.bindings.heating.text, '100%');
assert.equal(session.view().bindings.heating.text, '100%');
const before = session.save();
assert.deepEqual([session.dispatch('read', { value: 'x' }).code, session.save() === before], ['payload_invalid', true]);
assert.deepEqual([session.dispatch('read', { value: 41 }).code, session.save() === before], ['bound_exceeded', true]);
assert.equal(session.dispatch('warm').code, 'unknown_event');
assert.throws(() => session.dispatch('read', { value: NaN }), error => error instanceof CaveatError && error.kind === 'payload');
const resumed = runtime.restore(source, session.save());
assert.deepEqual(resumed.snapshot(), session.snapshot());
assert.deepEqual(resumed.dispatch('read', { value: 25 }), session.dispatch('read', { value: 25 }));
const clockSource = await readFile('clock.cav', 'utf8');
const clock = runtime.open(clockSource);
clock.dispatch('advance', { dt: 2.5 });
assert.equal(runtime.restore(clockSource, clock.save()).view().bindings.hud.elapsed, 2.5);
parseScenarioFile(await readFile('thermostat_history.scenarios.json', 'utf8'));
assert.deepEqual(explain(session.snapshot()).decisions[0].revisions.map(revision => revision.id), ['heating@1', 'heating@2']);
assert.deepEqual(dependents(session.snapshot(), 'temperature@2').decisions.map(item => [item.id, item.basis]), [['heating@2', 'grounds']]);
const server = createServer({ runtime, source });
assert.equal(server.handle(JSON.stringify({ op: 'dispatch', event: 'read', payload: { value: 17 } })).response.outcome, 'accepted');
server.close();
assert.match(declarations(runtime.interface(source)), /^    read: \{ value: number \};$/m);
assert.equal(typed(session), session);
console.log(JSON.stringify(runtime.identity));
`);
const used = node(['use.mjs'], consumer);
assert.equal(used.status, 0, used.stdout + used.stderr);
assert.equal(JSON.parse(used.stdout).revision, buildInfo.revision);
report.checks.library = true;

// Findings 123, 188-191 through the installed runtime: a genuine examination
// restores and plays on; an examination the budget never paid for is refused.
await writeFile(path.join(consumer, 'attention.mjs'), `
import assert from 'node:assert/strict';
import { loadRuntimeFromDirectory } from '${manifest.name}/node';
const runtime = await loadRuntimeFromDirectory();
const source = 'budget 2;\\nclaim ready;\\nevidence sensor from "sensor";\\ncaveat stale consequence low;\\n'
  + 'event observe;\\nevent quiet;\\non observe reveal sensor supports ready;\\non quiet examine stale cost 1;\\n';
const session = runtime.open(source);
assert.equal(session.dispatch('observe').outcome, 'accepted');
const forged = JSON.parse(session.save());
forged.graph.attention = { stale: 'examined' };
assert.throws(() => runtime.restore(source, JSON.stringify(forged)),
  error => error.kind === 'restore' && /examined caveats cost at least 1 attention to examine, more than the 0 the budget spent/.test(error.message));
assert.equal(session.dispatch('quiet').outcome, 'accepted');
const resumed = runtime.restore(source, session.save());
assert.deepEqual(resumed.snapshot(), session.snapshot());
assert.deepEqual(resumed.dispatch('observe'), session.dispatch('observe'));
assert.deepEqual(runtime.restore(source, resumed.save()).snapshot(), session.snapshot());
console.log(JSON.stringify(JSON.parse(session.save()).graph.attention));
`);
const attention = node(['attention.mjs'], consumer);
assert.equal(attention.status, 0, attention.stdout + attention.stderr);
report.checks.attentionRestore = JSON.parse(attention.stdout);

// The new refusal codes must keep working through the package's own runtime,
// session wrapper, server adapter and CLI, including after save/restore.
await writeFile(path.join(consumer, 'recovery.mjs'), `
import { loadRuntimeFromDirectory } from 'caveat-lang/node';
import { createServer } from 'caveat-lang/serve';
import { checkRecoveryClient } from ${JSON.stringify(pathToFileURL(path.join(root, 'scripts/check-recovery-client.mjs')).href)};
const runtime = await loadRuntimeFromDirectory(${JSON.stringify(path.join(installed, 'runtime'))});
const checks = await checkRecoveryClient({ runtime, createServer,
  command: [process.execPath, ${JSON.stringify(path.join(installed, 'bin/caveat.mjs'))}],
  directory: ${JSON.stringify(path.join(consumer, 'recovery-cases'))} });
console.log(JSON.stringify({ identity: runtime.identity, checks }));
`);
const recovery = node(['recovery.mjs'], consumer);
assert.equal(recovery.status, 0, recovery.stdout + recovery.stderr);
report.checks.recovery = JSON.parse(recovery.stdout);
assert.equal(report.checks.recovery.identity.reactiveWasmSha256,
  sha256(await readFile(path.join(installed, 'runtime/caveat_runtime_bg.wasm'))));

// The packaged getting-started guide, followed from an empty directory: install
// the tarball, check the command, then write each file, make each edit and run
// each command exactly as the guide prints them.
const reader = path.join(run, 'reader');
await mkdir(reader);
npm(['init', '-y'], reader);
npm(['install', tarball, '--offline', '--ignore-scripts', '--no-audit', '--no-fund'], reader);
assert.ok(existsSync(path.join(reader, 'node_modules', manifest.name)), 'the guide installs into its own directory');
const help = shell('npx --no-install caveat help', reader);
assert.equal(help.status, 0, help.stdout + help.stderr);
const guide = await readFile(path.join(reader, 'node_modules', manifest.name, 'docs', 'GETTING_STARTED.md'), 'utf8');
const followed = await followGuide(guide, { directory: reader, run: shell });
assert.equal(followed.length, 5);
for (const result of followed) {
  assert.equal(result.status, result.expected.includes('FAIL') ? 1 : 0, `${result.command}\n${result.stdout}${result.stderr}`);
  assert.equal(result.actual, result.expected, `the guide's output for ${result.command}`);
}
report.checks.guide = followed.map(result => result.command);

// The packaged worked example, followed the same way in a directory of its own
// below the one the package is installed in.
const exampleReader = path.join(reader, 'worked-example');
await mkdir(exampleReader);
const example = await readFile(path.join(reader, 'node_modules', manifest.name, 'docs', 'WORKED_EXAMPLE.md'), 'utf8');
const worked = await followGuide(example, { directory: exampleReader, run: shell });
assert.equal(worked.length, 4);
for (const result of worked) {
  assert.equal(result.status, 0, `${result.command}\n${result.stdout}${result.stderr}`);
  assert.equal(result.actual, result.expected, `the worked example's output for ${result.command}`);
}
report.checks.workedExample = worked.map(result => result.command);

// The agent-evidence example, copied out of the installed package as its
// README says, and run with its default launcher, `npx --no-install caveat`,
// which finds the installed command.
const agentEvidence = path.join(reader, 'agent-evidence');
await mkdir(agentEvidence);
for (const file of EXAMPLE_FILES) {
  await copyFile(path.join(reader, 'node_modules', manifest.name, file), path.join(agentEvidence, path.posix.basename(file)));
}
// The JavaScript host is executed from the copied example and resolves the
// installed package. Its checkpoint must preserve save text, including -0.
const checkpoint = path.join(agentEvidence, 'save-text-checkpoint.json');
const savedTextExample = node(['save-text.mjs', checkpoint], agentEvidence);
assert.equal(savedTextExample.status, 0, savedTextExample.stdout + savedTextExample.stderr);
assert.deepEqual(JSON.parse(savedTextExample.stdout), { negativeZero: true, angle: -Math.PI, decision: -1 });
assert.match(await readFile(checkpoint, 'utf8'), /"value":-0\.0/);
await unlink(checkpoint);
report.checks.saveTextExample = JSON.parse(savedTextExample.stdout);
for (const command of ['validate', 'check']) {
  const checked = shell(`npx --no-install caveat ${command} assessment.cav`, agentEvidence);
  assert.equal(checked.status, 0, checked.stdout + checked.stderr);
}
const called = await runCallerTests(agentEvidence);
assert.equal(called.status, 0, `${called.python}\n${called.stdout}${called.stderr}`);
// One caller test is for POSIX only; Windows reports it skipped.
assert.match(called.report, process.platform === 'win32' ? /\nOK \(skipped=1\)\n?$/ : /\nOK\n?$/, 'the caller tests report OK');
assert.doesNotMatch(called.report, /ResourceWarning/, 'the caller leaves a pipe or file open');
assert.deepEqual(called.after, called.before, 'the caller tests left files behind');
report.checks.agentEvidence = { python: called.python, tests: Number(/^Ran (\d+) tests?/m.exec(called.report)?.[1]) };
const rc7Called = await runCallerTests(agentEvidence, null, ['test_lifecycle', 'test_qualification', 'test_grounds', 'test_branching']);
assert.equal(rc7Called.status, 0, `${rc7Called.python}\n${rc7Called.stdout}${rc7Called.stderr}`);
assert.match(rc7Called.report, /\nOK\n?$/);
assert.doesNotMatch(rc7Called.report, /ResourceWarning/);
assert.deepEqual(rc7Called.after, rc7Called.before, 'the rc.7 example tests left files behind');
report.checks.agentEvidenceRc7 = { python: rc7Called.python, tests: Number(/^Ran (\d+) tests?/m.exec(rc7Called.report)?.[1]) };


// The session library and runner in a browser, loaded from the installed package.
if (!process.argv.includes('--no-browser')) {
  const outcome = await checkKitInBrowser({ root: consumer, kit: `/node_modules/${manifest.name}/lib/`, runtime: `/node_modules/${manifest.name}/runtime/`, examples: '/' });
  assertBrowserResults(outcome);
  report.checks.browser = outcome.browser;
}

await writeFile(path.join(run, 'report.json'), `${JSON.stringify(report, null, 2)}\n`);
// These are the exact files CI retains. Write the checksum manifest only once
// every package check has passed; do not pack a second tarball for uploading.
const checksums = [];
for (const file of [report.tarball, 'report.json', 'build-info.json']) {
  const digest = sha256(await readFile(path.join(run, file)));
  if (file === report.tarball) assert.equal(digest, report.sha256, 'the tested tarball is unchanged');
  checksums.push(`${digest}  ${file}`);
}
await writeFile(path.join(run, 'SHA256SUMS'), `${checksums.join('\n')}\n`);
console.log(`Packed kit checks pass: ${report.tarball}, ${report.size} bytes, sha256 ${report.sha256}`);
console.log(`Installed from the tarball: command, library${report.checks.browser ? ` and browser (${report.checks.browser})` : ''}. Report: ${path.relative(root, path.join(run, 'report.json'))}`);
