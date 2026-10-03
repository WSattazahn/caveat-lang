// Verifies a caveat-lang version on the public npm registry against the exact
// tested tarball, and produces the evidence for its
// docs/releases/v<version>-npm-publication.json record. It never publishes,
// moves dist-tags or writes to GitHub. From rc.11 on, a version without a
// provenance attestation from publish-npm.yml at the tagged revision fails.
//
//   node scripts/verify-npm-publication.mjs --version 0.1.0-rc.11 --sha256 HEX \
//     --revision SHA --dist-tag latest [--tarball FILE]
//
// Linux/macOS; it runs in .github/workflows/publish-npm.yml after publishing.
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { DIST_TAGS, PACKAGE, REGISTRY } from './release-publish.mjs';

export const REPOSITORY = 'https://github.com/WSattazahn/caveat-lang';
export const PUBLISH_WORKFLOW = '.github/workflows/publish-npm.yml';
const SLSA = 'https://slsa.dev/provenance/v1';
const hash = (algorithm, bytes, encoding = 'hex') => createHash(algorithm).update(bytes).digest(encoding);

// Registry metadata for one version: identity, integrity and the provenance
// attestation npm records for a trusted, provenance-attested publish.
export function evaluateRegistryMetadata(metadata, { version, sha512 }) {
  assert.equal(metadata.name, PACKAGE, 'Registry metadata names another package.');
  assert.equal(metadata.version, version, 'Registry metadata names another version.');
  assert.equal(new URL(metadata.dist?.tarball ?? 'invalid:').origin, REGISTRY, 'The registry tarball is not served by the public registry.');
  assert.equal(metadata.dist.integrity, `sha512-${Buffer.from(sha512, 'hex').toString('base64')}`, 'Registry integrity differs from the tested tarball.');
  const attestations = metadata.dist.attestations;
  assert.ok(attestations && typeof attestations === 'object',
    `Registry metadata for ${PACKAGE}@${version} has no dist.attestations: it was not published with provenance.`);
  assert.equal(attestations.provenance?.predicateType, SLSA, 'The registry records no SLSA provenance attestation.');
  assert.equal(new URL(attestations.url).origin, REGISTRY, 'The attestation is not served by the public registry.');
  return { url: attestations.url, predicateType: attestations.provenance.predicateType };
}

// The provenance statement must name these bytes, this repository, the publish
// workflow, the release tag and its revision.
export function evaluateProvenance(bundle, { version, sha512, tag, revision }) {
  const provenance = (bundle.attestations ?? []).filter(item => item.predicateType === SLSA);
  assert.equal(provenance.length, 1, 'Expected one SLSA provenance attestation.');
  const statement = JSON.parse(Buffer.from(provenance[0].bundle?.dsseEnvelope?.payload ?? '', 'base64').toString('utf8'));
  assert.deepEqual(statement.subject, [{ name: `pkg:npm/${PACKAGE}@${version}`, digest: { sha512 } }], 'Provenance names other bytes.');
  const workflow = statement.predicate?.buildDefinition?.externalParameters?.workflow;
  assert.equal(workflow?.repository, REPOSITORY, 'Provenance names another repository.');
  assert.equal(workflow.path, PUBLISH_WORKFLOW, 'Provenance names another workflow.');
  assert.equal(workflow.ref, `refs/tags/${tag}`, `Provenance names ${workflow.ref}, not the release tag.`);
  const commits = (statement.predicate.buildDefinition.resolvedDependencies ?? []).map(item => item.digest?.gitCommit);
  assert.deepEqual(commits, [revision], 'Provenance names another source revision.');
  return { workflow, revision, builder: statement.predicate.runDetails?.builder?.id ?? null };
}

export function parseArguments(args) {
  const fields = { '--version': 'version', '--sha256': 'sha256', '--revision': 'revision', '--dist-tag': 'distTag', '--tarball': 'tarball' };
  const result = {};
  for (let index = 0; index < args.length; index += 2) {
    const field = fields[args[index]];
    if (!field || result[field] || !args[index + 1]) throw new Error('Expected --version V --sha256 HEX --revision SHA --dist-tag next|latest [--tarball FILE]');
    result[field] = args[index + 1];
  }
  assert.match(result.version ?? '', /^\d+\.\d+\.\d+(?:-rc\.\d+)?$/, '--version is required');
  assert.match(result.sha256 ?? '', /^[0-9a-f]{64}$/, '--sha256 is required');
  assert.match(result.revision ?? '', /^[0-9a-f]{40}$/, '--revision is required');
  assert.ok(DIST_TAGS.includes(result.distTag), `--dist-tag must be one of ${DIST_TAGS.join(', ')}`);
  return result;
}

// A just-published version can take a little while to appear everywhere.
async function fetchOk(url, attempts = 10) {
  for (let attempt = 1; ; attempt++) {
    const response = await fetch(url);
    if (response.ok || attempt === attempts) {
      assert.ok(response.ok, `${url}: HTTP ${response.status}`);
      return response;
    }
    await new Promise(resolve => setTimeout(resolve, 15000));
  }
}

export async function verifyPublication({ version, sha256, revision, distTag, tarball }) {
  const out = path.resolve('test-results', 'npm-publication', version);
  await mkdir(out, { recursive: true });
  const tag = `v${version}`;
  const commands = [];
  const consumer = await mkdtemp(path.join(os.tmpdir(), 'caveat-npm-publication-'));
  function run(executable, args, { json = false } = {}) {
    const result = spawnSync(executable, args, { cwd: consumer, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024 });
    const id = String(commands.length + 1).padStart(2, '0');
    const receipt = { id, executable: path.basename(executable), args, exitCode: result.status };
    for (const stream of ['stdout', 'stderr']) receipt[stream] = { file: `${id}.${stream}.txt`, sha256: hash('sha256', result[stream] ?? '') };
    commands.push(receipt);
    return Promise.all(['stdout', 'stderr'].map(stream => writeFile(path.join(out, `${id}.${stream}.txt`), result[stream] ?? '')))
      .then(() => {
        assert.equal(result.status, 0, `${executable} ${args.join(' ')} failed: ${result.error?.message ?? result.stderr}`);
        return json ? JSON.parse(result.stdout) : result.stdout.trim();
      });
  }

  const metadata = await (await fetchOk(`${REGISTRY}/${PACKAGE}/${version}`)).json();
  const bytes = Buffer.from(await (await fetchOk(metadata.dist.tarball)).arrayBuffer());
  assert.equal(hash('sha256', bytes), sha256, 'Registry tarball bytes differ from the tested tarball.');
  if (tarball) assert.equal(hash('sha256', await readFile(tarball)), sha256, 'The local tarball is not the tested tarball.');
  const sha512 = hash('sha512', bytes);
  const attestation = evaluateRegistryMetadata(metadata, { version, sha512 });
  const bundle = await (await fetchOk(attestation.url)).json();
  await writeFile(path.join(out, 'attestations.json'), `${JSON.stringify(bundle, null, 2)}\n`);
  const provenance = evaluateProvenance(bundle, { version, sha512, tag, revision });
  const channels = await (await fetchOk(`${REGISTRY}/-/package/${PACKAGE}/dist-tags`)).json();
  assert.equal(channels[distTag], version, `npm ${distTag} names ${channels[distTag]}, not ${version}.`);

  // A fresh consumer: new cache, no lifecycle scripts, exact version.
  await writeFile(path.join(consumer, 'package.json'), JSON.stringify({ name: 'caveat-publication-check', version: '1.0.0', private: true }));
  const npmVersion = await run('npm', ['--version']);
  await run('npm', ['install', `${PACKAGE}@${version}`, '--ignore-scripts', '--no-audit', '--no-fund', '--registry', REGISTRY, '--cache', path.join(consumer, 'cache')]);
  const lock = JSON.parse(await readFile(path.join(consumer, 'package-lock.json'), 'utf8')).packages[`node_modules/${PACKAGE}`];
  assert.equal(lock.version, version);
  assert.equal(lock.integrity, metadata.dist.integrity);
  assert.equal(new URL(lock.resolved).origin, REGISTRY);
  // Verifies the registry signature and the provenance attestation cryptographically.
  const signatures = await run('npm', ['audit', 'signatures', '--registry', REGISTRY, '--cache', path.join(consumer, 'cache')]);
  assert.match(signatures, /verified registry signature/, 'npm did not verify the registry signature.');
  assert.match(signatures, /verified attestation/, 'npm did not verify the provenance attestation.');
  const bin = name => path.join(consumer, 'node_modules', '.bin', name);
  for (const alias of ['caveat', 'caveat-lang']) assert.equal(await run(bin(alias), ['--version']), `CAVEAT Language ${version}`);
  const doctor = await run(bin('caveat-lang'), ['doctor', '--json'], { json: true });
  assert.ok(doctor.ok, 'doctor reported a problem');
  assert.equal(doctor.package.version, version);
  assert.equal(doctor.runtime.buildInfo.revision, revision, 'The installed runtime was built from another revision.');
  const demo = await run(bin('caveat-lang'), ['demo', 'agent', '--json'], { json: true });
  assert.ok(demo.preservation.unchanged, 'demo changed project files');
  await run(bin('caveat-lang'), ['init']);
  await run(bin('caveat-lang'), ['test', 'umbrella.scenarios.json']);
  await run(bin('caveat-lang'), ['explain', 'umbrella.cav', 'events.jsonl']);

  const verification = { schema: 'caveat-npm-publication-verification/0.1', name: PACKAGE, version, revision, tag,
    verifiedAt: new Date().toISOString(), registry: REGISTRY,
    tarball: { url: metadata.dist.tarball, bytes: bytes.length, sha256, sha512, integrity: metadata.dist.integrity },
    attestation, provenance, channels,
    checks: { registryBytesMatch: true, provenanceAttested: true, signaturesVerified: true, freshExactVersionInstall: true,
      freshCache: true, lockIntegrityAndOrigin: true, cliAliases: ['caveat', 'caveat-lang'], doctor: true, demo: true,
      umbrellaScenarios: true, umbrellaExplanation: true, successfulCommands: commands.length },
    tools: { node: process.version, platform: process.platform, architecture: process.arch, npm: npmVersion },
    run: process.env.GITHUB_RUN_ID ?? null, commands };
  await writeFile(path.join(out, 'npm-publication-verification.json'), `${JSON.stringify(verification, null, 2)}\n`);
  return { out, verification };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const { out, verification } = await verifyPublication(parseArguments(process.argv.slice(2)));
    console.log(`Verified ${PACKAGE}@${verification.version} on npm with provenance from ${verification.provenance.workflow.ref}: ${path.join(out, 'npm-publication-verification.json')}`);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
